use api::handlers::router;
use api::state::AppState;
use axum::body::Body;
use axum::http::{Request, Response};
use infra::db::Database;
use secrecy::SecretString;
use shared::config::{ApplicationSettings, Config, DatabaseSettings, Environment};
use sqlx::PgPool;
use tower::ServiceExt;

pub struct TestApp {
    router: axum::Router,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        let db = Database::from_pool(pool);
        Self::build(db)
    }

    pub fn new_without_db() -> Self {
        let db = Database::from_pool(sqlx::PgPool::connect_lazy("postgres://unused").unwrap());
        Self::build(db)
    }

    fn build(db: Database) -> Self {
        let config = Config {
            application: ApplicationSettings {
                host: "127.0.0.1".to_string(),
                port: 0,
                environment: Environment::Development,
                log_level: "error".to_string(),
            },
            database: DatabaseSettings {
                username: "unused".to_string(),
                password: SecretString::from("unused".to_string()),
                host: "unused".to_string(),
                port: 5432,
                database_name: "unused".to_string(),
                require_ssl: false,
                max_connections: 5,
            },
        };
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
