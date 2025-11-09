use std::fmt::Display;

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
pub(crate) mod memories;
pub(crate) mod persona;
pub(crate) mod threads;
pub(crate) mod tool_suggestions;

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

    let linear_installation = sqlx::query!(
        "
    SELECT *
    FROM linear_installations
    LIMIT 1"
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
                a href="/admin/crons" class="text-blue-500 hover:underline mr-4" { "Manage Crons →" }
                a href="/admin/threads" class="text-blue-500 hover:underline mr-4" { "Agentic Threads →" }
                a href="/admin/tool-suggestions" class="text-blue-500 hover:underline mr-4" { "Tool Suggestions →" }
                a href="/admin/persona" class="text-blue-500 hover:underline mr-4" { "Persona →" }
                a href="/admin/memories" class="text-blue-500 hover:underline" { "Memory Blocks →" }
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

            h3 class="py-2 text-lg" { "Linear Integration" }
            @if let Some(installation) = linear_installation {
                p { "Linear Workspace ID: " (installation.external_workspace_id) }
                @if let Some(actor_id) = &installation.external_actor_id {
                    p { "Linear Actor ID: " (actor_id) }
                }
                p { "Installation Created: " (Timestamp(installation.created_at)) }
                p { "Last Updated: " (Timestamp(installation.updated_at)) }
                a href="/api/linear/auth" class="text-blue-500 hover:underline" { "Re-authenticate Linear" }
            } @else {
                p { "No Linear installation found" }
                a href="/api/linear/auth" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded inline-block" {
                    "Connect Linear Agent"
                }
            }
        },
        OpenGraph::default(),
    ))
}

struct MaybeTimestamp<T: chrono::TimeZone>(Option<chrono::DateTime<T>>);
struct Timestamp<T: chrono::TimeZone>(chrono::DateTime<T>);

impl<T: chrono::TimeZone> Render for MaybeTimestamp<T>
where
    T: chrono::TimeZone,
    T::Offset: Display,
{
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

impl<T> Render for Timestamp<T>
where
    T: chrono::TimeZone,
    T::Offset: Display,
{
    fn render(&self) -> maud::Markup {
        let timestamp = self.0.clone();
        let now = chrono::Utc::now().with_timezone(&timestamp.timezone());
        let duration = timestamp.clone() - now;
        let human_time = chrono_humanize::HumanTime::from(duration);

        // Format timestamp with timezone
        let timestamp_with_tz = timestamp.format("%Y-%m-%d %H:%M:%S %Z").to_string();
        let utc_time = timestamp
            .with_timezone(&chrono::Utc)
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();
        let title = if timestamp_with_tz.contains("UTC") {
            timestamp_with_tz
        } else {
            format!("{timestamp_with_tz}\n{utc_time}")
        };

        html! {
            span title=(title) { (human_time) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use chrono_tz::US::Pacific;

    #[test]
    fn test_timestamp_in_past() {
        // Create a timestamp 2 hours ago
        let two_hours_ago = Utc::now() - Duration::hours(2);
        let timestamp = Timestamp(two_hours_ago);

        let rendered = timestamp.render().into_string();

        // Check that it contains "ago" for past timestamps
        assert!(
            rendered.contains("ago"),
            "Expected 'ago' in rendered output: {rendered}"
        );

        // Check that it has the title attribute with the full timestamp
        assert!(
            rendered.contains("title="),
            "Expected title attribute in rendered output"
        );
        assert!(
            rendered.contains("<span"),
            "Expected span element in rendered output"
        );
    }

    #[test]
    fn test_timestamp_in_future() {
        // Create a timestamp 3 hours in the future
        let three_hours_future = Utc::now() + Duration::hours(3);
        let timestamp = Timestamp(three_hours_future);

        let rendered = timestamp.render().into_string();

        // Check that it contains "in" for future timestamps
        assert!(
            rendered.contains("in "),
            "Expected 'in' for future timestamp: {rendered}"
        );

        // Check structure
        assert!(
            rendered.contains("title="),
            "Expected title attribute in rendered output"
        );
        assert!(
            rendered.contains("<span"),
            "Expected span element in rendered output"
        );
    }

    #[test]
    fn test_timestamp_different_timezone() {
        // Create a timestamp in Pacific timezone
        let pacific_time = Pacific.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let timestamp = Timestamp(pacific_time);

        let rendered = timestamp.render().into_string();

        // Check that the title contains both timezone representations
        assert!(
            rendered.contains("PST"),
            "Expected PST timezone in title: {rendered}"
        );
        assert!(
            rendered.contains("UTC"),
            "Expected UTC time in title for non-UTC timestamp"
        );
        assert!(
            rendered.contains("title="),
            "Expected title attribute in rendered output"
        );
    }

    #[test]
    fn test_timestamp_utc_timezone() {
        // Create a timestamp in UTC
        let utc_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let timestamp = Timestamp(utc_time);

        let rendered = timestamp.render().into_string();

        // For UTC timestamps, title should only show UTC time once
        let title_count = rendered.matches("UTC").count();
        assert_eq!(
            title_count, 1,
            "Expected UTC to appear only once in title for UTC timestamp"
        );
    }

    #[test]
    fn test_maybe_timestamp_with_none() {
        let maybe_timestamp: MaybeTimestamp<Utc> = MaybeTimestamp(None);

        let rendered = maybe_timestamp.render().into_string();

        // Should render "Never" for None values
        assert!(
            rendered.contains("Never"),
            "Expected 'Never' for None timestamp"
        );
        assert!(
            rendered.contains("<span"),
            "Expected span element in rendered output"
        );
    }

    #[test]
    fn test_maybe_timestamp_with_some() {
        let past_time = Utc::now() - Duration::days(1);
        let maybe_timestamp = MaybeTimestamp(Some(past_time));

        let rendered = maybe_timestamp.render().into_string();

        // Should render like a normal timestamp
        assert!(
            rendered.contains("ago"),
            "Expected 'ago' for past timestamp"
        );
        assert!(
            rendered.contains("title="),
            "Expected title attribute for Some timestamp"
        );
    }
}
