use axum::{extract::State, response::IntoResponse};
use maud::{html, Render};

use crate::state::AppState;

use super::{
    auth::session::AdminUser,
    errors::ServerError,
    templates::{base_constrained, header::OpenGraph},
};

pub(crate) mod auth;
pub(crate) mod crons;
pub(crate) mod job_routes;

pub(crate) async fn dashboard(
    admin: AdminUser,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let google_user = sqlx::query!(
        "
    SELECT *
    FROM GoogleUsers
    WHERE user_id = $1",
        admin.user_id
    )
    .fetch_optional(&app_state.db)
    .await?;

    let last_refresh_ats = sqlx::query!("SELECT * FROM LastRefreshAts")
        .fetch_all(&app_state.db)
        .await?;

    let youtube_refresh_job = sqlx::query!(
        "
            SELECT *
            FROM Jobs
            WHERE name = 'RefreshVideos'
            ORDER BY created_at DESC
            LIMIT 1
            "
    )
    .fetch_optional(&app_state.db)
    .await?;

    Ok(base_constrained(
        html! {
            h1 class="text-xl" { "Admin Dashboard" }

            div class="my-4" {
                a href="/admin/crons" class="text-blue-500 hover:underline" { "Manage Crons →" }
            }

            h3 class="py-2 text-lg" { "Last Refresh Ats" }
            @for last_refresh_at in last_refresh_ats {
                div class="my-2" {
                    h4 class="text-md" { (last_refresh_at.key) }
                    p { "Last Refreshed: " (Timestamp(last_refresh_at.last_refresh_at)) }
                }
            }

            h3 class="py-2 text-lg" { "Google Auth Status" }
            @if let Some(google_user) = google_user {
                p { "Local Google User ID: " (google_user.google_user_id) }
                p { "Google Email: " (google_user.external_google_email) }
                p { "External Google ID: " (google_user.external_google_id) }

                h5 class="py-2 text-lg" { "Youtube Videos" }
                @if let Some(job) = youtube_refresh_job {
                    p { "Refresh Job Enqueued At: " (job.created_at) }
                    p { "Refresh Job Run At: " (job.run_at) }

                    @if let Some((locked_at, locked_by)) = job.locked_at.zip(job.locked_by) {
                        p { "Refresh Job Locked At: " (locked_at) }
                        p { "Refresh Job Locked By: " (locked_by) }
                    }
                }

                form action="/admin/jobs/refresh_youtube" method="post" {
                    input type="submit" value="Refresh Youtube Videos";
                }
            } @else {
                p { "No Google User Found" }
                a href="/admin/auth/google" { "Login now" }
            }
        },
        OpenGraph::default(),
    ))
}

struct MaybeTimestamp<T: chrono::TimeZone>(Option<chrono::DateTime<T>>);
struct Timestamp<T: chrono::TimeZone>(chrono::DateTime<T>);

impl<T: chrono::TimeZone> Render for MaybeTimestamp<T> {
    fn render(&self) -> maud::Markup {
        if let Some(timestamp) = self.0.clone() {
            Timestamp(timestamp).render()
        } else {
            html! {
                span { "Never" }
            }
        }
    }
}

impl<T: chrono::TimeZone> Render for Timestamp<T> {
    fn render(&self) -> maud::Markup {
        let timestamp = self.0.clone();
        let now = chrono::Utc::now().with_timezone(&timestamp.timezone());
        let ago = chrono_humanize::HumanTime::from(timestamp.clone() - now);
        html! {
            span title=(format!("{} UTC", timestamp.to_rfc3339())) { (ago) }
        }
    }
}
