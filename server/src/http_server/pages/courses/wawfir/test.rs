use axum::{extract::State, response::IntoResponse};
use base64::Engine;
use chrono::Duration;
use maud::html;
use miette::{Context, IntoDiagnostic};

use crate::{
    http_server::{errors::MietteError, templates::base_constrained},
    state::{AppState, MuxConfig},
};

pub(crate) async fn get(State(app): State<AppState>) -> Result<impl IntoResponse, MietteError> {
    let video_id = "47OvdD9b01Nhy6HkOUk6S00YduJmx01LPtG8nRyZyx9CVU";
    let token = sign(&app.mux, video_id, Audience::Video, Duration::hours(24)).into_diagnostic()?;
    Ok(base_constrained(
        html! {
          script src="https://cdn.jsdelivr.net/npm/@mux/mux-player" {}
          mux-player
            playback-id=(video_id)
            playback-token=(token)
            metadata-video-title="Placeholder (optional)"
            metadata-viewer-user-id="Placeholder (optional)"
            accent-color="#FF0000"
          {}
        },
        Default::default(),
    ))
}
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Claims {
    sub: String,
    exp: i64,
    kid: String,
    aud: String,
}

#[derive(Debug, Clone)]
pub enum Audience {
    Video,
    Thumbnail,
    Gif,
    Storyboard,
}

impl Audience {
    const fn as_str(&self) -> &str {
        match self {
            Self::Video => "v",
            Self::Thumbnail => "t",
            Self::Gif => "g",
            Self::Storyboard => "s",
        }
    }
}

pub fn sign(
    mux: &MuxConfig,
    video_id: &str,
    audience: Audience,
    expires_after: Duration,
) -> Result<String, MietteError> {
    let private_key = &mux.signing_private_key;
    let private_key = base64::engine::general_purpose::STANDARD
        .decode(private_key)
        .into_diagnostic()
        .wrap_err("Couldn't base64 decode mux private key")?;
    let rsa_private = EncodingKey::from_rsa_pem(&private_key)
        .into_diagnostic()
        .wrap_err("Couldn't create decoding key")?;

    let signing_key_id = &mux.signing_key_id;
    let now = chrono::Utc::now();
    let expires = (now + expires_after).timestamp();
    let payload = Claims {
        sub: video_id.to_string(),
        exp: expires,
        kid: signing_key_id.to_string(),
        aud: audience.as_str().to_string(),
    };
    let header = Header {
        alg: jsonwebtoken::Algorithm::RS256,
        ..Header::default()
    };
    let token = encode(&header, &payload, &rsa_private)
        .into_diagnostic()
        .wrap_err("Could not encode the jwt for mux signed token")?;
    Ok(token)
}
