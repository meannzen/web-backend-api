mod common;

use axum::http::StatusCode;
use sqlx::PgPool;

#[tokio::test]
async fn health_live_returns_ok() {
    let app = common::TestApp::new_without_db();
    let response = app.get("/health").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn health_ready_returns_ok_when_db_up(pool: PgPool) {
    let app = common::TestApp::new(pool);
    let response = app.get("/health_ready").await;
    assert_eq!(response.status(), StatusCode::OK);
}
