use std::sync::Arc;

use crate::ports::password_hasher::PasswordHasher;
use crate::users::errors::UserError;
use crate::users::model::{Email, NewUser, User, UserId, UserCursor, UserListQuery};
use crate::users::port::UserRepository;

pub struct UserService {
    repo: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self { repo, hasher }
    }

    pub async fn create(&self, email: &str, password: String) -> Result<User, UserError> {
        let email = Email::parse(email).map_err(UserError::InvalidEmail)?;
        let password_hash = self.hasher.hash(password).await.map_err(UserError::Internal)?;
        self.repo.create(NewUser { email, password_hash }).await
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
