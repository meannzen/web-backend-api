use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::users::errors::UserError;
use domain::users::model::{
    CursorValue, Email, NewUser, User, UserId, UserCursor, UserListQuery,
};
use domain::users::port::UserRepository;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
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

    async fn list(
        &self,
        query: &UserListQuery,
        after: Option<UserCursor>,
        limit: u32,
    ) -> Result<(Vec<User>, bool), UserError> {
        let fetch_limit = (limit + 1) as i64;
        let col = query.sort_by.column();
        let order = query.direction.sql_order();
        let op = query.direction.cursor_op();

        let mut conditions: Vec<String> = Vec::new();
        let mut param_idx: u32 = 1;

        if query.search.is_some() {
            conditions.push(format!("email ILIKE ${}", param_idx));
            param_idx += 1;
        }

        if after.is_some() {
            conditions.push(format!("({col}, id) {op} (${}, ${})", param_idx, param_idx + 1));
            param_idx += 2;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT id, email, password_hash, created_at, updated_at \
             FROM users \
             {where_clause} \
             ORDER BY {col} {order}, id {order} \
             LIMIT ${}",
            param_idx
        );

        let mut q = sqlx::query_as::<_, UserRow>(sqlx::AssertSqlSafe(sql));

        if let Some(ref search) = query.search {
            q = q.bind(format!("%{}%", search));
        }

        if let Some(cursor) = after {
            match cursor.value {
                CursorValue::Timestamp(ts) => {
                    q = q.bind(ts).bind(*cursor.id.as_uuid());
                }
                CursorValue::Text(s) => {
                    q = q.bind(s).bind(*cursor.id.as_uuid());
                }
            }
        }

        q = q.bind(fetch_limit);

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| UserError::Internal(anyhow::anyhow!(e).context("failed to list users")))?;

        let has_next_page = rows.len() > limit as usize;
        let mut rows = rows;
        rows.truncate(limit as usize);

        let users = rows
            .into_iter()
            .map(User::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(UserError::Internal)?;

        Ok((users, has_next_page))
    }
}
