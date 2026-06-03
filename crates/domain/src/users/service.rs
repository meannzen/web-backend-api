use std::sync::Arc;

use crate::ports::password_hasher::PasswordHasher;
use crate::users::errors::UserError;
use crate::users::model::{Email, NewUser, Role, User, UserId, UserCursor, UserListQuery};
use crate::users::port::UserRepository;

pub struct UserService {
    repo: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self { repo, hasher }
    }

    pub async fn create(
        &self,
        email: &str,
        password: String,
        first_name: String,
        last_name: String,
        role: Role,
    ) -> Result<User, UserError> {
        let email = Email::parse(email).map_err(UserError::InvalidEmail)?;
        let password_hash = self.hasher.hash(password).await.map_err(UserError::Internal)?;
        self.repo.create(NewUser { email, password_hash, first_name, last_name, role }).await
    }

    pub async fn authenticate(&self, email: &str, password: String) -> Result<User, UserError> {
        // A real Argon2id hash used as a dummy when the email is not found.
        // Running verify against it equalizes response time so callers cannot
        // enumerate valid emails by measuring latency.
        const DUMMY_HASH: &str =
            "$argon2id$v=19$m=19456,t=2,p=1$dFdVZhCf7IUVXfu3UJbm6Q$5yUtdFaHs6bwC+AfybrfJzM6oYDTkAaHVVGDRJg+Yo4";

        let email = Email::parse(email).map_err(|_| UserError::InvalidCredentials)?;
        let lookup = self.repo.find_by_email(&email).await;

        match lookup {
            Ok(user) => {
                let valid = self
                    .hasher
                    .verify(&password, user.password_hash())
                    .await
                    .map_err(UserError::Internal)?;
                if valid { Ok(user) } else { Err(UserError::InvalidCredentials) }
            }
            Err(UserError::NotFound) => {
                // Discard the result; we only care about the time spent.
                let _ = self.hasher.verify(&password, DUMMY_HASH).await;
                Err(UserError::InvalidCredentials)
            }
            Err(other) => Err(other),
        }
    }

    pub async fn get_by_id(&self, id: UserId) -> Result<User, UserError> {
        self.repo.find_by_id(&id).await
    }

    pub async fn list(
        &self,
        query: &UserListQuery,
        after: Option<UserCursor>,
        limit: u32,
    ) -> Result<(Vec<User>, bool), UserError> {
        self.repo.list(query, after, limit).await
    }
}
