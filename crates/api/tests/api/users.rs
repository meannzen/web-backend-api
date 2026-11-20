use axum::http::StatusCode;
use serde_json::json;

use crate::common::{MockError, TestApp, body_json};

// --- authorization ---

#[tokio::test]
async fn list_users_non_admin_returns_403() {
    let app = TestApp::new();
    let response = app.get("/api/v1/users").await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_user_non_admin_returns_403() {
    let app = TestApp::new();
    let response = app
        .post(
            "/api/v1/users",
            &json!({ "email": "user@example.com", "password": "secret123", "first_name": "Test", "last_name": "User" }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_user_non_admin_returns_403() {
    let app = TestApp::new();
    let response = app
        .get("/api/v1/users/00000000-0000-0000-0000-000000000001")
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// --- create user ---

#[tokio::test]
async fn create_user_returns_201() {
    let app = TestApp::new();

    let response = app
        .post_as_admin(
            "/api/v1/users",
            &json!({ "email": "user@example.com", "password": "secret123", "first_name": "Test", "last_name": "User" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["email"], "user@example.com");
    assert_eq!(body["first_name"], "Test");
    assert_eq!(body["last_name"], "User");
    assert_eq!(body["role"], "teacher");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn create_user_returns_409_on_duplicate_email() {
    let app = TestApp::new();
    let payload = json!({ "email": "dup@example.com", "password": "secret123", "first_name": "Test", "last_name": "User" });

    app.post_as_admin("/api/v1/users", &payload).await;
    let response = app.post_as_admin("/api/v1/users", &payload).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn create_user_returns_400_on_invalid_email() {
    let app = TestApp::new();

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

#[tokio::test]
async fn create_user_returns_400_on_short_password() {
    let app = TestApp::new();

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

#[tokio::test]
async fn create_user_returns_500_on_internal_error() {
    let app = TestApp::new();
    app.user_repo.set_error(MockError::Internal);

    let response = app
        .post_as_admin(
            "/api/v1/users",
            &json!({ "email": "user@example.com", "password": "secret123", "first_name": "Test", "last_name": "User" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_json(response).await;
    assert_eq!(body["error"]["type"], "internal_error");
    assert_eq!(body["error"]["message"], "an internal error occurred");
}

// --- list users ---

#[tokio::test]
async fn list_users_returns_empty_list() {
    let app = TestApp::new();
    let response = app.get_as_admin("/api/v1/users").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"], serde_json::json!([]));
    assert_eq!(body["meta"]["has_next_page"], false);
    assert_eq!(body["meta"]["next_cursor"], serde_json::Value::Null);
}

#[tokio::test]
async fn list_users_returns_created_users() {
    let app = TestApp::new();

    app.post_as_admin("/api/v1/users", &json!({ "email": "a@example.com", "password": "secret123", "first_name": "Alice", "last_name": "A" }))
        .await;
    app.post_as_admin("/api/v1/users", &json!({ "email": "b@example.com", "password": "secret123", "first_name": "Bob", "last_name": "B" }))
        .await;

    let response = app.get_as_admin("/api/v1/users").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["has_next_page"], false);
}

#[tokio::test]
async fn list_users_cursor_pagination_traverses_all_pages() {
    let app = TestApp::new();

    for i in 0..5 {
        app.post_as_admin(
            "/api/v1/users",
            &json!({ "email": format!("user{}@example.com", i), "password": "secret123", "first_name": "Test", "last_name": "User" }),
        )
        .await;
    }

    let response = app.get_as_admin("/api/v1/users?limit=2").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["has_next_page"], true);
    let cursor = body["meta"]["next_cursor"].as_str().unwrap().to_string();

    let response = app
        .get_as_admin(&format!("/api/v1/users?limit=2&after={}", cursor))
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["has_next_page"], true);
    let cursor = body["meta"]["next_cursor"].as_str().unwrap().to_string();

    let response = app
        .get_as_admin(&format!("/api/v1/users?limit=2&after={}", cursor))
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["meta"]["has_next_page"], false);
    assert_eq!(body["meta"]["next_cursor"], serde_json::Value::Null);
}

#[tokio::test]
async fn list_users_returns_400_on_invalid_cursor() {
    let app = TestApp::new();

    let response = app
        .get_as_admin("/api/v1/users?after=not-a-valid-cursor!!!")
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_users_returns_500_on_internal_error() {
    let app = TestApp::new();
    app.user_repo.set_error(MockError::Internal);

    let response = app.get_as_admin("/api/v1/users").await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_json(response).await;
    assert_eq!(body["error"]["type"], "internal_error");
}

// --- search ---

#[tokio::test]
async fn list_users_search_filters_by_email() {
    let app = TestApp::new();

    app.post_as_admin("/api/v1/users", &json!({ "email": "alice@example.com", "password": "secret123", "first_name": "Alice", "last_name": "A" }))
        .await;
    app.post_as_admin("/api/v1/users", &json!({ "email": "bob@example.com", "password": "secret123", "first_name": "Bob", "last_name": "B" }))
        .await;
    app.post_as_admin("/api/v1/users", &json!({ "email": "charlie@other.org", "password": "secret123", "first_name": "Charlie", "last_name": "C" }))
        .await;

    let response = app.get_as_admin("/api/v1/users?search=example").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["has_next_page"], false);
}

#[tokio::test]
async fn list_users_search_returns_empty_for_no_match() {
    let app = TestApp::new();

    app.post_as_admin("/api/v1/users", &json!({ "email": "user@example.com", "password": "secret123", "first_name": "Test", "last_name": "User" }))
        .await;

    let response = app.get_as_admin("/api/v1/users?search=nobody").await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"], serde_json::json!([]));
}

// --- sort ---

#[tokio::test]
async fn list_users_sort_by_email_asc() {
    let app = TestApp::new();

    app.post_as_admin("/api/v1/users", &json!({ "email": "charlie@example.com", "password": "secret123", "first_name": "Charlie", "last_name": "C" }))
        .await;
    app.post_as_admin("/api/v1/users", &json!({ "email": "alice@example.com", "password": "secret123", "first_name": "Alice", "last_name": "A" }))
        .await;
    app.post_as_admin("/api/v1/users", &json!({ "email": "bob@example.com", "password": "secret123", "first_name": "Bob", "last_name": "B" }))
        .await;

    let response = app
        .get_as_admin("/api/v1/users?sort_by=email&sort_direction=asc")
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    let emails: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["email"].as_str().unwrap())
        .collect();

    assert_eq!(
        emails,
        vec![
            "alice@example.com",
            "bob@example.com",
            "charlie@example.com"
        ]
    );
}

#[tokio::test]
async fn list_users_sort_by_email_cursor_traverses_pages() {
    let app = TestApp::new();

    for email in [
        "alice@x.com",
        "bob@x.com",
        "charlie@x.com",
        "diana@x.com",
        "eve@x.com",
    ] {
        app.post_as_admin("/api/v1/users", &json!({ "email": email, "password": "secret123", "first_name": "Test", "last_name": "User" })).await;
    }

    let response = app
        .get_as_admin("/api/v1/users?sort_by=email&sort_direction=asc&limit=2")
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["has_next_page"], true);
    let cursor = body["meta"]["next_cursor"].as_str().unwrap().to_string();

    let response = app
        .get_as_admin(&format!(
            "/api/v1/users?sort_by=email&sort_direction=asc&limit=2&after={}",
            cursor
        ))
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn list_users_cursor_sort_mismatch_returns_400() {
    let app = TestApp::new();

    app.post_as_admin("/api/v1/users", &json!({ "email": "a@example.com", "password": "secret123", "first_name": "A", "last_name": "A" }))
        .await;
    app.post_as_admin("/api/v1/users", &json!({ "email": "b@example.com", "password": "secret123", "first_name": "B", "last_name": "B" }))
        .await;
    app.post_as_admin("/api/v1/users", &json!({ "email": "c@example.com", "password": "secret123", "first_name": "C", "last_name": "C" }))
        .await;

    let response = app
        .get_as_admin("/api/v1/users?sort_by=email&sort_direction=asc&limit=1")
        .await;
    let body = body_json(response).await;
    let cursor = body["meta"]["next_cursor"].as_str().unwrap().to_string();

    let response = app
        .get_as_admin(&format!(
            "/api/v1/users?sort_by=created_at&after={}",
            cursor
        ))
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// --- get user ---

#[tokio::test]
async fn get_user_returns_200() {
    let app = TestApp::new();

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

#[tokio::test]
async fn get_user_returns_404_for_unknown_id() {
    let app = TestApp::new();

    let response = app
        .get_as_admin("/api/v1/users/00000000-0000-0000-0000-000000000000")
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_user_returns_500_on_internal_error() {
    let app = TestApp::new();
    app.user_repo.set_error(MockError::Internal);

    let response = app
        .get_as_admin("/api/v1/users/00000000-0000-0000-0000-000000000000")
        .await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_json(response).await;
    assert_eq!(body["error"]["type"], "internal_error");
}

// --- role / created_at filters ---

async fn seed_two_users(app: &TestApp) {
    for (e, n) in [("a@example.com", "A"), ("b@example.com", "B")] {
        app.post_as_admin(
            "/api/v1/users",
            &json!({ "email": e, "password": "secret123", "first_name": n, "last_name": n }),
        )
        .await;
    }
}

async fn list_len(app: &TestApp, qs: &str) -> usize {
    let response = app.get_as_admin(&format!("/api/v1/users?{qs}")).await;
    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await["data"].as_array().unwrap().len()
}

#[tokio::test]
async fn list_users_filters_by_role() {
    let app = TestApp::new();
    seed_two_users(&app).await;

    assert_eq!(list_len(&app, "role=teacher").await, 2);
    assert_eq!(list_len(&app, "role=hr").await, 0);
}

#[tokio::test]
async fn list_users_invalid_role_returns_400() {
    let app = TestApp::new();
    let response = app.get_as_admin("/api/v1/users?role=wizard").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_users_filters_by_created_at_range() {
    let app = TestApp::new();
    seed_two_users(&app).await;

    assert_eq!(
        list_len(&app, "created_at_gt=2000-01-01T00:00:00Z").await,
        2
    );
    assert_eq!(
        list_len(&app, "created_at_gt=2999-01-01T00:00:00Z").await,
        0
    );
    assert_eq!(
        list_len(&app, "created_at_lt=2999-01-01T00:00:00Z").await,
        2
    );
    assert_eq!(
        list_len(&app, "created_at_lt=2000-01-01T00:00:00Z").await,
        0
    );
    assert_eq!(
        list_len(&app, "created_at_eq=2000-01-01T00:00:00Z").await,
        0
    );
}

#[tokio::test]
async fn list_users_invalid_created_at_returns_400() {
    let app = TestApp::new();
    let response = app
        .get_as_admin("/api/v1/users?created_at_gt=yesterday")
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_users_query_error_names_field_and_value() {
    let app = TestApp::new();

    for (qs, field, value) in [
        (
            "role=teacher&created_at_eq=2026-11-20T04",
            "created_at_eq",
            "2026-11-20T04",
        ),
        ("role=wizard", "role", "wizard"),
        ("limit=abc", "limit", "abc"),
    ] {
        let response = app.get_as_admin(&format!("/api/v1/users?{qs}")).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(
            msg.contains(&format!("`{field}`")) && msg.contains(value),
            "message should name field and value: {msg}"
        );
    }
}

#[tokio::test]
async fn list_users_query_error_includes_hint_and_truncates_value() {
    let app = TestApp::new();

    let response = app
        .get_as_admin("/api/v1/users?created_at_gt=2026-11-20T04")
        .await;
    let body = body_json(response).await;
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(msg.contains("RFC 3339"), "{msg}");

    let long = "x".repeat(500);
    let response = app
        .get_as_admin(&format!("/api/v1/users?role={long}"))
        .await;
    let body = body_json(response).await;
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(msg.len() < 300, "value should be truncated: {msg}");
    assert!(msg.contains("..."), "{msg}");
}
