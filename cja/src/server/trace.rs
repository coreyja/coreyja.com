use tower_http::trace::{MakeSpan, OnResponse};
use tracing::Level;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Tracer;

impl<Body> MakeSpan<Body> for Tracer {
    fn make_span(&mut self, request: &http::Request<Body>) -> tracing::Span {
        let route = http_route(request);
        let span_name = format!("{} {}", request.method(), route);

        tracing::span!(
            Level::INFO,
            "server.request",
            otel.name = span_name,
            kind = "server",
            uri = %request.uri(),
            url.path = %request.uri().path(),
            url.query = request.uri().query(),
            url.scheme = request.uri().scheme_str(),
            server.address = request.uri().host(),
            server.port = request.uri().port_u16(),
            http_version = ?request.version(),
            user_agent.original = request.headers().get("user-agent").and_then(|h| h.to_str().ok()),
            http.route = route,
            http.request.method = %request.method(),
            http.request.header.host = request.headers().get("host").and_then(|h| h.to_str().ok()),
            http.request.header.forwarded_for = request.headers().get("x-forwarded-for").and_then(|h| h.to_str().ok()),
            http.request.header.forwarded_proto = request.headers().get("x-forwarded-proto").and_then(|h| h.to_str().ok()),
            http.request.header.host = request.headers().get("x-forwarded-ssl").and_then(|h| h.to_str().ok()),
            http.request.header.referer = request.headers().get("referer").and_then(|h| h.to_str().ok()),
            http.request.header.fly_forwarded_port = request.headers().get("fly-forwarded-port").and_then(|h| h.to_str().ok()),
            http.request.header.fly_region = request.headers().get("fly-region").and_then(|h| h.to_str().ok()),
            http.request.header.via = request.headers().get("via").and_then(|h| h.to_str().ok()),

            http.response.status_code = tracing::field::Empty,
            http.response.header.content_type = tracing::field::Empty,
        )
    }
}

impl<Body> OnResponse<Body> for Tracer {
    fn on_response(
        self,
        response: &http::Response<Body>,
        latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        let status_code = response.status().as_u16();
        tracing::event!(
            Level::INFO,
            status = status_code,
            latency = format_args!("{} ms", latency.as_millis()),
            "finished processing request"
        );

        span.record("http.response.status_code", status_code);
        span.record(
            "http.response.header.content_type",
            response
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok()),
        );
    }
}

#[inline]
fn http_route<B>(req: &http::Request<B>) -> &str {
    req.extensions()
        .get::<axum::extract::MatchedPath>()
        .map_or_else(|| "", |mp| mp.as_str())
}
