use axum::extract::{FromRequest, rejection::JsonRejection};

use super::ApiError;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct JsonBody<T>(pub T);

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::new(rejection.status(), "invalid_body", rejection.body_text())
    }
}
