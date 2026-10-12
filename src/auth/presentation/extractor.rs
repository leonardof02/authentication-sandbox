use std::sync::Arc;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    auth::domain::{TokenError, TokenService},
    shared::presentation::ApiError,
};

/// User authenticated from the `Authorization: Bearer <token>` header.
/// Adding it as a handler argument makes the route require a valid token.
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    Arc<dyn TokenService>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).ok_or_else(|| {
            AuthRejection(ApiError::new(
                StatusCode::UNAUTHORIZED,
                "missing_token",
                "Missing `Authorization: Bearer <token>` header",
            ))
        })?;

        let tokens = Arc::<dyn TokenService>::from_ref(state);
        let claims = tokens.verify(token)?;

        Ok(Self {
            user_id: claims.user_id,
        })
    }
}

fn bearer_token(parts: &Parts) -> Option<&str> {
    let value = parts.headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;

    // The scheme is case-insensitive (RFC 9110 §11.1).
    scheme
        .eq_ignore_ascii_case("bearer")
        .then_some(token.trim())
}

/// A 401 with `WWW-Authenticate: Bearer`, as required by RFC 6750.
pub struct AuthRejection(ApiError);

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        ([(header::WWW_AUTHENTICATE, "Bearer")], self.0).into_response()
    }
}

impl From<TokenError> for AuthRejection {
    fn from(err: TokenError) -> Self {
        let api_error = match err {
            TokenError::Expired => {
                ApiError::new(StatusCode::UNAUTHORIZED, "token_expired", err.to_string())
            }
            TokenError::Invalid => {
                ApiError::new(StatusCode::UNAUTHORIZED, "invalid_token", err.to_string())
            }
            TokenError::Issuing(_) => ApiError::internal(&err),
        };
        Self(api_error)
    }
}
