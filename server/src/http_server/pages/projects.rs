use axum::extract::{Path, State};
use maud::{html, Markup, Render};
use posts::projects::{ProjectStatus, Projects};
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

struct StatusTag(ProjectStatus);

impl StatusTag {
    fn color_class(&self) -> &'static str {
        match self.0 {
            ProjectStatus::Active => "fill-success-400",
            ProjectStatus::Maintenance => "fill-warning-400",
            ProjectStatus::OnIce => "fill-blue-400",
            ProjectStatus::Complete => "fill-success-200",
            ProjectStatus::Archived => "fill-grey-400",
        }
    }
}

impl Render for StatusTag {
    fn render(&self) -> Markup {
        html! {
          span class="inline-flex items-center gap-x-1.5 rounded-md px-2 py-1 text-xs font-medium text-white ring-1 ring-inset ring-grey-800" {
            svg class=(format!("h-1.5 w-1.5 {}", self.color_class())) viewBox="0 0 6 6" aria-hidden="true" {
              circle cx="3" cy="3" r="3";
            }
            (self.0)
          }
        }
    }
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
      div class="flex flex-row pb-8 align-middle" {
        (StatusTag(project.frontmatter.status))
        a href=(&project.frontmatter.repo) target="_blank" rel="noopener noreferrer" class="mx-2 py-3" {
          i class="fa-brands fa-github" {}
        }
      }

      (markdown)

      h3 class="text-lg mt-8" { "Streams" }
      (pages::streams::StreamPostList(streams))
    }))
}