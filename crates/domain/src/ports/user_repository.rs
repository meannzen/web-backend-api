use async_trait::async_trait;

use crate::error::UserError;
use crate::models::user::{Email, NewUser, User, UserId};

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create(&self, new_user: NewUser) -> Result<User, UserError>;
    async fn find_by_id(&self, id: &UserId) -> Result<User, UserError>;
    async fn find_by_email(&self, email: &Email) -> Result<User, UserError>;
    async fn list(&self, offset: u32, limit: u32) -> Result<(Vec<User>, u64), UserError>;
}
