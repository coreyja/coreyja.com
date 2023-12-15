use axum::{
    extract::{Path, State},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
    routing::*,
    Router,
};
use chrono::{DateTime, NaiveTime, Utc};
use include_dir::*;
use miette::{Context, IntoDiagnostic, Result};
use posts::{
    blog::{BlogPost, ToCanonicalPath},
    date::PostedOn,
    past_streams::PastStream,
    til::TilPost,
    title::Title,
    Post,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

use crate::{AppConfig, AppState};
pub use config::*;
use errors::*;

use self::{
    pages::blog::md::{IntoHtml, SyntaxHighlightingContext},
    templates::ShortDesc,
};

pub(crate) mod cmd;

pub(crate) mod pages;

mod config;
pub mod errors;
mod routes;
mod server_tracing;
mod templates;

pub(crate) mod auth;

pub(crate) mod admin;

const TAILWIND_STYLES: &str = include_str!("../../../target/tailwind.css");
const COMIC_CODE_STYLES: &str = include_str!("../styles/comic_code.css");

const STATIC_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

type ResponseResult<T = Response> = Result<T, MietteError>;

pub(crate) async fn run_axum(config: AppState) -> miette::Result<()> {
    let syntax_css = syntect::html::css_for_theme_with_class_style(
        &config.markdown_to_html_context.theme,
        syntect::html::ClassStyle::Spaced,
    )
    .into_diagnostic()?;

    let tracer = server_tracing::Tracer;
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(tracer)
        .on_response(tracer);

    let app = routes::make_router(syntax_css)
        .with_state(config)
        .layer(trace_layer)
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", addr);

    axum::serve(listener, app)
        .await
        .into_diagnostic()
        .wrap_err("Failed to run server")
}

pub(crate) trait LinkTo {
    fn relative_link(&self) -> String;

    fn absolute_link(&self, config: &AppConfig) -> String {
        config.app_url(&self.relative_link())
    }
}

impl LinkTo for BlogPost {
    fn relative_link(&self) -> String {
        format!("/posts/{}", self.path.canonical_path())
    }
}

impl LinkTo for TilPost {
    fn relative_link(&self) -> String {
        format!("/til/{}", self.frontmatter.slug)
    }
}

impl LinkTo for PastStream {
    fn relative_link(&self) -> String {
        format!("/streams/{}", self.frontmatter.date)
    }
}

pub(crate) trait ToRssItem {
    fn to_rss_item(
        &self,
        config: &AppConfig,
        context: &SyntaxHighlightingContext,
    ) -> Result<rss::Item>;
}

impl<FrontMatter> ToRssItem for Post<FrontMatter>
where
    FrontMatter: PostedOn + Title,
    Post<FrontMatter>: LinkTo,
{
    fn to_rss_item(
        &self,
        config: &AppConfig,
        context: &SyntaxHighlightingContext,
    ) -> Result<rss::Item> {
        let link = self.absolute_link(config);

        let posted_on: DateTime<Utc> = self.posted_on().and_time(NaiveTime::MIN).and_utc();
        let formatted_date = posted_on.to_rfc2822();

        Ok(rss::ItemBuilder::default()
            .title(Some(self.title().to_string()))
            .link(Some(link))
            .description(self.short_description())
            .pub_date(Some(formatted_date))
            .content(Some(
                self.markdown()
                    .ast
                    .0
                    .into_html(config, context)?
                    .into_string(),
            ))
            .build())
    }
}
