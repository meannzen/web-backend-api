use std::net::SocketAddr;

use api::routes::router;
use api::state::AppState;
use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, Response, header};
use http_body_util::BodyExt;
use infra::db::Database;
use secrecy::SecretString;
use shared::config::{ApplicationSettings, Config, DatabaseSettings, Environment};
use sqlx::PgPool;
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test-secret-key-for-testing-minimum-32-chars!!";

fn make_test_token() -> String {
    use jsonwebtoken::{EncodingKey, Header, encode};

    #[derive(serde::Serialize)]
    struct TestClaims {
        sub: String,
        exp: usize,
    }

    encode(
        &Header::default(),
        &TestClaims {
            sub: "00000000-0000-0000-0000-000000000001".to_string(),
            exp: 9_999_999_999,
        },
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .unwrap()
}

pub struct TestApp {
    router: Router,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        let config = Config {
            application: ApplicationSettings {
                host: "127.0.0.1".to_string(),
                port: 0,
                environment: Environment::Development,
                log_level: "error".to_string(),
                jwt_secret: Some(SecretString::from(TEST_JWT_SECRET)),
                cors_origins: vec![],
            },
            database: DatabaseSettings {
                username: String::new(),
                password: SecretString::from(String::new()),
                host: String::new(),
                port: 5432,
                database_name: String::new(),
                require_ssl: false,
                max_connections: 5,
            },
        };
        let db = Database::from_pool(pool);
        let state = AppState::new(config, db);
        TestApp { router: router(state) }
    }

    pub async fn get(&self, uri: &str) -> Response<Body> {
        let (mut parts, body) = Request::builder()
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {}", make_test_token()))
            .body(Body::empty())
            .unwrap()
            .into_parts();
        parts
            .extensions
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 0))));
        self.router
            .clone()
            .oneshot(Request::from_parts(parts, body))
            .await
            .unwrap()
    }

    pub async fn post<B: serde::Serialize>(&self, uri: &str, body: &B) -> Response<Body> {
        let (mut parts, body) = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {}", make_test_token()))
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap()
            .into_parts();
        parts
            .extensions
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 0))));
        self.router
            .clone()
            .oneshot(Request::from_parts(parts, body))
            .await
            .unwrap()
    }
}

pub async fn body_json(response: Response<Body>) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}
