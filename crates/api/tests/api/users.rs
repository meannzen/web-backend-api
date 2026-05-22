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
    assert!(body["updated_at"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_user_returns_409_on_duplicate_email(pool: PgPool) {
    let app = TestApp::new(pool);
    let payload = json!({ "email": "dup@example.com", "password": "secret123" });

    app.post("/api/v1/users", &payload).await;
    let response = app.post("/api/v1/users", &payload).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// validation tests do not touch the DB — validator fires before any repository call
#[tokio::test]
async fn create_user_returns_400_on_invalid_email() {
    let app = TestApp::new_without_db();

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

#[tokio::test]
async fn create_user_returns_400_on_short_password() {
    let app = TestApp::new_without_db();

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

// --- list users ---

#[sqlx::test(migrations = "../../migrations")]
async fn list_users_returns_empty_list(pool: PgPool) {
    let app = TestApp::new(pool);
    let response = app.get("/api/v1/users").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"], serde_json::json!([]));
    assert_eq!(body["meta"]["page"], 1);
    assert_eq!(body["meta"]["total"], 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_users_returns_created_users(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post("/api/v1/users", &json!({ "email": "a@example.com", "password": "secret123" }))
        .await;
    app.post("/api/v1/users", &json!({ "email": "b@example.com", "password": "secret123" }))
        .await;

    let response = app.get("/api/v1/users").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["meta"]["total"], 2);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_users_respects_pagination(pool: PgPool) {
    let app = TestApp::new(pool);

    for i in 0..5 {
        app.post(
            "/api/v1/users",
            &json!({ "email": format!("user{}@example.com", i), "password": "secret123" }),
        )
        .await;
    }

    let response = app.get("/api/v1/users?page=1&per_page=2").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["total"], 5);
    assert_eq!(body["meta"]["total_pages"], 3);
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
