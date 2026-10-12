use std::sync::Arc;

use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    auth::domain::{AccessToken, TokenError, TokenService},
    credentials::domain::{
        CredentialKind, CredentialRepository, PasswordHashError, PasswordHasher,
    },
    shared::domain::RepositoryError,
    users::domain::UserRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    /// Same error for unknown email and wrong password, so it doesn't reveal which emails are registered.
    #[error("Email or password is incorrect")]
    InvalidCredentials,

    #[error(transparent)]
    Hashing(#[from] PasswordHashError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(transparent)]
    Token(#[from] TokenError),
}

#[derive(Clone)]
pub struct LoginService {
    users: Arc<dyn UserRepository>,
    credentials: Arc<dyn CredentialRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
    dummy_hash: Arc<OnceCell<String>>,
}

impl LoginService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        credentials: Arc<dyn CredentialRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            users,
            credentials,
            hasher,
            tokens,
            dummy_hash: Arc::new(OnceCell::new()),
        }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AccessToken, LoginError> {
        let email = email.trim().to_lowercase();

        let user = self.users.find_by_email(&email).await?;
        let stored_hash = match &user {
            Some(user) => self.password_hash_of(user.id).await?,
            None => None,
        };

        // For unknown emails, verify against a dummy hash anyway so the response takes
        // the same time and timing doesn't reveal whether the account exists.
        let password_matches = match &stored_hash {
            Some(hash) => self.hasher.verify(password, hash).await?,
            None => {
                self.hasher
                    .verify(password, self.dummy_hash().await?)
                    .await?;
                false
            }
        };

        match user {
            Some(user) if password_matches => Ok(self.tokens.issue(user.id)?),
            _ => Err(LoginError::InvalidCredentials),
        }
    }

    async fn password_hash_of(&self, user_id: Uuid) -> Result<Option<String>, RepositoryError> {
        let passwords = self
            .credentials
            .find_active_by_user_and_kind(user_id, CredentialKind::Password)
            .await?;

        Ok(passwords
            .into_iter()
            .next()
            .map(|credential| credential.credential_type.secret().to_owned()))
    }

    /// Computed lazily once, with the same hasher (and parameters) as real hashes.
    async fn dummy_hash(&self) -> Result<&str, PasswordHashError> {
        self.dummy_hash
            .get_or_try_init(|| self.hasher.hash("dummy password to equalize timing"))
            .await
            .map(String::as_str)
    }
}
