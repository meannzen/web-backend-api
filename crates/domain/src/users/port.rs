use async_trait::async_trait;

use crate::users::errors::UserError;
use crate::users::model::{Email, NewUser, User, UserCursor, UserId, UserListQuery};

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create(&self, new_user: NewUser) -> Result<User, UserError>;
    async fn find_by_id(&self, id: &UserId) -> Result<User, UserError>;
    async fn find_by_email(&self, email: &Email) -> Result<User, UserError>;
    async fn list(
        &self,
        query: &UserListQuery,
        after: Option<UserCursor>,
        limit: u32,
    ) -> Result<(Vec<User>, bool), UserError>;
}
