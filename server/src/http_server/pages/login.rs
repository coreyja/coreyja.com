use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Json,
};
use jsonwebtoken::{Algorithm, Validation};

use crate::{
    http_server::{auth::session::is_admin_user, ResponseResult},
    state::AppState,
};
use cja::color_eyre;

pub(crate) fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(login))
        .route("/{from_app}", get(app_login).post(app_claim))
}

async fn login(
    State(app_state): State<AppState>,
    Query(queries): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let state = sqlx::query!(
        r#"
      INSERT INTO GithubLoginStates (github_login_state_id, state, return_to)
      VALUES ($1, $2, $3)
      RETURNING *
      "#,
        uuid::Uuid::new_v4(),
        "created",
        queries.get("return_to"),
    )
    .fetch_one(&app_state.db)
    .await?;

    ResponseResult::Ok(Redirect::temporary(&format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&state={}",
        app_state.github.client_id,
        app_state.app.app_url("/auth/github"),
        state.github_login_state_id
    )))
}

async fn app_login(
    State(app_state): State<AppState>,
    Path(from_app): Path<String>,
) -> impl IntoResponse {
    app_state
        .projects
        .projects
        .iter()
        .find(|p| p.slug().unwrap() == from_app)
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Project does not exist"))?;

    let state = sqlx::query!(
        r#"
      INSERT INTO GithubLoginStates (github_login_state_id, app, state)
      VALUES ($1, $2, 'created')
      RETURNING *
      "#,
        uuid::Uuid::new_v4(),
        from_app,
    )
    .fetch_one(&app_state.db)
    .await?;

    ResponseResult::Ok(Redirect::temporary(&format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&state={}",
        app_state.github.client_id,
        app_state.app.app_url("/auth/github"),
        state.github_login_state_id
    )))
}

#[derive(Debug, serde::Deserialize)]
struct ClaimBody {
    jwt: String,
}

#[derive(Debug, serde::Deserialize)]
struct JWTClaim {
    sub: String,
}

#[derive(Debug, serde::Serialize, Clone)]
struct ClaimResponse {
    user_id: uuid::Uuid,
    is_active_sponsor: bool,
    is_admin: bool,
}

async fn app_claim(
    State(app_state): State<AppState>,
    Path(project_slug): Path<String>,
    Json(body): Json<ClaimBody>,
) -> impl IntoResponse {
    let projects = app_state.projects.clone();
    let project = projects
        .projects
        .iter()
        .find(|p| p.slug().unwrap() == project_slug)
        .unwrap();
    let auth_public_key = project.frontmatter.auth_public_key.as_ref().unwrap();

    let jwt = body.jwt;
    let jwt = jsonwebtoken::decode::<JWTClaim>(
        &jwt,
        &jsonwebtoken::DecodingKey::from_rsa_pem(auth_public_key.as_bytes())?,
        &Validation::new(Algorithm::RS256),
    )?;

    let github_login_state_id = jwt.claims.sub.parse::<uuid::Uuid>()?;
    let state = sqlx::query!(
        r#"
       SELECT state, Users.user_id
       FROM GithubLoginStates
       JOIN GithubLinks using (github_link_id)
       JOIN Users using (user_id)
       WHERE github_login_state_id = $1
       "#,
        github_login_state_id
    )
    .fetch_one(&app_state.db)
    .await?;

    if state.state == "claimed" {
        return Err(cja::color_eyre::eyre::eyre!("This Login has already been claimed").into());
    }

    if state.state != "github_completed" {
        return Err(color_eyre::eyre::eyre!("This login is not in the correct state").into());
    }

    debug_assert_eq!(state.state, "github_completed");

    sqlx::query!(
        r#"
       UPDATE GithubLoginStates
       SET state = 'claimed'
       WHERE github_login_state_id = $1
       RETURNING *
       "#,
        github_login_state_id
    )
    .fetch_one(&app_state.db)
    .await?;

    let sponsor = sqlx::query!(
        r#"
        SELECT *
        FROM GithubSponsors
        WHERE user_id = $1
        "#,
        state.user_id
    )
    .fetch_optional(&app_state.db)
    .await?;

    let is_active_sponsor = sponsor.is_some_and(|s| s.is_active && !s.is_one_time_payment);
    let is_admin = is_admin_user(state.user_id, &app_state).await?;

    ResponseResult::Ok(Json(ClaimResponse {
        user_id: state.user_id,
        is_active_sponsor,
        is_admin,
    }))
}
