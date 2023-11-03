use axum::extract::{Path, State};
use maud::{html, Markup};
use posts::projects::Projects;
use reqwest::StatusCode;

use crate::{http_server::templates::base_constrained, *};

use super::blog::md::IntoHtml;

#[instrument(skip_all)]
pub(crate) async fn projects_index(
    State(projects): State<Arc<Projects>>,
) -> Result<Markup, StatusCode> {
    let projects = projects.by_title();

    Ok(base_constrained(html! {
      h1 class="text-3xl" { "Projects" }

      ul {
        @for project in &projects {
          li class="my-4" {
            a href=(project.relative_link().unwrap()) {
              (project.frontmatter.title)
            }
          }
        }
      }
    }))
}

#[instrument(skip(streams, projects))]
#[axum_macros::debug_handler(state = AppState)]
pub(crate) async fn projects_get(
    State(projects): State<Arc<Projects>>,
    State(streams): State<Arc<PastStreams>>,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Markup, StatusCode> {
    let project = projects
        .projects
        .iter()
        .find(|p| p.slug().unwrap() == slug)
        .ok_or(StatusCode::NOT_FOUND)?;

    let streams: Vec<_> = streams
        .by_recency()
        .into_iter()
        .filter(|s| s.frontmatter.project.as_ref() == Some(&slug))
        .collect();

    let markdown = project.ast.0.clone().into_html(&state);

    Ok(base_constrained(html! {
      h1 class="text-3xl" { (project.frontmatter.title) }
      @if let Some(sub) = &project.frontmatter.subtitle {
        h2 class="text-xl mb-4" { (sub) }
      }
      a href=(&project.frontmatter.repo) target="_blank" rel="noopener noreferrer" {
        i class="fa-brands fa-github pb-8" {}
      }

      (markdown)

      h3 class="text-lg mt-8" { "Streams" }
      (pages::streams::StreamPostList(streams))
    }))
}
