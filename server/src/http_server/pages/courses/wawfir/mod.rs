use axum::{
    extract::FromRequestParts,
    response::{IntoResponse, Redirect},
};
use maud::html;

use crate::{
    http_server::{
        auth::session::{AdminUser, DBSession},
        errors::MietteError,
        templates::{base, constrained_width},
    },
    state::AppState,
};
pub(crate) mod test;

#[derive(Debug, Clone)]
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

fn granted_access_banner(access: &CourseAccess) -> maud::Markup {
    html! {
        div class="bg-primary-500 px-6 py-2.5 sm:px-3.5" {
            (constrained_width(html! {
                p class="text-sm leading-6 text-background" {
                    strong class="font-semibold" { "Access Granted" }
                    svg viewBox="0 0 2 2" class="mx-2 inline h-0.5 w-0.5 fill-current" aria-hidden="true" {
                        circle cx="1" cy="1" r="1";
                    }
                    "You have access to this course because "

                    @match access {
                        CourseAccess::Admin(_) => { "you are an admin!" }
                        CourseAccess::GithubSponsor(_) => { "you are a Github Sponsor! Thanks so much!" }
                    }
                }
            }))
        }
    }
}

fn access_denied_banner() -> maud::Markup {
    html! {
        div class="bg-warning-200 px-6 py-2.5 sm:px-3.5" {
            (constrained_width(html! {
                p class="text-sm leading-6 text-background" {
                    strong class="font-semibold" { "Access Denied" }
                    svg viewBox="0 0 2 2" class="mx-2 inline h-0.5 w-0.5 fill-current" aria-hidden="true" {
                        circle cx="1" cy="1" r="1";
                    }
                    "You do not have access. Yet! If you would like pre-release access to this course please "
                    a class="underline" href="https://github.com/sponsors/coreyja" { "sponsor me on Github Sponsors"}
                }
            }))

        }
    }
}

fn access_banner(access: &Option<CourseAccess>) -> maud::Markup {
    let banner = match access {
        Some(access) => granted_access_banner(access),
        None => access_denied_banner(),
    };

    html! {
        div class="pb-4" {
            (banner)
        }
    }
}

pub async fn get(access: Option<CourseAccess>) -> Result<impl IntoResponse, MietteError> {
    Ok(base(
        html! {
            (access_banner(&access))
            (constrained_width(
                html! {
                    div class="max-w-prose" {
                        h1 class="text-xl mb-8" { "Writing a Web Framework in Rust" }

                        p class="mb-2" { "This course guides you through building your very own Web Framework in Rust!" }

                        p class="mb-2" { "We'll start with only what the Rust standard library provides, and build from a TcpClient all the way up to a working HTTP server." }
                        p class="mb-2" { "Each chapter includes a video walkthrough, as well as the full source code for that chapter." }
                    }
                }
            ))
        },
        Default::default(),
    ))
}