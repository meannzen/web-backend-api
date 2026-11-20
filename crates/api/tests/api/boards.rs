use axum::http::StatusCode;
use serde_json::json;

use crate::common::{TestApp, body_json};

// --- authorization ---

#[tokio::test]
async fn list_boards_non_admin_returns_403() {
    let app = TestApp::new();
    let response = app.get("/api/v1/boards").await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_board_non_admin_returns_403() {
    let app = TestApp::new();
    let response = app
        .post(
            "/api/v1/boards",
            &json!({ "name": "Toronto DSB", "code": "TDSB" }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_board_non_admin_returns_403() {
    let app = TestApp::new();
    let response = app
        .get("/api/v1/boards/00000000-0000-0000-0000-000000000001")
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_boards_without_token_returns_401() {
    let app = TestApp::new();
    let response = app.get_no_auth("/api/v1/boards").await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// --- create board ---

#[tokio::test]
async fn create_board_returns_201() {
    let app = TestApp::new();

    let response = app
        .post_as_admin(
            "/api/v1/boards",
            &json!({ "name": "Toronto DSB", "code": "tdsb" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["name"], "Toronto DSB");
    assert_eq!(body["code"], "TDSB");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn create_board_returns_400_on_invalid_code_characters() {
    let app = TestApp::new();

    let response = app
        .post_as_admin(
            "/api/v1/boards",
            &json!({ "name": "Toronto DSB", "code": "TD SB" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_board_returns_400_on_blank_name() {
    let app = TestApp::new();

    let response = app
        .post_as_admin("/api/v1/boards", &json!({ "name": "   ", "code": "TDSB" }))
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// --- get / list ---

#[tokio::test]
async fn get_board_returns_404_for_unknown_id() {
    let app = TestApp::new();
    let response = app
        .get_as_admin("/api/v1/boards/00000000-0000-0000-0000-000000000000")
        .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_boards_returns_400_on_invalid_sort_by() {
    let app = TestApp::new();
    let response = app.get_as_admin("/api/v1/boards?sort_by=email").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
