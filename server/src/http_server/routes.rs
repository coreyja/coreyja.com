use axum::{body::Body, Json};
use jsonwebtoken::{Algorithm, Validation};
use posts::blog::BlogPosts;
use serde_json::json;

use super::*;

pub(crate) fn make_router(syntax_css: String) -> Router<AppState> {
    Router::new()
        .route("/_", get(pages::admin::versions))
        .route("/static/*path", get(static_assets))
        .route("/styles/syntax.css", get(|| async move { syntax_css }))
        .route("/styles/tailwind.css", get(|| async { TAILWIND_STYLES }))
        .route(
            "/styles/comic_code.css",
            get(|| async { COMIC_CODE_STYLES }),
        )
        .route("/", get(pages::home::home_page))
        .route("/posts/rss.xml", get(pages::blog::rss_feed))
        .route("/rss.xml", get(pages::blog::full_rss_feed))
        .route("/posts", get(pages::blog::posts_index))
        .route(
            "/posts/weekly/",
            // I accidently published by first newsletter under this path
            // so I'm redirecting it to the newsletter home page. I'll
            // update the few links I made outside this blog to the correct link
            get(|| async { Redirect::permanent("/newsletter") }),
        )
        .route("/posts/*key", get(pages::blog::post_get))
        .route("/til", get(pages::til::til_index))
        .route("/til/rss.xml", get(pages::til::rss_feed))
        .route("/til/:slug", get(pages::til::til_get))
        .route("/streams", get(pages::streams::streams_index))
        .route("/streams/:date", get(pages::streams::stream_get))
        .route("/projects", get(pages::projects::projects_index))
        .route("/projects/:slug", get(pages::projects::projects_get))
        .route("/tags/*tag", get(redirect_to_posts_index))
        .route("/year/*year", get(redirect_to_posts_index))
        .route("/newsletter", get(newsletter_get))
        .route("/auth/github_oauth", get(auth::routes::github_oauth))
        .route(
            "/login",
            get(|State(app_state): State<AppState>| async move {
                Redirect::temporary(&format!(
                    "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}",
                    app_state.github.client_id,
                    app_state.app.app_url("/auth/github_oauth")
                ))
            }),
        )
        .route(
            "/login/:from_app",
            get(
                |State(app_state): State<AppState>, Path(from_app): Path<String>| async move {
                    let state = sqlx::query!(
                        r#"
                        INSERT INTO GithubLoginStates (github_login_state_id, app, state)
                        VALUES ($1, $2, 'created')
                        RETURNING *
                        "#,
                        uuid::Uuid::new_v4(),
                        from_app,
                    ).fetch_one(&app_state.db).await.unwrap();

                    Redirect::temporary(&format!(
                        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&state={}",
                        app_state.github.client_id,
                        app_state.app.app_url("/auth/github_oauth"),
                        state.github_login_state_id
                    ))
                },
            ),
        )
        .route(
            "/login/:project_slug/claim",
            post(
                |State(app_state): State<AppState>, Path(project_slug): Path<String>, Json(body): Json<ClaimBody>| async move {
                    let projects = app_state.projects.clone();
                    let project = projects.projects.iter().find(|p| p.slug().unwrap() == project_slug).unwrap();
                    let auth_public_key = project.frontmatter.auth_public_key.as_ref().unwrap();

                    let jwt = body.jwt;
                    let jwt = jsonwebtoken::decode::<JWTClaim>(
                        &jwt,
                        &jsonwebtoken::DecodingKey::from_rsa_pem(
                            auth_public_key.as_bytes(),
                        ).unwrap(),
                        &Validation::new(Algorithm::RS256),
                    ).unwrap();

                    let github_login_state_id = jwt.claims.sub.parse::<uuid::Uuid>().unwrap();
                    let state = sqlx::query!(
                        r#"
                        SELECT state, Users.user_id
                        FROM GithubLoginStates
                        JOIN GithubLinks using (github_link_id)
                        JOIN Users using (user_id)
                        WHERE github_login_state_id = $1 and state = 'github_completed'
                        "#,
                        github_login_state_id
                    ).fetch_one(&app_state.db).await.unwrap();

                    assert_eq!(state.state, "github_completed");

                    sqlx::query!(
                        r#"
                        UPDATE GithubLoginStates
                        SET state = $1
                        WHERE github_login_state_id = $2
                        RETURNING *
                        "#,
                        "claimed",
                        github_login_state_id
                    ).fetch_one(&app_state.db).await.unwrap();

                    Json(json!({
                        "user_id": state.user_id,
                    }))
                },
            ),
        )
        .fallback(fallback)
}

#[derive(Debug, serde::Deserialize)]
struct ClaimBody {
    jwt: String,
}

#[derive(Debug, serde::Deserialize)]
struct JWTClaim {
    sub: String,
}

async fn redirect_to_posts_index() -> impl IntoResponse {
    Redirect::permanent("/posts")
}

async fn fallback(uri: Uri, State(posts): State<Arc<BlogPosts>>) -> Response {
    let path = uri.path();
    let decoded = urlencoding::decode(path).unwrap();
    let key = decoded.as_ref();
    let key = key.strip_prefix('/').unwrap_or(key);
    let key = key.strip_suffix('/').unwrap_or(key);

    let post = posts.posts().iter().find(|p| p.matches_path(key).is_some());

    match post {
        Some(post) => {
            Redirect::permanent(&format!("/posts/{}", post.path.canonical_path())).into_response()
        }
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

async fn static_assets(Path(p): Path<String>) -> ResponseResult {
    let path = p.strip_prefix('/').unwrap_or(&p);
    let path = path.strip_suffix('/').unwrap_or(path);

    let entry = STATIC_ASSETS.get_file(path);

    let Some(entry) = entry else {
        return Ok((
            axum::http::StatusCode::NOT_FOUND,
            format!("Static asset {} not found", path),
        )
            .into_response());
    };

    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        mime.to_string().parse().unwrap(),
    );

    Ok((headers, entry.contents()).into_response())
}

async fn newsletter_get(State(posts): State<Arc<BlogPosts>>) -> ResponseResult {
    let newsletters = posts
        .by_recency()
        .into_iter()
        .filter(|p| p.frontmatter.is_newsletter)
        .collect::<Vec<_>>();

    Ok((
        axum::http::StatusCode::OK,
        templates::newsletter::newsletter_page(newsletters),
    )
        .into_response())
}
