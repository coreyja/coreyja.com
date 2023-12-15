use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

use super::Job;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshVideos;

#[async_trait::async_trait]
impl Job for RefreshVideos {
    const NAME: &'static str = "RefreshVideos";

    async fn run(&self, _app_state: crate::AppState) -> miette::Result<()> {
        let hub = google_youtube3::YouTube::new(
            google_youtube3::hyper::Client::builder().build(
                google_youtube3::hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
            std::env::var("COREYJA_YOUTUBE_ACCESS_TOKEN")
                .unwrap()
                .to_string(),
        );

        let result = hub
            .videos()
            .list(&vec!["coreyja".to_string()])
            .doit()
            .await
            .into_diagnostic()?;
        dbg!(result);

        Ok(())
    }
}
