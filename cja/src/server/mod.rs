use axum::{extract::Request, response::Response, serve::IncomingStream};
use color_eyre::eyre::WrapErr;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_service::Service;

pub mod cookies;
pub mod session;

pub mod trace;

pub async fn run_server<AS: Clone + Sync + Send, S>(
    routes: axum::Router<AS>,
) -> color_eyre::Result<()>
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

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = port.parse()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // Check if we're being run under systemfd (LISTEN_FD will be set)
    let listener = if let Ok(listener_env) = std::env::var("LISTEN_FD") {
        // Use the socket2 crate for socket reuse
        let builder = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;

        // Set reuse options
        builder.set_reuse_address(true)?;
        #[cfg(unix)]
        builder.set_reuse_port(true)?;
        
        // Bind to the address
        let socket_addr = addr.into();
        builder.bind(&socket_addr)?;
        builder.listen(1024)?;

        tracing::info!("Zero-downtime reloading enabled (LISTEN_FD={})", listener_env);
        tracing::info!("Using reusable socket on port {}", port);
        
        // Convert to a TcpListener
        let std_listener = builder.into();
        TcpListener::from_std(std_listener)?
    } else {
        // Otherwise, create our own listener
        tracing::info!("Starting server on port {}", port);
        TcpListener::bind(&addr)
            .await
            .wrap_err("Failed to open port")?
    };
    
    let addr = listener.local_addr()?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .await
        .wrap_err("Failed to run server")
}
