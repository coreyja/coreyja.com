use candle_core::{DType, Device, Tensor};
use candle_nn::{ops::softmax, VarBuilder};
// use clap::{Parser, ValueEnum};
use hf_hub::{api::sync::Api, Repo, RepoType};
use miette::IntoDiagnostic;
use rand::{distributions::Distribution, SeedableRng};
use tokenizers::Tokenizer;

pub(crate) mod audio;
mod model;
pub(crate) use model::{Config, Whisper};
mod multilingual;

pub use miette::Result;

pub(crate) const DTYPE: DType = DType::F32;

// Audio parameters.
const SAMPLE_RATE: usize = 16000;
const N_FFT: usize = 400;
pub(crate) const N_MELS: usize = 80;
const HOP_LENGTH: usize = 160;
const CHUNK_LENGTH: usize = 30;
const N_SAMPLES: usize = CHUNK_LENGTH * SAMPLE_RATE; // 480000 samples in a 30-second chunk
const N_FRAMES: usize = N_SAMPLES / HOP_LENGTH; // 3000 frames in a mel spectrogram input

const NO_SPEECH_THRESHOLD: f64 = 0.6;
const LOGPROB_THRESHOLD: f64 = -1.0;
const TEMPERATURES: [f64; 6] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
const COMPRESSION_RATIO_THRESHOLD: f64 = 2.4;

