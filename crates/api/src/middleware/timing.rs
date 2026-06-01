use axum::{extract::Request, middleware::Next, response::Response};

pub async fn timing_middleware(req: Request, next: Next) -> Response {
    let start = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    let response = next.run(req).await;

    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration_ms = %start.elapsed().as_millis(),
        "request completed"
    );

    response
}
