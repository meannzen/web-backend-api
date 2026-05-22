use axum::http::StatusCode;

use crate::common::TestApp;

#[tokio::test]
async fn health_live_returns_ok() {
    let app = TestApp::new();
    let response = app.get("/health").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_ready_returns_ok_when_db_up() {
    let app = TestApp::new_with_db_health(true);
    let response = app.get("/health/ready").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_ready_returns_503_when_db_down() {
    let app = TestApp::new_with_db_health(false);
    let response = app.get("/health/ready").await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}
