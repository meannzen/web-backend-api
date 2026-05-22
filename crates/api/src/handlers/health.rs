use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use domain::ports::health_indicator::HealthIndicator;

#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is alive"),
    )
)]
#[axum::debug_handler]
pub async fn health_live() -> StatusCode {
    StatusCode::OK
}

#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Service is ready to accept traffic"),
        (status = 503, description = "Database is unreachable"),
    )
)]
#[axum::debug_handler]
pub async fn health_ready(State(db): State<Arc<dyn HealthIndicator>>) -> StatusCode {
    if db.ping().await {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
