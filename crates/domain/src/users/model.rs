use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        UserId(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        UserId(id)
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim().to_lowercase();
        let parts: Vec<&str> = s.splitn(2, '@').collect();
        if parts.len() != 2 || parts[0].is_empty() || !parts[1].contains('.') {
            return Err(format!("'{}' is not a valid email", s));
        }
        Ok(Email(s))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: Email,
    password_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        id: UserId,
        email: Email,
        password_hash: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self { id, email, password_hash, created_at, updated_at }
    }

    pub fn id(&self) -> &UserId { &self.id }
    pub fn email(&self) -> &Email { &self.email }
    pub fn password_hash(&self) -> &str { &self.password_hash }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

pub struct NewUser {
    pub email: Email,
    pub password_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    CreatedAt,
    Email,
}

impl SortField {
    pub fn column(&self) -> &'static str {
        match self {
            SortField::CreatedAt => "created_at",
            SortField::Email => "email",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn sql_order(&self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }

    /// SQL comparison operator to advance past the cursor in this direction.
    pub fn cursor_op(&self) -> &'static str {
        match self {
            SortDirection::Asc => ">",
            SortDirection::Desc => "<",
        }
    }
}

pub struct UserListQuery {
    pub search: Option<String>,
    pub sort_by: SortField,
    pub direction: SortDirection,
}

#[derive(Debug, Clone)]
pub enum CursorValue {
    Timestamp(DateTime<Utc>),
    Text(String),
}

pub struct UserCursor {
    pub sort_by: SortField,
    pub direction: SortDirection,
    pub value: CursorValue,
    pub id: UserId,
}
