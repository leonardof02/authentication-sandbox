use std::time::Duration;

use chrono::Utc;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::domain::{AccessToken, TokenClaims, TokenError, TokenService};

const ISSUER: &str = "auth-server";
const MIN_SECRET_BYTES: usize = 32;

/// Standard JWT Claims (RFC 7519).
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    iss: String,
    iat: i64,
    exp: i64,
    jti: Uuid,
}

#[derive(thiserror::Error)]
#[error("JWT_SECRET must be at least {MIN_SECRET_BYTES} bytes long")]
pub struct WeakSecretError;

impl std::fmt::Debug for WeakSecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

pub struct JwtTokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    ttl: Duration,
}

impl JwtTokenService {
    pub fn new(secret: &[u8], ttl: Duration) -> Result<Self, WeakSecretError> {
        if secret.len() < MIN_SECRET_BYTES {
            return Err(WeakSecretError);
        }

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[ISSUER]);
        validation.set_required_spec_claims(&["exp", "iss", "sub"]);

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            validation,
            ttl,
        })
    }
}

impl TokenService for JwtTokenService {
    fn issue(&self, user_id: Uuid) -> Result<AccessToken, TokenError> {
        let now = Utc::now().timestamp();

        let claims = Claims {
            sub: user_id,
            iss: ISSUER.to_owned(),
            iat: now,
            exp: now + self.ttl.as_secs() as i64,
            jti: Uuid::new_v4(),
        };

        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)
            .map_err(|err| TokenError::Issuing(Box::new(err)))?;

        Ok(AccessToken {
            token,
            expires_in: self.ttl,
        })
    }

    fn verify(&self, token: &str) -> Result<TokenClaims, TokenError> {
        let data = decode::<Claims>(token, &self.decoding_key, &self.validation).map_err(
            |err| match err.kind() {
                ErrorKind::ExpiredSignature => TokenError::Expired,
                _ => TokenError::Invalid,
            },
        )?;

        Ok(TokenClaims {
            user_id: data.claims.sub,
        })
    }
}
