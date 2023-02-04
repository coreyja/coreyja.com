use axum::{
    extract::{Path, State},
    Form,
};
use color_eyre::eyre::eyre;
use maud::{html, Markup, PreEscaped};

use crate::*;

pub(crate) async fn upwork_proposal_get(
    Path(id): Path<String>,
    State(config): State<Config>,
) -> Result<Markup, http_server::EyreError> {
    let db_record = sqlx::query!("SELECT * FROM UpworkJobs where id = ?", id)
        .fetch_optional(&config.db_pool)
        .await?;

    let db_record = db_record.ok_or_else(|| eyre!("No record found for id {}", id))?;

    let template = include_str!("../data/proposal_templates/logo.md");

    Ok(html! {
        h1 { "Upwork Job: " (db_record.title) }
        p { (PreEscaped(&db_record.content)) }

        form method="post" {
          textarea name="prompt" {
            (template)

          }

          button type="submit" { "Submit" }
        }
    })
}

#[derive(Deserialize)]
pub(crate) struct ProposalForm {
    prompt: String,
}

pub(crate) async fn upwork_proposal_post(
    Path(id): Path<String>,
    State(config): State<Config>,
    Form(form): Form<ProposalForm>,
) -> Result<Markup, http_server::EyreError> {
    let db_record = sqlx::query!("SELECT * FROM UpworkJobs where id = ?", id)
        .fetch_optional(&config.db_pool)
        .await?;
    let db_record = db_record.ok_or_else(|| eyre!("No record found for id {}", id))?;

    let prompt = form.prompt;
    let completion = open_ai::complete_prompt(&config.open_ai, &prompt).await?;

    Ok(html! {
        h1 { "Upwork Job: " (db_record.title) }
        p { (PreEscaped(&db_record.content)) }

        h2 { "Completion Ran"}
        form method="post" {
          textarea {
            (completion)
          }

          button type="submit" { "Submit" }
        }
    })
}
