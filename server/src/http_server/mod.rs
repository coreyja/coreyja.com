use axum::{
    extract::{Path, State},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
    routing::*,
    Router, Server,
};
use chrono::{DateTime, NaiveTime, Utc};
use include_dir::*;
use miette::{Context, IntoDiagnostic};
use posts::{
    blog::{BlogPost, ToCanonicalPath},
    date::PostedOn,
    past_streams::PastStream,
    til::TilPost,
    title::Title,
    Post,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;

use crate::{AppConfig, AppState};
pub use config::*;
use errors::*;

use self::{pages::blog::md::IntoHtml, templates::ShortDesc};

pub(crate) mod cmd;

pub(crate) mod pages {
    pub mod blog;
    pub mod home;
    pub mod projects;
    pub mod streams;
    pub mod til;
}

mod config;
pub mod errors;
mod routes;
mod server_tracing;
mod templates;

const TAILWIND_STYLES: &str = include_str!("../../../target/tailwind.css");

const STATIC_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

type ResponseResult<T = Response> = Result<T, MietteError>;

pub(crate) async fn run_axum(config: AppState) -> miette::Result<()> {
    let syntax_css = syntect::html::css_for_theme_with_class_style(
        &config.markdown_to_html_context.theme,
        syntect::html::ClassStyle::Spaced,
    )
    .unwrap();

    let tracer = server_tracing::Tracer;
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(tracer)
        .on_response(tracer);

    let app = routes::make_router(syntax_css)
        .with_state(config)
        .layer(trace_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
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
    fn to_rss_item(&self, state: &AppState) -> rss::Item;
}

impl<FrontMatter> ToRssItem for Post<FrontMatter>
where
    FrontMatter: PostedOn + Title,
    Post<FrontMatter>: LinkTo,
{
    fn to_rss_item(&self, state: &AppState) -> rss::Item {
        let link = self.absolute_link(&state.app);

        let posted_on: DateTime<Utc> = self.posted_on().and_time(NaiveTime::MIN).and_utc();
        let formatted_date = posted_on.to_rfc2822();

        rss::ItemBuilder::default()
            .title(Some(self.title().to_string()))
            .link(Some(link))
            .description(self.short_description())
            .pub_date(Some(formatted_date))
            .content(Some(self.markdown().ast.0.into_html(state).into_string()))
            .build()
    }
}
