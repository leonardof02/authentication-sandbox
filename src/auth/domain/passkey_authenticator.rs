use std::error::Error;

use serde_json::Value;
use uuid::Uuid;

use crate::{credentials::domain::Credential, users::domain::User};

pub trait PasskeyAuthenticator: Send + Sync {
    fn start_registration(
        &self,
        user: &User,
        existing: &[Credential],
    ) -> Result<PasskeyChallenge, PasskeyError>;

    fn finish_registration(
        &self,
        user_id: Uuid,
        ceremony_id: Uuid,
        response: Value,
    ) -> Result<String, PasskeyError>;

    fn start_authentication(
        &self,
        user_id: Uuid,
        passkeys: &[Credential],
    ) -> Result<PasskeyChallenge, PasskeyError>;

    fn finish_authentication(
        &self,
        ceremony_id: Uuid,
        response: Value,
    ) -> Result<VerifiedPasskey, PasskeyError>;
}

pub struct PasskeyChallenge {
    pub ceremony_id: Uuid,
    pub options: Value,
}

pub struct VerifiedPasskey {
    pub user_id: Uuid,
    pub credential_id: Uuid,
    pub updated_data: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PasskeyError {
    #[error("Passkey ceremony not found or expired")]
    CeremonyNotFound,

    #[error("Passkey verification failed: {0}")]
    Verification(#[source] Box<dyn Error + Send + Sync>),

    #[error("Passkey error: {0}")]
    Internal(#[source] Box<dyn Error + Send + Sync>),
}
