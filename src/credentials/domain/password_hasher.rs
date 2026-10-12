use std::error::Error;

use async_trait::async_trait;

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &str) -> Result<String, PasswordHashError>;
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, PasswordHashError>;
}

#[derive(Debug, thiserror::Error)]
#[error("Error processing password: {0}")]
pub struct PasswordHashError(#[source] pub Box<dyn Error + Send + Sync>);
