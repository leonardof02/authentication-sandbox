use axum::http::StatusCode;

use crate::{
    auth::application::{LinkPasskeyError, LoginError, PasskeyLoginError, RegisterError},
    shared::presentation::ApiError,
};

impl From<RegisterError> for ApiError {
    fn from(err: RegisterError) -> Self {
        match err {
            RegisterError::InvalidEmail => ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_email",
                err.to_string(),
            ),
            RegisterError::InvalidPassword => ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_password",
                err.to_string(),
            ),
            RegisterError::EmailTaken => {
                ApiError::new(StatusCode::CONFLICT, "email_taken", err.to_string())
            }
            RegisterError::Hashing(_) | RegisterError::Repository(_) => ApiError::internal(&err),
        }
    }
}

impl From<LoginError> for ApiError {
    fn from(err: LoginError) -> Self {
        match err {
            LoginError::InvalidCredentials => ApiError::new(
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                err.to_string(),
            ),
            LoginError::Hashing(_) | LoginError::Repository(_) | LoginError::Token(_) => {
                ApiError::internal(&err)
            }
        }
    }
}

impl From<LinkPasskeyError> for ApiError {
    fn from(err: LinkPasskeyError) -> Self {
        match err {
            LinkPasskeyError::UserNotFound => {
                ApiError::new(StatusCode::UNAUTHORIZED, "invalid_token", err.to_string())
            }
            LinkPasskeyError::Rejected(_) => {
                ApiError::new(StatusCode::BAD_REQUEST, "passkey_rejected", err.to_string())
            }
            LinkPasskeyError::Passkey(_) | LinkPasskeyError::Repository(_) => {
                ApiError::internal(&err)
            }
        }
    }
}

impl From<PasskeyLoginError> for ApiError {
    fn from(err: PasskeyLoginError) -> Self {
        match err {
            PasskeyLoginError::NotAvailable => ApiError::new(
                StatusCode::NOT_FOUND,
                "passkey_not_available",
                err.to_string(),
            ),
            PasskeyLoginError::Rejected(_) => ApiError::new(
                StatusCode::UNAUTHORIZED,
                "passkey_rejected",
                err.to_string(),
            ),
            PasskeyLoginError::Passkey(_)
            | PasskeyLoginError::Repository(_)
            | PasskeyLoginError::Token(_) => ApiError::internal(&err),
        }
    }
}
