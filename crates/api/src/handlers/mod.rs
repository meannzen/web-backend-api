use axum::{Router, routing::get};

use crate::handlers::health::{health_live, health_ready};
use crate::state::AppState;
mod health;

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", api_router())
        .route("/health", get(health_live))
        .route("/health_ready", get(health_ready))
        .with_state(state)
}

fn api_router() -> Router<AppState> {
    Router::new().route("/test", get(async || "<html><div>test</div></html>"))
}
