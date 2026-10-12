use argon2::{
    Argon2,
    password_hash::{PasswordHasher as _, PasswordVerifier as _, phc::PasswordHash},
};
use async_trait::async_trait;
use tokio::task;

use crate::credentials::domain::{PasswordHashError, PasswordHasher};

pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, password: &str) -> Result<String, PasswordHashError> {
        let password = password.to_owned();

        task::spawn_blocking(move || {
            Argon2::default()
                .hash_password(password.as_bytes())
                .map(|hash| hash.to_string())
        })
        .await
        .map_err(|err| PasswordHashError(Box::new(err)))?
        .map_err(|err| PasswordHashError(Box::new(err)))
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool, PasswordHashError> {
        let password = password.to_owned();
        let hash = hash.to_owned();

        task::spawn_blocking(move || {
            let Ok(parsed) = PasswordHash::new(&hash) else {
                tracing::error!("Invalid hash format in the database");
                return false;
            };

            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .await
        .map_err(|err| PasswordHashError(Box::new(err)))
    }
}
