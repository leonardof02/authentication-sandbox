use async_trait::async_trait;

use crate::{
    credentials::domain::CredentialRepository, shared::domain::RepositoryError,
    users::domain::UserRepository,
};

#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn UnitOfWork>, RepositoryError>;
}

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    fn users(&self) -> Box<dyn UserRepository + '_>;
    fn credentials(&self) -> Box<dyn CredentialRepository + '_>;
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
}
