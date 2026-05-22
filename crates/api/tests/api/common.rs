use std::sync::Arc;

use api::routes::router;
use api::state::AppState;
use axum::body::Body;
use axum::http::{Request, Response, header};
use infra::db::Database;
use infra::repositories::user_repository::PgUserRepository;
use secrecy::SecretString;
use shared::config::{ApplicationSettings, Config, DatabaseSettings, Environment};
use sqlx::PgPool;
use tower::ServiceExt;

pub struct TestApp {
    router: axum::Router,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
        let db = Database::from_pool(pool);
        Self::build(db, user_repo)
    }

    pub fn new_without_db() -> Self {
        let pool = sqlx::PgPool::connect_lazy("postgres://unused").unwrap();
        let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
        let db = Database::from_pool(pool);
        Self::build(db, user_repo)
    }

    fn build(db: Database, user_repo: Arc<PgUserRepository>) -> Self {
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
        let state = AppState::new(config, db, user_repo);
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

    pub async fn post<B: serde::Serialize>(&self, uri: &str, body: &B) -> Response<Body> {
        self.request(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
    }
}

pub async fn body_json(response: Response<Body>) -> serde_json::Value {
    use http_body_util::BodyExt;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}
