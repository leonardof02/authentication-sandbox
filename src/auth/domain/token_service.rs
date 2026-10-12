use std::{error::Error, time::Duration};

use uuid::Uuid;

pub trait TokenService: Send + Sync {
    fn issue(&self, user_id: Uuid) -> Result<AccessToken, TokenError>;

    fn verify(&self, token: &str) -> Result<TokenClaims, TokenError>;
}

pub struct AccessToken {
    pub token: String,
    pub expires_in: Duration,
}

#[derive(Debug)]
pub struct TokenClaims {
    pub user_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Token has expired")]
    Expired,

    #[error("Token is not valid")]
    Invalid,

    #[error("Could not issue token: {0}")]
    Issuing(#[source] Box<dyn Error + Send + Sync>),
}
