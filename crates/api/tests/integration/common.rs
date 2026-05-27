use api::routes::router;
use api::state::AppState;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, header};
use http_body_util::BodyExt;
use infra::db::Database;
use secrecy::SecretString;
use shared::config::{ApplicationSettings, Config, DatabaseSettings, Environment};
use sqlx::PgPool;
use tower::ServiceExt;

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
        self.router
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    pub async fn post<B: serde::Serialize>(&self, uri: &str, body: &B) -> Response<Body> {
        self.router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap()
    }
}

pub async fn body_json(response: Response<Body>) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}
