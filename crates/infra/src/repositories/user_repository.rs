use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::error::UserError;
use domain::models::user::{Email, NewUser, User, UserId};
use domain::ports::user_repository::UserRepository;
use sqlx::PgPool;
use uuid::Uuid;

struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = anyhow::Error;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let email = Email::parse(&row.email).map_err(|e| anyhow::anyhow!(e))?;
        Ok(User::new(
            UserId::from(row.id),
            email,
            row.password_hash,
            row.created_at,
            row.updated_at,
        ))
    }
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, new_user: NewUser) -> Result<User, UserError> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1, $2)
            RETURNING id, email, password_hash, created_at, updated_at
            "#,
            new_user.email.as_ref(),
            new_user.password_hash,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.constraint() == Some("users_email_key") => {
                UserError::EmailTaken
            }
            _ => UserError::Internal(anyhow::anyhow!(e).context("failed to create user")),
        })?;

        User::try_from(row).map_err(UserError::Internal)
    }

    async fn find_by_id(&self, id: &UserId) -> Result<User, UserError> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users WHERE id = $1
            "#,
            id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::Internal(anyhow::anyhow!(e).context("failed to find user by id")))?
        .ok_or(UserError::NotFound)?;

        User::try_from(row).map_err(UserError::Internal)
    }

    async fn find_by_email(&self, email: &Email) -> Result<User, UserError> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users WHERE email = $1
            "#,
            email.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            UserError::Internal(anyhow::anyhow!(e).context("failed to find user by email"))
        })?
        .ok_or(UserError::NotFound)?;

        User::try_from(row).map_err(UserError::Internal)
    }
}