// Tokenizer dependent bits.
const SOT_TOKEN: &str = "<|startoftranscript|>";
const TRANSCRIBE_TOKEN: &str = "<|transcribe|>";
const EOT_TOKEN: &str = "<|endoftext|>";
const NO_SPEECH_TOKEN: &str = "<|nocaptions|>";

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DecodingResult {
    tokens: Vec<u32>,
    text: String,
    avg_logprob: f64,
    no_speech_prob: f64,
    temperature: f64,
    compression_ratio: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct Segment {
    start: f64,
    duration: f64,
    dr: DecodingResult,
}

pub(crate) struct Decoder {
    model: Whisper,
    rng: rand::rngs::StdRng,
    tokenizer: Tokenizer,
    suppress_tokens: Tensor,
    sot_token: u32,
    transcribe_token: u32,
    eot_token: u32,
    no_speech_token: u32,
    language_token: Option<u32>,
}

impl Decoder {
    pub(crate) fn new(
        model: Whisper,
        tokenizer: Tokenizer,
        seed: u64,
        device: &Device,
        language_token: Option<u32>,
    ) -> Result<Self> {
        let suppress_tokens: Vec<f32> = (0..model.config.vocab_size as u32)
            .map(|i| {
                if model.config.suppress_tokens.contains(&i) {
                    f32::NEG_INFINITY
                } else {
                    0f32
                }
            })
            .collect();
        let suppress_tokens = Tensor::new(suppress_tokens.as_slice(), device).into_diagnostic()?;
        let sot_token = token_id(&tokenizer, SOT_TOKEN).into_diagnostic()?;
        let transcribe_token = token_id(&tokenizer, TRANSCRIBE_TOKEN).into_diagnostic()?;
        let eot_token = token_id(&tokenizer, EOT_TOKEN).into_diagnostic()?;
        let no_speech_token = token_id(&tokenizer, NO_SPEECH_TOKEN).into_diagnostic()?;
        Ok(Self {
            model,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            tokenizer,
            suppress_tokens,
            sot_token,
            transcribe_token,
            eot_token,
            no_speech_token,
            language_token,
        })
    }

    fn decode(&mut self, mel: &Tensor, t: f64) -> Result<DecodingResult> {
        let model = &mut self.model;
        let audio_features = model.encoder.forward(mel, true).into_diagnostic()?;
        println!("audio features: {:?}", audio_features.dims());
        let sample_len = model.config.max_target_positions / 2;
        let mut sum_logprob = 0f64;
        let mut no_speech_prob = f64::NAN;
        let mut tokens = vec![self.sot_token];
        if let Some(language_token) = self.language_token {
            tokens.push(language_token)
        }
        tokens.push(self.transcribe_token);
        for i in 0..sample_len {
            let tokens_t = Tensor::new(tokens.as_slice(), mel.device()).into_diagnostic()?;

            // The model expects a batch dim but this inference loop does not handle
            // it so we add it at this point.
            let tokens_t = tokens_t.unsqueeze(0).into_diagnostic()?;
            let logits = model
                .decoder
                .forward(&tokens_t, &audio_features, i == 0)
                .into_diagnostic()?;
            let logits = logits.squeeze(0).into_diagnostic()?;

            // Extract the no speech probability on the first iteration by looking at the first
            // token logits and the probability for the according token.
            if i == 0 {
                no_speech_prob = softmax(&logits.get(0).into_diagnostic()?, 0)
                    .into_diagnostic()?
                    .get(self.no_speech_token as usize)
                    .into_diagnostic()?
                    .to_scalar::<f32>()
                    .into_diagnostic()? as f64;
            }

            let (seq_len, _) = logits.dims2().into_diagnostic()?;
            let logits = logits
                .get(seq_len - 1)
                .into_diagnostic()?
                .broadcast_add(&self.suppress_tokens)
                .into_diagnostic()?;
            let next_token = if t > 0f64 {
                let prs = softmax(&(&logits / t).into_diagnostic()?, 0).into_diagnostic()?;
                let logits_v: Vec<f32> = prs.to_vec1().into_diagnostic()?;
                let distr = rand::distributions::WeightedIndex::new(&logits_v).into_diagnostic()?;
                distr.sample(&mut self.rng) as u32
            } else {
                let logits_v: Vec<f32> = logits.to_vec1().into_diagnostic()?;
                logits_v
                    .iter()
                    .enumerate()
                    .max_by(|(_, u), (_, v)| u.total_cmp(v))
                    .map(|(i, _)| i as u32)
                    .unwrap()
            };
            tokens.push(next_token);
            let prob = softmax(&logits, candle_core::D::Minus1)
                .into_diagnostic()?
                .get(next_token as usize)
                .into_diagnostic()?
                .to_scalar::<f32>()
                .into_diagnostic()? as f64;
            if next_token == self.eot_token || tokens.len() > model.config.max_target_positions {
                break;
            }
            sum_logprob += prob.ln();
        }
        let text = self
            .tokenizer
            .decode(tokens.clone(), true)
            .map_err(|e| miette::miette!(e))?;
        let avg_logprob = sum_logprob / tokens.len() as f64;

        Ok(DecodingResult {
            tokens,
            text,
            avg_logprob,
            no_speech_prob,
            temperature: t,
            compression_ratio: f64::NAN,
        })
    }

    fn decode_with_fallback(&mut self, segment: &Tensor) -> Result<DecodingResult> {
        for (i, &t) in TEMPERATURES.iter().enumerate() {
            let dr: Result<DecodingResult> = self.decode(segment, t);
            if i == TEMPERATURES.len() - 1 {
                return dr;
            }
            // On errors, we try again with a different temperature.
            match dr {
                Ok(dr) => {
                    let needs_fallback = dr.compression_ratio > COMPRESSION_RATIO_THRESHOLD
                        || dr.avg_logprob < LOGPROB_THRESHOLD;
                    if !needs_fallback || dr.no_speech_prob > NO_SPEECH_THRESHOLD {
                        return Ok(dr);
                    }
                }
                Err(err) => {
                    println!("Error running at {t}: {err}")
                }
            }
        }
        unreachable!()
    }

    pub(crate) fn run(&mut self, mel: &Tensor) -> miette::Result<Vec<Segment>> {
        let (_, _, content_frames) = mel.dims3().into_diagnostic()?;
        let mut seek = 0;
        let mut segments = vec![];
        while seek < content_frames {
            let start = std::time::Instant::now();
            let time_offset = (seek * HOP_LENGTH) as f64 / SAMPLE_RATE as f64;
            let segment_size = usize::min(content_frames - seek, N_FRAMES);
            let mel_segment = mel.narrow(2, seek, segment_size).into_diagnostic()?;
            let segment_duration = (segment_size * HOP_LENGTH) as f64 / SAMPLE_RATE as f64;
            let dr = self.decode_with_fallback(&mel_segment)?;
            seek += segment_size;
            if dr.no_speech_prob > NO_SPEECH_THRESHOLD && dr.avg_logprob < LOGPROB_THRESHOLD {
                println!("no speech detected, skipping {seek} {dr:?}");
                continue;
            }
            let segment = Segment {
                start: time_offset,
                duration: segment_duration,
                dr,
            };
            println!("{seek}: {segment:?}, in {:?}", start.elapsed());
            segments.push(segment)
        }
        Ok(segments)
    }
}

pub fn token_id(tokenizer: &Tokenizer, token: &str) -> candle_core::Result<u32> {
    match tokenizer.token_to_id(token) {
        None => candle_core::bail!("no token-id for {token}"),
        Some(id) => Ok(id),
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum WhichModel {
    Tiny,
    TinyEn,
    Base,
    BaseEn,
    SmallEn,
    MediumEn,
    LargeV2,
}

impl WhichModel {
    fn is_multilingual(&self) -> bool {
        match self {
            Self::Tiny | Self::Base | Self::LargeV2 => true,
            Self::TinyEn | Self::BaseEn | Self::SmallEn | Self::MediumEn => false,
        }
    }
    pub(crate) fn model_and_revision(&self) -> (&'static str, &'static str) {
        match self {
            Self::Tiny => ("openai/whisper-tiny", "main"),
            Self::TinyEn => ("openai/whisper-tiny.en", "refs/pr/15"),
            Self::Base => ("openai/whisper-base", "refs/pr/22"),
            Self::BaseEn => ("openai/whisper-base.en", "refs/pr/13"),
            Self::SmallEn => ("openai/whisper-small.en", "refs/pr/10"),
            Self::MediumEn => ("openai/whisper-medium.en", "refs/pr/11"),
            Self::LargeV2 => ("openai/whisper-large-v2", "refs/pr/57"),
        }
    }
}
