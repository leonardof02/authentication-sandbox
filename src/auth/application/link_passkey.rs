use std::sync::Arc;

use serde_json::Value;
use uuid::Uuid;

use crate::{
    auth::domain::{PasskeyAuthenticator, PasskeyChallenge, PasskeyError},
    credentials::domain::{Credential, CredentialKind, CredentialRepository, CredentialType},
    shared::domain::RepositoryError,
    users::domain::UserRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum LinkPasskeyError {
    #[error("User no longer exists")]
    UserNotFound,

    #[error("Passkey was rejected: {0}")]
    Rejected(#[source] PasskeyError),

    #[error(transparent)]
    Passkey(PasskeyError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl From<PasskeyError> for LinkPasskeyError {
    fn from(err: PasskeyError) -> Self {
        match err {
            PasskeyError::Internal(_) => Self::Passkey(err),
            _ => Self::Rejected(err),
        }
    }
}

#[derive(Clone)]
pub struct LinkPasskeyService {
    users: Arc<dyn UserRepository>,
    credentials: Arc<dyn CredentialRepository>,
    passkeys: Arc<dyn PasskeyAuthenticator>,
}

impl LinkPasskeyService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        credentials: Arc<dyn CredentialRepository>,
        passkeys: Arc<dyn PasskeyAuthenticator>,
    ) -> Self {
        Self {
            users,
            credentials,
            passkeys,
        }
    }

    pub async fn start(&self, user_id: Uuid) -> Result<PasskeyChallenge, LinkPasskeyError> {
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or(LinkPasskeyError::UserNotFound)?;
        let existing = self
            .credentials
            .find_active_by_user_and_kind(user_id, CredentialKind::Passkey)
            .await?;

        Ok(self.passkeys.start_registration(&user, &existing)?)
    }

    pub async fn finish(
        &self,
        user_id: Uuid,
        ceremony_id: Uuid,
        response: Value,
    ) -> Result<Credential, LinkPasskeyError> {
        let passkey = self
            .passkeys
            .finish_registration(user_id, ceremony_id, response)?;

        let credential = Credential::new(user_id, CredentialType::new_passkey(passkey));
        self.credentials.create(&credential).await?;

        Ok(credential)
    }
}
