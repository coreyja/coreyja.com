use axum::{extract::Request, response::Response, serve::IncomingStream};
use miette::{Context, IntoDiagnostic};
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_service::Service;

pub mod cookies;
pub mod session;

pub mod trace;

pub async fn run_server<AS: Clone + Sync + Send, S>(routes: axum::Router<AS>) -> miette::Result<()>
where
    axum::Router<AS>:
        for<'a> Service<IncomingStream<'a>, Error = Infallible, Response = S> + Send + 'static,
    for<'a> <axum::Router<AS> as Service<IncomingStream<'a>>>::Future: Send,
    S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
{
    let tracer = trace::Tracer;
    let trace_layer = tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(tracer)
        .on_response(tracer);

    let app = routes.layer(trace_layer).layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", addr);

    axum::serve(listener, app)
        .await
        .into_diagnostic()
        .wrap_err("Failed to run server")
}
