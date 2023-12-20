use axum::{extract::State, response::IntoResponse};
use chrono::Utc;
use maud::{html, Render};
use miette::IntoDiagnostic;

use crate::state::AppState;

use super::{auth::session::AdminUser, errors::MietteError, templates::base_constrained};

pub(crate) mod auth;
pub(crate) mod job_routes;

pub(crate) async fn dashboard(
    admin: AdminUser,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, MietteError> {
    let google_user = sqlx::query!(
        "
    SELECT *
    FROM GoogleUsers
    WHERE user_id = $1",
        admin.session.user_id
    )
    .fetch_optional(&app_state.db)
    .await
    .into_diagnostic()?;

    let youtube_last_refresh_at =
        sqlx::query!("SELECT * FROM LastRefreshAts where key = 'youtube_videos'")
            .fetch_one(&app_state.db)
            .await
            .into_diagnostic()?
            .last_refresh_at;

    Ok(base_constrained(
        html! {
            h1 class="text-xl" { "Admin Dashboard" }

            h3 class="py-2 text-lg" { "Google Auth Status" }
            @if let Some(google_user) = google_user {
                p { "Local Google User ID: " (google_user.google_user_id) }
                p { "Google Email: " (google_user.external_google_email) }
                p { "External Google ID: " (google_user.external_google_id) }

                h5 class="py-2 text-lg" { "Youtube Videos" }
                p { "Last Refreshed: " (Timestamp(youtube_last_refresh_at)) }

                form action="/admin/jobs/refresh_youtube" method="post" {
                    input type="submit" value="Refresh Youtube Videos";
                }
            } @else {
                p { "No Google User Found" }
                a href="/admin/auth/google" { "Login now" }
            }
        },
        Default::default(),
    ))
}

struct Timestamp(chrono::DateTime<Utc>);

impl Render for Timestamp {
    fn render(&self) -> maud::Markup {
        let ago = chrono_humanize::HumanTime::from(self.0 - chrono::Utc::now());
        html! {
            span title=(format!("{} UTC", self.0.to_rfc3339())) { (ago) }
        }
    }
}
