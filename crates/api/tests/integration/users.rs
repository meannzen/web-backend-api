use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::common::{TestApp, body_json};

// --- create user ---

#[sqlx::test(migrations = "../../migrations")]
async fn create_user_returns_201(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .post_as_admin(
            "/api/v1/users",
            &json!({ "email": "alice@example.com", "password": "secret123", "first_name": "Alice", "last_name": "Smith" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["email"], "alice@example.com");
    assert_eq!(body["first_name"], "Alice");
    assert_eq!(body["last_name"], "Smith");
    assert_eq!(body["role"], "user");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_user_returns_409_on_duplicate_email(pool: PgPool) {
    let app = TestApp::new(pool);
    let payload = json!({ "email": "dup@example.com", "password": "secret123", "first_name": "Test", "last_name": "User" });

    app.post_as_admin("/api/v1/users", &payload).await;
    let response = app.post_as_admin("/api/v1/users", &payload).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_user_returns_400_on_invalid_email(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .post_as_admin(
            "/api/v1/users",
            &json!({ "email": "not-an-email", "password": "secret123", "first_name": "Test", "last_name": "User" }),
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
        .post_as_admin(
            "/api/v1/users",
            &json!({ "email": "user@example.com", "password": "short", "first_name": "Test", "last_name": "User" }),
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
        app.post_as_admin(
            "/api/v1/users",
            &json!({ "email": "get@example.com", "password": "secret123", "first_name": "Get", "last_name": "User" }),
        )
        .await,
    )
    .await;

    let id = created["id"].as_str().unwrap();
    let response = app.get_as_admin(&format!("/api/v1/users/{}", id)).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["id"], id);
    assert_eq!(body["email"], "get@example.com");
    assert_eq!(body["first_name"], "Get");
    assert_eq!(body["last_name"], "User");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_user_returns_404_for_unknown_id(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .get_as_admin("/api/v1/users/00000000-0000-0000-0000-000000000000")
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// --- list users ---

#[sqlx::test(migrations = "../../migrations")]
async fn list_users_returns_empty_list(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app.get_as_admin("/api/v1/users").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"], json!([]));
    assert_eq!(body["meta"]["has_next_page"], false);
    assert_eq!(body["meta"]["next_cursor"], serde_json::Value::Null);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_users_returns_created_users(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post_as_admin(
        "/api/v1/users",
        &json!({ "email": "a@example.com", "password": "secret123", "first_name": "A", "last_name": "A" }),
    )
    .await;
    app.post_as_admin(
        "/api/v1/users",
        &json!({ "email": "b@example.com", "password": "secret123", "first_name": "B", "last_name": "B" }),
    )
    .await;

    let response = app.get_as_admin("/api/v1/users").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["has_next_page"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_users_cursor_pagination_traverses_all_pages(pool: PgPool) {
    let app = TestApp::new(pool);

    for i in 0..5 {
        app.post_as_admin(
            "/api/v1/users",
            &json!({ "email": format!("user{}@example.com", i), "password": "secret123", "first_name": "Test", "last_name": "User" }),
        )
        .await;
    }

    let body = body_json(app.get_as_admin("/api/v1/users?limit=2").await).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["has_next_page"], true);
    let cursor = body["meta"]["next_cursor"].as_str().unwrap().to_string();

    let body = body_json(app.get_as_admin(&format!("/api/v1/users?limit=2&after={cursor}")).await).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["has_next_page"], true);
    let cursor = body["meta"]["next_cursor"].as_str().unwrap().to_string();

    let body = body_json(app.get_as_admin(&format!("/api/v1/users?limit=2&after={cursor}")).await).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["meta"]["has_next_page"], false);
    assert_eq!(body["meta"]["next_cursor"], serde_json::Value::Null);
}

// --- list with fixtures ---

#[sqlx::test(migrations = "../../migrations", fixtures("users"))]
async fn list_users_with_fixtures_returns_seeded_rows(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app.get_as_admin("/api/v1/users").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
}
