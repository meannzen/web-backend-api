use axum::extract::State;
use axum::http::StatusCode;
use infra::db::Database;

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
pub async fn health_ready(State(db): State<Database>) -> StatusCode {
    if db.ping().await {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
