use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::users::errors::UserError;
use domain::users::model::{
    CursorValue, Email, NewUser, Role, User, UserId, UserCursor, UserListQuery,
};
use domain::users::port::UserRepository;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    first_name: String,
    last_name: String,
    role: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = anyhow::Error;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let email = Email::parse(&row.email).map_err(|e| anyhow::anyhow!(e))?;
        let role = row.role.parse::<Role>().map_err(|e| anyhow::anyhow!(e))?;
        Ok(User::new(
            UserId::from(row.id),
            NewUser { email, password_hash: row.password_hash, first_name: row.first_name, last_name: row.last_name, role },
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
            INSERT INTO users (email, password_hash, first_name, last_name, role)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, password_hash, first_name, last_name, role, created_at, updated_at
            "#,
            new_user.email.as_ref(),
            new_user.password_hash,
            new_user.first_name,
            new_user.last_name,
            new_user.role.as_str(),
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
            SELECT id, email, password_hash, first_name, last_name, role, created_at, updated_at
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
            SELECT id, email, password_hash, first_name, last_name, role, created_at, updated_at
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

        let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            "SELECT id, email, password_hash, first_name, last_name, role, created_at, updated_at FROM users "
        );

        let mut has_where = false;

        if let Some(ref search) = query.search {
            query_builder.push("WHERE email ILIKE ");
            let escaped = search.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            query_builder.push_bind(format!("%{}%", escaped));
            has_where = true;
        }

        if let Some(cursor) = after {
            if has_where {
                query_builder.push(" AND ");
            } else {
                query_builder.push(" WHERE ");
            }
            query_builder.push(format!("({col}, id) {op} ("));

            match cursor.value {
                CursorValue::Timestamp(ts) => {
                    query_builder.push_bind(ts);
                }
                CursorValue::Text(s) => {
                    query_builder.push_bind(s);
                }
            }
            query_builder.push(", ");
            query_builder.push_bind(*cursor.id.as_uuid());
            query_builder.push(")");
        }

        query_builder.push(format!(" ORDER BY {col} {order}, id {order} "));
        query_builder.push(" LIMIT ");
        query_builder.push_bind(fetch_limit);

        let rows = query_builder
            .build_query_as::<UserRow>()
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
