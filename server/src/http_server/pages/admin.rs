use axum::{
    extract::{Path, State},
    Form,
};
use maud::{html, Markup, PreEscaped};

use crate::*;

pub(crate) async fn upwork_proposal_get(
    Path(id): Path<String>,
    State(config): State<AppState>,
) -> Result<Markup, http_server::MietteError> {
    let db_record = sqlx::query!("SELECT * FROM UpworkJobs where id = ?", id)
        .fetch_optional(&config.db_pool)
        .await
        .into_diagnostic()?;

    let db_record = db_record.ok_or_else(|| miette::miette!("No record found for id {}", id))?;

    let sample_proposal = include_str!("../../data/proposal_templates/logo.md");

    let template_instructions = include_str!("../../data/proposal_templates/instructions.md");
    let template_contents = format!("{}\n{}", db_record.title, db_record.content);
    let instructions = template_instructions.replace("{job_posting}", &template_contents);
    let instructions = instructions.replace("{sample_proposal}", sample_proposal);

    Ok(html! {
        h1 { "Upwork Job: " (db_record.title) }
        p { (PreEscaped(&db_record.content)) }

        form method="post" {
          textarea name="prompt" {
            (instructions)
          }

          button type="submit" { "Edit" }
        }
    })
}

#[derive(Deserialize)]
pub(crate) struct ProposalForm {
    prompt: String,
}

pub(crate) async fn upwork_proposal_post(
    Path(id): Path<String>,
    State(config): State<AppState>,
    Form(form): Form<ProposalForm>,
) -> Result<Markup, http_server::MietteError> {
    let db_record = sqlx::query!("SELECT * FROM UpworkJobs where id = ?", id)
        .fetch_optional(&config.db_pool)
        .await
        .into_diagnostic()?;
    let db_record = db_record.ok_or_else(|| miette::miette!("No record found for id {}", id))?;

    let prompt = form.prompt;

    let edit = open_ai::complete_prompt(&config.open_ai, &prompt).await?;

    Ok(html! {
        h1 { "Upwork Job: " (db_record.title) }
        p { (PreEscaped(&db_record.content)) }

        h2 { "Edited" }
        form method="post" {
          textarea name="prompt" {
            (prompt)
          }

          button type="submit" { "Edit" }
        }
        p style="white-space: pre-wrap" { (PreEscaped(&edit))}
    })
}
