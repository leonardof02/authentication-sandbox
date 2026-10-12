use async_trait::async_trait;
use uuid::Uuid;

use super::{Credential, CredentialKind};
use crate::shared::domain::RepositoryError;

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn create(&self, credential: &Credential) -> Result<(), RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Credential>, RepositoryError>;

    async fn find_active_by_user(&self, user_id: Uuid) -> Result<Vec<Credential>, RepositoryError>;

    async fn find_active_by_user_and_kind(
        &self,
        user_id: Uuid,
        kind: CredentialKind,
    ) -> Result<Vec<Credential>, RepositoryError>;

    async fn count_active_by_user(&self, user_id: Uuid) -> Result<i64, RepositoryError>;
    async fn update(&self, credential: &Credential) -> Result<(), RepositoryError>;
}
