use std::sync::Arc;

use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    auth::domain::{
        AccessToken, PasskeyAuthenticator, PasskeyChallenge, PasskeyError, TokenError, TokenService,
    },
    credentials::domain::{CredentialKind, CredentialRepository, CredentialType},
    shared::domain::RepositoryError,
    users::domain::UserRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum PasskeyLoginError {
    /// Unknown email or no passkeys; either way the client must use another method.
    #[error("No passkey available for this account")]
    NotAvailable,

    #[error("Passkey was rejected: {0}")]
    Rejected(#[source] PasskeyError),

    #[error(transparent)]
    Passkey(PasskeyError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(transparent)]
    Token(#[from] TokenError),
}

impl From<PasskeyError> for PasskeyLoginError {
    fn from(err: PasskeyError) -> Self {
        match err {
            PasskeyError::Internal(_) => Self::Passkey(err),
            _ => Self::Rejected(err),
        }
    }
}

#[derive(Clone)]
pub struct PasskeyLoginService {
    users: Arc<dyn UserRepository>,
    credentials: Arc<dyn CredentialRepository>,
    passkeys: Arc<dyn PasskeyAuthenticator>,
    tokens: Arc<dyn TokenService>,
}

impl PasskeyLoginService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        credentials: Arc<dyn CredentialRepository>,
        passkeys: Arc<dyn PasskeyAuthenticator>,
        tokens: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            users,
            credentials,
            passkeys,
            tokens,
        }
    }

    pub async fn start(&self, email: &str) -> Result<PasskeyChallenge, PasskeyLoginError> {
        let email = email.trim().to_lowercase();
        let user = self
            .users
            .find_by_email(&email)
            .await?
            .ok_or(PasskeyLoginError::NotAvailable)?;

        let passkeys = self
            .credentials
            .find_active_by_user_and_kind(user.id, CredentialKind::Passkey)
            .await?;
        if passkeys.is_empty() {
            return Err(PasskeyLoginError::NotAvailable);
        }

        Ok(self.passkeys.start_authentication(user.id, &passkeys)?)
    }

    pub async fn finish(
        &self,
        ceremony_id: Uuid,
        response: Value,
    ) -> Result<AccessToken, PasskeyLoginError> {
        let verified = self.passkeys.finish_authentication(ceremony_id, response)?;

        // The passkey may have been removed between `start` and `finish`.
        let mut credential = self
            .credentials
            .find_by_id(verified.credential_id)
            .await?
            .filter(|c| c.is_active() && c.user_id == verified.user_id)
            .ok_or_else(|| {
                PasskeyLoginError::Rejected(PasskeyError::Verification(
                    "passkey is no longer linked to the account".into(),
                ))
            })?;

        if let Some(data) = verified.updated_data {
            credential.credential_type = CredentialType::new_passkey(data);
            credential.updated_at = Utc::now();
            self.credentials.update(&credential).await?;
        }

        Ok(self.tokens.issue(verified.user_id)?)
    }
}
