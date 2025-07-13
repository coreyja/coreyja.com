use axum::{extract::State, response::IntoResponse, Form};
use chrono::Utc;
use maud::html;
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

use cja::cron::Schedule;

use super::{
    super::{
        auth::session::AdminUser,
        components::AutoRefreshButton,
        errors::ServerError,
        templates::{base_constrained, header::OpenGraph},
    },
    Timestamp,
};

pub(crate) async fn list_crons(
    _admin: AdminUser,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let crons = sqlx::query!(
        "SELECT cron_id, name, last_run_at, created_at, updated_at FROM Crons ORDER BY name"
    )
    .fetch_all(&app_state.db)
    .await?;

    let job_configs = crate::cron::cron_registry()?;

    let timezone = cja::chrono_tz::US::Eastern;

    Ok(base_constrained(
        html! {
            div class="flex items-center justify-between mb-4" {
                h1 class="text-xl" { "Cron Management" }
                (AutoRefreshButton::new("#cron-table", Some("/admin/crons")).render())
            }

            a href="/admin" class="text-blue-500 hover:underline mb-4 inline-block" { "← Back to Admin Dashboard" }

            div class="overflow-x-auto" id="cron-table-container" {
                table class="min-w-full bg-white border border-gray-300" id="cron-table" {
                    thead {
                        tr class="bg-gray-100" {
                            th class="px-4 py-2 border" { "Name" }
                            th class="px-4 py-2 border" { "Type" }
                            th class="px-4 py-2 border" { "Last Run At" }
                            th class="px-4 py-2 border" { "Next Run At" }
                            th class="px-4 py-2 border" { "Created At" }
                            th class="px-4 py-2 border" { "Updated At" }
                            th class="px-4 py-2 border" { "Actions" }
                        }
                    }
                    tbody {
                        @for cron in crons {
                            @let job_config = job_configs.get(cron.name.as_str());
                            @let (job_type_label, job_type_class) = if let Some(config) = job_config {
                                match &config.schedule {
                                    Schedule::Interval { .. } => ("Interval", "bg-blue-100 text-blue-800"),
                                    Schedule::Cron { .. } => ("Cron", "bg-green-100 text-green-800"),
                                }
                            } else {
                                ("Unknown", "bg-gray-100 text-gray-800")
                            };
                            @let next_run = job_config.map(|config| config.schedule.next_run(Some(&cron.last_run_at), Utc::now(), timezone));

                            tr {
                                td class="px-4 py-2 border" { (cron.name) }
                                td class="px-4 py-2 border" {
                                    span class={"px-2 py-1 text-xs font-semibold rounded " (job_type_class)} {
                                        (job_type_label)
                                    }
                                }
                                td class="px-4 py-2 border" { (Timestamp(cron.last_run_at)) }
                                td class="px-4 py-2 border" {
                                    @if let Some(next_run_time) = next_run {
                                        (Timestamp(next_run_time))
                                    } @else {
                                        span class="text-gray-500" { "N/A" }
                                    }
                                }
                                td class="px-4 py-2 border" { (Timestamp(cron.created_at)) }
                                td class="px-4 py-2 border" { (Timestamp(cron.updated_at)) }
                                td class="px-4 py-2 border text-center" {
                                    form action="/admin/crons/reset" method="post" class="inline" {
                                        input type="hidden" name="cron_id" value=(cron.cron_id.to_string());
                                        input type="submit" value="Reset Last Run" class="px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600 cursor-pointer";
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        OpenGraph::default(),
    ))
}

#[derive(Deserialize)]
pub(crate) struct ResetCronForm {
    cron_id: Uuid,
}

pub(crate) async fn reset_cron(
    _admin: AdminUser,
    State(app_state): State<AppState>,
    Form(form): Form<ResetCronForm>,
) -> Result<impl IntoResponse, ServerError> {
    // Set last_run_at to a very old date (epoch) to ensure the cron will run on next check
    let epoch = chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    sqlx::query!(
        "UPDATE Crons SET last_run_at = $1, updated_at = NOW() WHERE cron_id = $2",
        epoch,
        form.cron_id
    )
    .execute(&app_state.db)
    .await?;

    // Redirect back to the crons list
    Ok(axum::response::Redirect::to("/admin/crons"))
}
