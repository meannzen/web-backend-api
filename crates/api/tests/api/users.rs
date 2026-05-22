use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::common::{TestApp, body_json};

// --- create user ---

#[sqlx::test(migrations = "../../migrations")]
async fn create_user_returns_201(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .post(
            "/api/v1/users",
            &json!({ "email": "user@example.com", "password": "secret123" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["email"], "user@example.com");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
    assert!(body["password_hash"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_user_returns_409_on_duplicate_email(pool: PgPool) {
    let app = TestApp::new(pool);
    let payload = json!({ "email": "dup@example.com", "password": "secret123" });

    app.post("/api/v1/users", &payload).await;
    let response = app.post("/api/v1/users", &payload).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_user_returns_400_on_invalid_email(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .post(
            "/api/v1/users",
            &json!({ "email": "not-an-email", "password": "secret123" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_json(response).await;
    assert!(body["error"]["fields"]["email"].is_array());
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_user_returns_400_on_short_password(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .post(
            "/api/v1/users",
            &json!({ "email": "user@example.com", "password": "short" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_json(response).await;
    assert!(body["error"]["fields"]["password"].is_array());
}

// --- get user ---

#[sqlx::test(migrations = "../../migrations")]
async fn get_user_returns_200(pool: PgPool) {
    let app = TestApp::new(pool);

    let created = body_json(
        app.post(
            "/api/v1/users",
            &json!({ "email": "get@example.com", "password": "secret123" }),
        )
        .await,
    )
    .await;

    let id = created["id"].as_str().unwrap();
    let response = app.get(&format!("/api/v1/users/{}", id)).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["id"], id);
    assert_eq!(body["email"], "get@example.com");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_user_returns_404_for_unknown_id(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .get("/api/v1/users/00000000-0000-0000-0000-000000000000")
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
