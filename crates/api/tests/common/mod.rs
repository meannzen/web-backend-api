use api::handlers::router;
use api::state::AppState;
use axum::body::Body;
use axum::http::{Request, Response};
use infra::db::Database;
use secrecy::SecretString;
use shared::config::{Config, DatabaseConfig, Environment, ServerConfig};
use sqlx::PgPool;
use tower::ServiceExt;

pub struct TestApp {
    router: axum::Router,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        let config = Config {
            server: ServerConfig {
                port: 0,
                environment: Environment::Development,
                log_level: "error".to_string(),
            },
            database: DatabaseConfig {
                url: SecretString::from("unused".to_string()),
                max_connections: 5,
            },
        };
        let db = Database::from_pool(pool);
        let state = AppState::new(config, db);
        TestApp {
            router: router(state),
        }
    }

    pub async fn request(&self, req: Request<Body>) -> Response<Body> {
        self.router.clone().oneshot(req).await.unwrap()
    }

    pub async fn get(&self, uri: &str) -> Response<Body> {
        self.request(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
    }
}
