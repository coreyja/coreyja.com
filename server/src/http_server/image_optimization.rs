use std::{
    collections::HashMap,
    io::{BufWriter, Cursor},
};

use image::{io::Reader, ImageFormat};
use include_dir::{Dir, File};
use miette::IntoDiagnostic;
use mime_guess::mime;
use ssri::Integrity;
use tracing::{info, instrument};

#[derive(Debug)]
pub(crate) struct Assets<'dir> {
    entries: HashMap<String, Asset<'dir>>,
}

pub(crate) const CACHE_DIR: &str = "./.cache/images";

#[derive(Debug)]
pub(crate) enum Asset<'dir> {
    Image(Image<'dir>),
    Other(File<'dir>),
}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) struct Image<'dir> {
    pub orig: File<'dir>,
    pub resized_hash: Integrity,
}

impl<'dir> Asset<'dir> {
    #[instrument(skip_all, fields(path = %f.path().to_string_lossy()))]
    pub async fn from_file(f: File<'dir>) -> crate::Result<Asset<'dir>> {
        let path = f.path();
        let mime = mime_guess::from_path(path).first_or_octet_stream();

        Ok(
            if let (mime::IMAGE, mime::JPEG | mime::PNG) = (mime.type_(), mime.subtype()) {
                let contents = f.contents();
                let reader = Reader::new(Cursor::new(contents))
                    .with_guessed_format()
                    .expect("Cursor io never fails");

                let image = reader.decode().into_diagnostic()?;
                let image = image.resize(1000, 600, image::imageops::FilterType::Triangle);

                let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
                image.write_to(&mut buffer, ImageFormat::WebP).unwrap();

                let ssri = cacache::write(
                    CACHE_DIR,
                    &path.to_string_lossy(),
                    buffer.into_inner().into_diagnostic()?.into_inner(),
                )
                .await?;

                info!(%ssri, %mime, "Wrote image to cache");
                let i = Image {
                    orig: f,
                    resized_hash: ssri,
                };
                Asset::Image(i)
            } else {
                info!(%mime, "Not a convertable image");
                Asset::Other(f)
            },
        )
    }
}

impl<'dir> Assets<'dir> {
    #[instrument(skip(dir))]
    pub async fn from_dir(dir: Dir<'dir>) -> crate::Result<Assets<'dir>> {
        let mut entries = HashMap::new();

        for f in dir.files().cloned() {
            let path = f.path();
            let path = path.to_string_lossy().to_string();
            let asset = Asset::from_file(f).await?;
            entries.insert(path, asset);
        }

        Ok(Assets { entries })
    }

    pub fn get(&self, path: &str) -> Option<&Asset<'dir>> {
        self.entries.get(path)
    }
}
