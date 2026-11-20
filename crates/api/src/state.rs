use std::sync::Arc;

use axum::extract::FromRef;
use domain::boards::service::BoardService;
use domain::ports::health_indicator::HealthIndicator;
use domain::users::service::UserService;
use infra::db::Database;
use infra::repositories::board_repository::PgBoardRepository;
use infra::repositories::user_repository::PgUserRepository;
use infra::security::password_hasher::Argon2PasswordHasher;
use shared::config::Config;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<dyn HealthIndicator>,
    pub user_service: Arc<UserService>,
    pub board_service: Arc<BoardService>,
}

impl AppState {
    pub fn new(config: Config, db: Database) -> Self {
        let user_repo = Arc::new(PgUserRepository::new(db.pool().clone()));
        let board_repo = Arc::new(PgBoardRepository::new(db.pool().clone()));
        let hasher = Arc::new(Argon2PasswordHasher::from_settings(&config.password_hasher));
        AppState {
            config: Arc::new(config),
            db: Arc::new(db),
            user_service: Arc::new(UserService::new(user_repo, hasher)),
            board_service: Arc::new(BoardService::new(board_repo)),
        }
    }

    pub fn new_test(
        config: Config,
        db: impl HealthIndicator + 'static,
        user_service: Arc<UserService>,
        board_service: Arc<BoardService>,
    ) -> Self {
        AppState {
            config: Arc::new(config),
            db: Arc::new(db),
            user_service,
            board_service,
        }
    }
}
