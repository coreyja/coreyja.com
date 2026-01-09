use axum::{extract::State, response::IntoResponse, Form};
use chrono::Utc;
use maud::html;
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

use cja::cron::{CronSchedule, IntervalSchedule, Schedule};

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

    let job_configs = crate::cron::cron_registry();

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
                            @let (job_type_label, job_type_class, schedule_details) = if let Some(config) = job_config {
                                match &config.schedule {
                                    Schedule::Interval(IntervalSchedule(duration)) => {
                                        let hours = duration.as_secs() / 3600;
                                        let minutes = (duration.as_secs() % 3600) / 60;
                                        let details = if hours > 0 {
                                            format!("{hours}h")
                                        } else {
                                            format!("{minutes}m")
                                        };
                                        ("Interval", "bg-blue-100 text-blue-800", details)
                                    },
                                    Schedule::Cron(CronSchedule(schedule)) => {
                                        ("Cron", "bg-green-100 text-green-800", schedule.to_string())
                                    }
                                }
                            } else {
                                ("Unknown", "bg-gray-100 text-gray-800", String::new())
                            };
                            @let next_run = job_config.map(|config| config.schedule.next_run(Some(&cron.last_run_at), Utc::now(), timezone));

                            tr {
                                td class="px-4 py-2 border" { (cron.name) }
                                td class="px-4 py-2 border" {
                                    span class={"px-2 py-1 text-xs font-semibold rounded " (job_type_class)} {
                                        (job_type_label)
                                    }
                                    @if !schedule_details.is_empty() {
                                        span class="ml-2 text-sm text-gray-600" {
                                            "(" (schedule_details) ")"
                                        }
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
                                    div class="flex gap-2 justify-center" {
                                        form action="/admin/crons/run" method="post" class="inline" {
                                            input type="hidden" name="cron_name" value=(cron.name);
                                            input type="submit" value="Run Now" class="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600 cursor-pointer";
                                        }
                                        form action="/admin/crons/reset" method="post" class="inline" {
                                            input type="hidden" name="cron_id" value=(cron.cron_id.to_string());
                                            input type="submit" value="Reset" class="px-3 py-1 bg-yellow-500 text-white rounded hover:bg-yellow-600 cursor-pointer";
                                        }
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

#[derive(Deserialize)]
pub(crate) struct RunCronForm {
    cron_name: String,
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

pub(crate) async fn run_cron(
    _admin: AdminUser,
    State(app_state): State<AppState>,
    Form(form): Form<RunCronForm>,
) -> Result<impl IntoResponse, ServerError> {
    let registry = crate::cron::cron_registry();
    let job_config = registry.get(&form.cron_name);

    if let Some(job_config) = job_config {
        job_config
            .run(app_state, "From Admin Dashboard".to_string())
            .await
            .map_err(|e| cja::color_eyre::eyre::eyre!("Error running cron job: {}", e))?;
    } else {
        return Err(cja::color_eyre::eyre::eyre!("Unknown cron job: {}", form.cron_name).into());
    }

    // Redirect back to the crons list
    Ok(axum::response::Redirect::to("/admin/crons"))
}
