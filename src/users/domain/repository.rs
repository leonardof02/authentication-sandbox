use async_trait::async_trait;
use uuid::Uuid;

use super::User;
use crate::shared::domain::RepositoryError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// `Conflict` if a user with that email already exists.
    async fn create(&self, user: &User) -> Result<(), RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError>;

    /// Case-insensitive.
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError>;

    /// `NotFound` if the user does not exist.
    async fn update(&self, user: &User) -> Result<(), RepositoryError>;

    /// Deletes the user and all their credentials. Returns `false` if it did not exist.
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
}
