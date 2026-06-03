use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Admin,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Admin => "admin",
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Role::User),
            "admin" => Ok(Role::Admin),
            other => Err(format!("invalid role: {other}")),
        }
    }
}

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

pub struct NewUser {
    pub email: Email,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub role: Role,
}

#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: Email,
    password_hash: String,
    first_name: String,
    last_name: String,
    role: Role,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: UserId, data: NewUser, created_at: DateTime<Utc>, updated_at: DateTime<Utc>) -> Self {
        Self {
            id,
            email: data.email,
            password_hash: data.password_hash,
            first_name: data.first_name,
            last_name: data.last_name,
            role: data.role,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> &UserId { &self.id }
    pub fn email(&self) -> &Email { &self.email }
    pub fn password_hash(&self) -> &str { &self.password_hash }
    pub fn first_name(&self) -> &str { &self.first_name }
    pub fn last_name(&self) -> &str { &self.last_name }
    pub fn role(&self) -> &Role { &self.role }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
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

    pub fn parse_cursor_value(&self, s: &str) -> Result<CursorValue, String> {
        match self {
            SortField::CreatedAt => {
                let ts = DateTime::parse_from_rfc3339(s)
                    .map_err(|_| "invalid cursor".to_string())?
                    .to_utc();
                Ok(CursorValue::Timestamp(ts))
            }
            SortField::Email => Ok(CursorValue::Text(s.to_string())),
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

    pub fn as_str(&self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }

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
