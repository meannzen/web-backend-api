mod common;

use axum::http::StatusCode;
use sqlx::PgPool;

#[sqlx::test]
async fn health_live_returns_ok(_pool: PgPool) {
    let app = common::TestApp::new(_pool);
    let response = app.get("/health").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn health_ready_returns_ok_when_db_up(pool: PgPool) {
    let app = common::TestApp::new(pool);
    let response = app.get("/health_ready").await;
    assert_eq!(response.status(), StatusCode::OK);
}
