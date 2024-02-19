#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> miette::Result<()> {
    use axum::extract::State;
    use axum::response::IntoResponse;
    use axum::Router;
    use http::Request;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_site::app::*;
    use leptos_site::fileserv::file_and_error_handler;
    use leptos_site::server::state::AppState;

    let app_state = AppState::from_env().await?;

    let leptos_options = app_state.leptos_options.clone();
    let addr = leptos_options.site_addr;

    // Disable query loading.
    leptos_query::suppress_query_load(true);
    // Introspect App Routes.
    let routes = generate_route_list(App);
    // Enable query loading.
    leptos_query::suppress_query_load(false);

    async fn server_fn_handler(
        State(app_state): State<AppState>,
        request: Request<axum::body::Body>,
    ) -> impl IntoResponse {
        leptos_axum::handle_server_fns_with_context(
            move || {
                // provide_context(session.clone());
                provide_context(app_state.clone());
            },
            request,
        )
        .await
    }

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/*fn_name",
            axum::routing::get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes(&app_state, routes, App)
        .fallback(file_and_error_handler)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
