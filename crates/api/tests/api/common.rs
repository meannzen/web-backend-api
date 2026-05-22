use std::sync::{Arc, Mutex};

use api::routes::router;
use api::state::AppState;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, Response, header};
use chrono::Utc;
use domain::users::errors::UserError;
use domain::users::model::{Email, NewUser, User, UserId};
use domain::users::port::UserRepository;
use domain::users::service::UserService;
use infra::db::Database;
use infra::security::password_hasher::Argon2PasswordHasher;
use secrecy::SecretString;
use shared::config::{ApplicationSettings, Config, DatabaseSettings, Environment};
use tower::ServiceExt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockError {
    EmailTaken,
    NotFound,
    Internal,
}

pub struct InMemoryUserRepository {
    users: Mutex<Vec<User>>,
    should_fail: Mutex<Option<MockError>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(Vec::new()),
            should_fail: Mutex::new(None),
        }
    }

    pub fn set_error(&self, err: MockError) {
        *self.should_fail.lock().unwrap() = Some(err);
    }

    #[allow(dead_code)]
    pub fn clear_error(&self) {
        *self.should_fail.lock().unwrap() = None;
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, new_user: NewUser) -> Result<User, UserError> {
        if let Some(err) = *self.should_fail.lock().unwrap() {
            return Err(match err {
                MockError::EmailTaken => UserError::EmailTaken,
                MockError::NotFound => UserError::NotFound,
                MockError::Internal => UserError::Internal(anyhow::anyhow!("mock internal error")),
            });
        }

        let mut users = self.users.lock().unwrap();
        if users.iter().any(|u| u.email().as_ref() == new_user.email.as_ref()) {
            return Err(UserError::EmailTaken);
        }

        let now = Utc::now();
        let user = User::new(
            UserId::new(),
            new_user.email,
            new_user.password_hash,
            now,
            now,
        );
        users.push(user.clone());
        Ok(user)
    }

    async fn find_by_id(&self, id: &UserId) -> Result<User, UserError> {
        if let Some(err) = *self.should_fail.lock().unwrap() {
            return Err(match err {
                MockError::EmailTaken => UserError::EmailTaken,
                MockError::NotFound => UserError::NotFound,
                MockError::Internal => UserError::Internal(anyhow::anyhow!("mock internal error")),
            });
        }

        let users = self.users.lock().unwrap();
        users
            .iter()
            .find(|u| u.id().as_uuid() == id.as_uuid())
            .cloned()
            .ok_or(UserError::NotFound)
    }

    async fn find_by_email(&self, email: &Email) -> Result<User, UserError> {
        if let Some(err) = *self.should_fail.lock().unwrap() {
            return Err(match err {
                MockError::EmailTaken => UserError::EmailTaken,
                MockError::NotFound => UserError::NotFound,
                MockError::Internal => UserError::Internal(anyhow::anyhow!("mock internal error")),
            });
        }

        let users = self.users.lock().unwrap();
        users
            .iter()
            .find(|u| u.email().as_ref() == email.as_ref())
            .cloned()
            .ok_or(UserError::NotFound)
    }

    async fn list(&self, offset: u32, limit: u32) -> Result<(Vec<User>, u64), UserError> {
        if let Some(err) = *self.should_fail.lock().unwrap() {
            return Err(match err {
                MockError::EmailTaken => UserError::EmailTaken,
                MockError::NotFound => UserError::NotFound,
                MockError::Internal => UserError::Internal(anyhow::anyhow!("mock internal error")),
            });
        }

        let users = self.users.lock().unwrap();
        let total = users.len() as u64;

        let mut sorted = users.clone();
        sorted.sort_by(|a, b| b.created_at().cmp(&a.created_at()));

        let sliced = sorted
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect::<Vec<_>>();

        Ok((sliced, total))
    }
}

pub struct TestApp {
    router: axum::Router,
    pub user_repo: Arc<InMemoryUserRepository>,
}

impl TestApp {
    pub fn new() -> Self {
        Self::new_with_db_health(true)
    }

    pub fn new_with_db_health(db_healthy: bool) -> Self {
        let user_repo = Arc::new(InMemoryUserRepository::new());
        let db = Database::mock(db_healthy);

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

        let hasher = Arc::new(Argon2PasswordHasher::new());
        let user_service = Arc::new(UserService::new(user_repo.clone(), hasher));
        let state = AppState::new_test(config, db, user_service);

        TestApp {
            router: router(state),
            user_repo,
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
