use axum::{
    extract::FromRequestParts,
    response::{IntoResponse, Redirect},
};
use maud::html;

use crate::{
    http_server::{
        auth::session::{AdminUser, DBSession},
        errors::MietteError,
        templates::base_constrained,
    },
    state::AppState,
};

pub enum CourseAccess {
    Admin(AdminUser),
    GithubSponsor(db::GithubSponsor),
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for CourseAccess {
    type Rejection = axum::response::Redirect;

    async fn from_request_parts<'life0, 'life1>(
        parts: &'life0 mut axum::http::request::Parts,
        state: &'life1 AppState,
    ) -> Result<Self, Self::Rejection> {
        let admin_user: Option<AdminUser> = FromRequestParts::from_request_parts(parts, state)
            .await
            .ok();

        if let Some(admin_user) = admin_user {
            return Ok(Self::Admin(admin_user));
        }

        let db_session = DBSession::from_request_parts(parts, state).await?;

        let sponsor = sqlx::query_as!(
            db::GithubSponsor,
            "
            SELECT GithubSponsors.*
            FROM GithubSponsors
            JOIN GithubLinks on GithubLinks.external_github_id = GithubSponsors.github_id
            WHERE GithubLinks.user_id = $1
            ",
            db_session.user_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            Redirect::temporary("/login")
        })?;

        if let Some(sponsor) = sponsor {
            return Ok(Self::GithubSponsor(sponsor));
        }

        return Err(Redirect::temporary("/login"));
    }
}

pub async fn get(access: Option<CourseAccess>) -> Result<impl IntoResponse, MietteError> {
    Ok(base_constrained(
        html! {
          div class="max-w-prose" {
            h1 class="text-xl mb-8" { "Writing a Web Framework in Rust" }

            p class="mb-2" { "This course guides you through building your very own Web Framework in Rust!" }

            p class="mb-2" { "We'll start with only what the Rust standard library provides, and build from a TcpClient all the way up to a working HTTP server." }
            p class="mb-2" { "Each chapter includes a video walkthrough, as well as the full source code for that chapter." }

            @if let Some(access) = access {
              p class="mb-2" { "You have access! Thanks for supporting me!" }

              p class="mb-2" {
                    "You have access because: "
                    @match access {
                        CourseAccess::Admin(_) => { "You are an admin!" }
                        CourseAccess::GithubSponsor(_) => { "You are a Github Sponsor!" }
                    }
                }
            } @else {
              p class="mb-2" { "You do not have access!" }
            }
          }
        },
        Default::default(),
    ))
}
