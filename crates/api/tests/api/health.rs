use axum::http::StatusCode;
use sqlx::PgPool;

use crate::common::TestApp;

#[tokio::test]
async fn health_live_returns_ok() {
    let app = TestApp::new_without_db();
    let response = app.get("/health").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn health_ready_returns_ok_when_db_up(pool: PgPool) {
    let app = TestApp::new(pool);
    let response = app.get("/health/ready").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_ready_returns_503_when_db_down() {
    let app = TestApp::new_without_db();
    let response = app.get("/health/ready").await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}
