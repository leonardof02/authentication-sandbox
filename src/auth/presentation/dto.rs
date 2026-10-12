use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::domain::{AccessToken, PasskeyChallenge},
    credentials::domain::Credential,
    users::domain::User,
};

// No `Debug` on purpose, so the password can never end up in a log.
#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(format = Email, example = "ana@example.com")]
    pub email: String,

    #[schema(format = Password, min_length = 8, max_length = 128, example = "correct horse battery staple")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,

    #[schema(example = "ana@example.com")]
    pub email: String,

    pub email_verified: bool,

    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            created_at: user.created_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(format = Email, example = "ana@example.com")]
    pub email: String,

    #[schema(format = Password, example = "correct horse battery staple")]
    pub password: String,
}

/// Field names follow the OAuth 2.0 token response (RFC 6749 §5.1).
#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,

    #[schema(example = "Bearer")]
    pub token_type: &'static str,

    /// Seconds until the token expires.
    #[schema(example = 900)]
    pub expires_in: u64,
}

impl From<AccessToken> for TokenResponse {
    fn from(token: AccessToken) -> Self {
        Self {
            access_token: token.token,
            token_type: "Bearer",
            expires_in: token.expires_in.as_secs(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct PasskeyChallengeResponse {
    /// Must be sent back in the `finish` request.
    pub ceremony_id: Uuid,

    /// Options for `navigator.credentials.create()` (registration) or `.get()` (sign-in).
    #[schema(value_type = Object)]
    pub options: serde_json::Value,
}

impl From<PasskeyChallenge> for PasskeyChallengeResponse {
    fn from(challenge: PasskeyChallenge) -> Self {
        Self {
            ceremony_id: challenge.ceremony_id,
            options: challenge.options,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct PasskeyFinishRequest {
    pub ceremony_id: Uuid,

    /// The credential returned by the browser, serialized as JSON.
    #[schema(value_type = Object)]
    pub credential: serde_json::Value,
}

#[derive(Deserialize, ToSchema)]
pub struct PasskeyLoginStartRequest {
    #[schema(format = Email, example = "ana@example.com")]
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct PasskeyResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<Credential> for PasskeyResponse {
    fn from(credential: Credential) -> Self {
        Self {
            id: credential.id,
            created_at: credential.created_at,
        }
    }
}
