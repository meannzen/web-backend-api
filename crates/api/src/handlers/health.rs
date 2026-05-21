use axum::extract::State;
use axum::http::StatusCode;
use infra::db::Database;

pub async fn health_live() -> StatusCode {
    StatusCode::OK
}

pub async fn health_ready(State(db): State<Database>) -> StatusCode {
    if db.ping().await {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
