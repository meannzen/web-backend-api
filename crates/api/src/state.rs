use std::sync::Arc;

use axum::extract::FromRef;
use domain::ports::health_indicator::HealthIndicator;
use domain::users::service::UserService;
use infra::db::Database;
use infra::repositories::user_repository::PgUserRepository;
use infra::security::password_hasher::Argon2PasswordHasher;
use shared::config::Config;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<dyn HealthIndicator>,
    pub user_service: Arc<UserService>,
}

impl AppState {
    pub fn new(config: Config, db: Database) -> Self {
        let user_repo = Arc::new(PgUserRepository::new(db.pool().clone()));
        let hasher = Arc::new(Argon2PasswordHasher::new());
        AppState {
            config: Arc::new(config),
            db: Arc::new(db),
            user_service: Arc::new(UserService::new(user_repo, hasher)),
        }
    }

    pub fn new_test(
        config: Config,
        db: impl HealthIndicator + Send + Sync + 'static,
        user_service: Arc<UserService>,
    ) -> Self {
        AppState {
            config: Arc::new(config),
            db: Arc::new(db),
            user_service,
        }
    }
}
