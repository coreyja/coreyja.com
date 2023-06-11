use miette::{IntoDiagnostic, Result};
use sitemap::{reader::SiteMapReader, structs::UrlEntry};

#[tokio::main]
async fn main() -> Result<()> {
    let body = reqwest::get("https://coreyja.com/sitemap.xml")
        .await
        .into_diagnostic()?
        .text()
        .await
        .into_diagnostic()?;

    let parser = SiteMapReader::new(body.as_bytes());

    for url in parser {
        if let sitemap::reader::SiteMapEntity::Url(UrlEntry {
            loc: sitemap::structs::Location::Url(url),
            ..
        }) = url
        {
            let mut new_url = url.clone();
            new_url.set_scheme("https").unwrap();
            new_url
                .set_host(Some("beta.coreyja.com"))
                .into_diagnostic()?;

            let response = reqwest::get(new_url.clone()).await.into_diagnostic()?;

            if response.status().is_success() {
                println!("✅: {new_url}");
            } else {
                println!("❌: {new_url}");
            }
        }
    }

    Ok(())
}
