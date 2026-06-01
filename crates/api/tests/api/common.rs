use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use api::routes::router;
use api::state::AppState;
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, Response, header};
use chrono::Utc;
use domain::users::errors::UserError;
use domain::users::model::{
    CursorValue, Email, NewUser, SortField, User, UserId, UserCursor, UserListQuery,
};
use domain::users::port::UserRepository;
use domain::users::service::UserService;
use infra::db::Database;
use infra::security::password_hasher::Argon2PasswordHasher;
use secrecy::SecretString;
use shared::config::{ApplicationSettings, Config, DatabaseSettings, Environment};
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

    async fn list(
        &self,
        query: &UserListQuery,
        after: Option<UserCursor>,
        limit: u32,
    ) -> Result<(Vec<User>, bool), UserError> {
        if let Some(err) = *self.should_fail.lock().unwrap() {
            return Err(match err {
                MockError::EmailTaken => UserError::EmailTaken,
                MockError::NotFound => UserError::NotFound,
                MockError::Internal => UserError::Internal(anyhow::anyhow!("mock internal error")),
            });
        }

        let users = self.users.lock().unwrap();
        let sorted = users.clone();

        // Apply search filter
        let filtered: Vec<User> = if let Some(ref search) = query.search {
            let lower = search.to_lowercase();
            sorted.into_iter().filter(|u| u.email().as_ref().contains(&lower)).collect()
        } else {
            sorted
        };

        // Sort according to query
        let mut sorted = filtered;
        match query.sort_by {
            SortField::CreatedAt => {
                sorted.sort_by(|a, b| {
                    let ord = a.created_at().cmp(&b.created_at())
                        .then_with(|| a.id().as_uuid().cmp(b.id().as_uuid()));
                    match query.direction {
                        domain::users::model::SortDirection::Asc => ord,
                        domain::users::model::SortDirection::Desc => ord.reverse(),
                    }
                });
            }
            SortField::Email => {
                sorted.sort_by(|a, b| {
                    let ord = a.email().as_ref().cmp(b.email().as_ref())
                        .then_with(|| a.id().as_uuid().cmp(b.id().as_uuid()));
                    match query.direction {
                        domain::users::model::SortDirection::Asc => ord,
                        domain::users::model::SortDirection::Desc => ord.reverse(),
                    }
                });
            }
        }

        // Apply cursor filter
        let page: Vec<User> = if let Some(cursor) = after {
            let is_after: Box<dyn Fn(&User) -> bool> = match (&cursor.value, cursor.sort_by) {
                (CursorValue::Timestamp(ts), SortField::CreatedAt) => {
                    let ts = *ts;
                    let uuid = *cursor.id.as_uuid();
                    match query.direction {
                        domain::users::model::SortDirection::Desc => Box::new(move |u: &User| {
                            (u.created_at(), u.id().as_uuid()) < (ts, &uuid)
                        }),
                        domain::users::model::SortDirection::Asc => Box::new(move |u: &User| {
                            (u.created_at(), u.id().as_uuid()) > (ts, &uuid)
                        }),
                    }
                }
                (CursorValue::Text(s), SortField::Email) => {
                    let s = s.clone();
                    let uuid = *cursor.id.as_uuid();
                    match query.direction {
                        domain::users::model::SortDirection::Desc => Box::new(move |u: &User| {
                            (u.email().as_ref(), u.id().as_uuid()) < (s.as_str(), &uuid)
                        }),
                        domain::users::model::SortDirection::Asc => Box::new(move |u: &User| {
                            (u.email().as_ref(), u.id().as_uuid()) > (s.as_str(), &uuid)
                        }),
                    }
                }
                _ => Box::new(|_: &User| false),
            };
            sorted.into_iter().filter(|u| is_after(u)).take(limit as usize + 1).collect()
        } else {
            sorted.into_iter().take(limit as usize + 1).collect()
        };

        let has_next_page = page.len() > limit as usize;
        let mut page = page;
        page.truncate(limit as usize);

        Ok((page, has_next_page))
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
                jwt_secret: Some(SecretString::from(TEST_JWT_SECRET)),
                cors_origins: vec![],
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
        let (mut parts, body) = req.into_parts();
        parts
            .extensions
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 0))));
        self.router
            .clone()
            .oneshot(Request::from_parts(parts, body))
            .await
            .unwrap()
    }

    pub async fn get(&self, uri: &str) -> Response<Body> {
        self.request(
            Request::builder()
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {}", make_test_token()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }

    pub async fn post<B: serde::Serialize>(&self, uri: &str, body: &B) -> Response<Body> {
        self.request(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", make_test_token()))
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
