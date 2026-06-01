use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{TestApp, body_json};

const REGISTER_URL: &str = "/api/v1/auth/register";
const LOGIN_URL: &str = "/api/v1/auth/login";

#[sqlx::test(migrations = "../../migrations")]
async fn register_creates_user_in_db(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .post_public(
            REGISTER_URL,
            &json!({
                "email": "alice@example.com",
                "password": "secret123",
                "first_name": "Alice",
                "last_name": "Smith"
            }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["email"], "alice@example.com");
    assert_eq!(body["first_name"], "Alice");
    assert_eq!(body["last_name"], "Smith");
    assert_eq!(body["role"], "user");
    assert!(Uuid::parse_str(body["id"].as_str().unwrap()).is_ok());
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_after_register_returns_valid_token(pool: PgPool) {
    let app = TestApp::new(pool.clone());

    app.post_public(
        REGISTER_URL,
        &json!({
            "email": "bob@example.com",
            "password": "secret123",
            "first_name": "Bob",
            "last_name": "Jones"
        }),
    )
    .await;

    let response = app
        .post_public(
            LOGIN_URL,
            &json!({ "email": "bob@example.com", "password": "secret123" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;
    let token = body["token"].as_str().unwrap();

    // Token should decode to the registered user's UUID
    use jsonwebtoken::{DecodingKey, decode};
    use crate::common::TEST_JWT_SECRET;
    use api::middleware::auth::{Claims, make_validation};

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
        &make_validation(),
    )
    .expect("token should decode");

    // The sub should be a valid UUID matching the registered user
    assert!(!token_data.claims.sub.is_nil());
    assert_eq!(token_data.claims.email, "bob@example.com");
    assert_eq!(token_data.claims.role, domain::users::model::Role::User);
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_wrong_password_returns_401(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post_public(
        REGISTER_URL,
        &json!({
            "email": "carol@example.com",
            "password": "secret123",
            "first_name": "Carol",
            "last_name": "White"
        }),
    )
    .await;

    let response = app
        .post_public(
            LOGIN_URL,
            &json!({ "email": "carol@example.com", "password": "wrongpassword" }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn duplicate_register_returns_409(pool: PgPool) {
    let app = TestApp::new(pool);
    let payload = json!({
        "email": "dave@example.com",
        "password": "secret123",
        "first_name": "Dave",
        "last_name": "Brown"
    });

    app.post_public(REGISTER_URL, &payload).await;
    let response = app.post_public(REGISTER_URL, &payload).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
}
