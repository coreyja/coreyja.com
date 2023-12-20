use axum::{extract::State, response::IntoResponse};
use maud::html;
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

    Ok(base_constrained(
        html! {
            h1 class="text-xl" { "Admin Dashboard" }

            h3 class="text-lg" { "Google Auth Status" }
            @if let Some(google_user) = google_user {
                p { "Local Google User ID: " (google_user.google_user_id) }
                p { "Google Email: " (google_user.external_google_email) }
                p { "External Google ID: " (google_user.external_google_id) }

                form action="/admin/jobs/refresh_youtube" method="post" {
                    input type="submit" value="Refresh";
                }
            } @else {
                p { "No Google User Found" }
                a href="/admin/auth/google" { "Login now" }
            }
        },
        Default::default(),
    ))
}
