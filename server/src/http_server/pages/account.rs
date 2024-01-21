use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, State},
    response::IntoResponse,
};
use cja::server::session::DBSession;
use maud::html;

use crate::{
    github::sponsors::GithubSponsorFromDB,
    http_server::{
        current_user::{self, CurrentUser},
        templates::base_constrained,
        MietteError,
    },
    AppState,
};

pub(crate) async fn account_page(
    current_user: CurrentUser,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, MietteError> {
    Ok(base_constrained(
        html! {
          h1 class="text-2xl mb-4" { "Account" }
          h1 class="text-xl mb-2" { "Hey 👋" }

          p {
            "You are logged in with Github as " (current_user.github_link.external_github_login) "."
          }
        },
        Default::default(),
    ))
}

pub struct Sponsor {
    user: CurrentUser,
    sponsor: GithubSponsorFromDB,
}

#[async_trait]
impl FromRequestParts<AppState> for Sponsor {
    type Rejection = current_user::CurrentUserError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let current_user = CurrentUser::from_request_parts(parts, state).await?;

        let sponsor = sqlx::query_as!(
            GithubSponsorFromDB,
            r#"
            SELECT *
            FROM GithubSponsors
            WHERE user_id = $1
            "#,
            current_user.user.user_id
        )
        .fetch_one(&state.db)
        .await?;

        let sponsor = Sponsor {
            user: current_user,
            sponsor,
        };

        Ok(sponsor)
    }
}

pub(crate) async fn sponsorship_page(
    current_user: CurrentUser,
    sponsor: Option<Sponsor>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, MietteError> {
    Ok(base_constrained(
        html! {
          h1 class="text-xl mb-4" { "Hey "  (current_user.github_link.external_github_login) }

          @if let Some(sponsor) = sponsor {
            @if sponsor.sponsor.is_active {
              h2 class="text-lg" { "Thanks SO much for sponsoring my work!" }
            } @else {
              h2 class="text-lg" { "Thank you for sponsoring me in the past!" }
            }
          } @else {
            h2 class="text-lg mb-4" { "You aren't sponsoring my work right now" }

            p {
              "If you'd like to sponsor my work, you can do so on "
              a href="https://github.com/sponsors/coreyja" class="underline" { "Github Sponsors" }
            }

            p {
              "Sponsoring helps me continue to make vidoes and other content for everyone!"
            }
          }
        },
        Default::default(),
    ))
}
