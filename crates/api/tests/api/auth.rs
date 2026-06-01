use axum::http::StatusCode;
use serde_json::json;

use crate::common::{TestApp, body_json};

const REGISTER_URL: &str = "/api/v1/auth/register";
const LOGIN_URL: &str = "/api/v1/auth/login";

fn valid_register() -> serde_json::Value {
    json!({
        "email": "alice@example.com",
        "password": "secret123",
        "first_name": "Alice",
        "last_name": "Smith"
    })
}

// --- register ---

#[tokio::test]
async fn register_returns_201_with_user_data() {
    let app = TestApp::new();

    let response = app.post_public(REGISTER_URL, &valid_register()).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["email"], "alice@example.com");
    assert_eq!(body["first_name"], "Alice");
    assert_eq!(body["last_name"], "Smith");
    assert_eq!(body["role"], "user");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
}

#[tokio::test]
async fn register_duplicate_email_returns_409() {
    let app = TestApp::new();

    app.post_public(REGISTER_URL, &valid_register()).await;
    let response = app.post_public(REGISTER_URL, &valid_register()).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn register_invalid_email_returns_400() {
    let app = TestApp::new();

    let response = app
        .post_public(
            REGISTER_URL,
            &json!({ "email": "not-an-email", "password": "secret123", "first_name": "Alice", "last_name": "Smith" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert!(body["error"]["fields"]["email"].is_array());
}

#[tokio::test]
async fn register_short_password_returns_400() {
    let app = TestApp::new();

    let response = app
        .post_public(
            REGISTER_URL,
            &json!({ "email": "alice@example.com", "password": "short", "first_name": "Alice", "last_name": "Smith" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert!(body["error"]["fields"]["password"].is_array());
}

#[tokio::test]
async fn register_empty_first_name_returns_400() {
    let app = TestApp::new();

    let response = app
        .post_public(
            REGISTER_URL,
            &json!({ "email": "alice@example.com", "password": "secret123", "first_name": "", "last_name": "Smith" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert!(body["error"]["fields"]["first_name"].is_array());
}

#[tokio::test]
async fn register_empty_last_name_returns_400() {
    let app = TestApp::new();

    let response = app
        .post_public(
            REGISTER_URL,
            &json!({ "email": "alice@example.com", "password": "secret123", "first_name": "Alice", "last_name": "" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert!(body["error"]["fields"]["last_name"].is_array());
}

#[tokio::test]
async fn register_name_too_long_returns_400() {
    let app = TestApp::new();
    let long_name = "A".repeat(101);

    let response = app
        .post_public(
            REGISTER_URL,
            &json!({ "email": "alice@example.com", "password": "secret123", "first_name": long_name, "last_name": "Smith" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert!(body["error"]["fields"]["first_name"].is_array());
}

// --- login ---

#[tokio::test]
async fn login_returns_200_with_token() {
    let app = TestApp::new();
    app.post_public(REGISTER_URL, &valid_register()).await;

    let response = app
        .post_public(
            LOGIN_URL,
            &json!({ "email": "alice@example.com", "password": "secret123" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert!(body["token"].is_string());
    assert!(!body["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn login_wrong_password_returns_401() {
    let app = TestApp::new();
    app.post_public(REGISTER_URL, &valid_register()).await;

    let response = app
        .post_public(
            LOGIN_URL,
            &json!({ "email": "alice@example.com", "password": "wrongpassword" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_unknown_email_returns_401() {
    let app = TestApp::new();

    let response = app
        .post_public(
            LOGIN_URL,
            &json!({ "email": "nobody@example.com", "password": "secret123" }),
        )
        .await;

    // Same 401 as wrong password — no email enumeration
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_missing_email_returns_400() {
    let app = TestApp::new();

    let response = app
        .post_public(LOGIN_URL, &json!({ "password": "secret123" }))
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn login_missing_password_returns_400() {
    let app = TestApp::new();

    let response = app
        .post_public(LOGIN_URL, &json!({ "email": "alice@example.com" }))
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// --- auth token / protected routes ---

#[tokio::test]
async fn valid_admin_token_allows_access_to_protected_route() {
    let app = TestApp::new();

    let response = app.get_as_admin("/api/v1/users").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn missing_token_returns_401() {
    let app = TestApp::new();

    let response = app.get_no_auth("/api/v1/users").await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn invalid_token_returns_401() {
    let app = TestApp::new();

    let response = app.get_with_token("/api/v1/users", "not-a-valid-jwt-token").await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_secret_token_returns_401() {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

    #[derive(serde::Serialize)]
    struct TestClaims {
        sub: String,
        email: String,
        role: String,
        iss: String,
        aud: String,
        exp: usize,
        iat: usize,
    }

    let wrong_token = encode(
        &Header::new(Algorithm::HS256),
        &TestClaims {
            sub: "00000000-0000-0000-0000-000000000001".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            iss: "tpa".to_string(),
            aud: "tpa-api".to_string(),
            exp: 9_999_999_999,
            iat: 0,
        },
        &EncodingKey::from_secret(b"completely-wrong-secret-key-here!"),
    )
    .unwrap();

    let app = TestApp::new();
    let response = app.get_with_token("/api/v1/users", &wrong_token).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// --- full register → login → access flow ---

#[tokio::test]
async fn register_then_login_returns_valid_jwt() {
    let app = TestApp::new();

    // Register
    let reg_response = app.post_public(REGISTER_URL, &valid_register()).await;
    assert_eq!(reg_response.status(), StatusCode::CREATED);

    // Login
    let login_response = app
        .post_public(
            LOGIN_URL,
            &json!({ "email": "alice@example.com", "password": "secret123" }),
        )
        .await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let login_body = body_json(login_response).await;
    let token = login_body["token"].as_str().unwrap();
    assert!(!token.is_empty());

    // The token is a valid JWT (auth passes, not 401), but registered users are
    // role:user so they are forbidden from admin-only routes (403, not 401).
    let response = app.get_with_token("/api/v1/users", token).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
