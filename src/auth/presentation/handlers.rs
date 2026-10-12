use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::Html};

use super::{
    AuthenticatedUser,
    dto::{
        LoginRequest, PasskeyChallengeResponse, PasskeyFinishRequest, PasskeyLoginStartRequest,
        PasskeyResponse, RegisterRequest, TokenResponse, UserResponse,
    },
};
use crate::{
    auth::application::{LinkPasskeyService, LoginService, PasskeyLoginService, RegisterService},
    shared::presentation::{ApiError, JsonBody},
    users::domain::UserRepository,
};

#[utoipa::path(
    post,
    path = "/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = CREATED, description = "User created", body = UserResponse),
        (status = BAD_REQUEST, description = "Body is not valid JSON", body = ApiError),
        (status = CONFLICT, description = "An account with that email already exists", body = ApiError),
        (status = UNSUPPORTED_MEDIA_TYPE, description = "Missing `Content-Type: application/json`", body = ApiError),
        (status = UNPROCESSABLE_ENTITY, description = "Missing fields, or invalid email or password", body = ApiError),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = ApiError),
    )
)]
pub async fn register(
    State(register): State<RegisterService>,
    JsonBody(body): JsonBody<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    let user = register.register(&body.email, &body.password).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

#[utoipa::path(
    post,
    path = "/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = OK, description = "Signed in", body = TokenResponse),
        (status = BAD_REQUEST, description = "Body is not valid JSON", body = ApiError),
        (status = UNAUTHORIZED, description = "Wrong email or password", body = ApiError),
        (status = UNSUPPORTED_MEDIA_TYPE, description = "Missing `Content-Type: application/json`", body = ApiError),
        (status = UNPROCESSABLE_ENTITY, description = "Missing fields", body = ApiError),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = ApiError),
    )
)]
pub async fn login(
    State(login): State<LoginService>,
    JsonBody(body): JsonBody<LoginRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let token = login.login(&body.email, &body.password).await?;

    Ok(Json(token.into()))
}

#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    security(("bearer_auth" = [])),
    responses(
        (status = OK, description = "Authenticated user", body = UserResponse),
        (status = UNAUTHORIZED, description = "Token is missing, invalid or expired", body = ApiError),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = ApiError),
    )
)]
pub async fn me(
    auth: AuthenticatedUser,
    State(users): State<Arc<dyn UserRepository>>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = users
        .find_by_id(auth.user_id)
        .await
        .map_err(|err| ApiError::internal(&err))?
        .ok_or_else(|| {
            // Valid token, but the user was deleted after it was issued.
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "User no longer exists",
            )
        })?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    post,
    path = "/passkeys/register/start",
    tag = "passkeys",
    security(("bearer_auth" = [])),
    responses(
        (status = OK, description = "Challenge created", body = PasskeyChallengeResponse),
        (status = UNAUTHORIZED, description = "Token is missing, invalid or expired", body = ApiError),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = ApiError),
    )
)]
pub async fn start_passkey_registration(
    auth: AuthenticatedUser,
    State(link): State<LinkPasskeyService>,
) -> Result<Json<PasskeyChallengeResponse>, ApiError> {
    let challenge = link.start(auth.user_id).await?;

    Ok(Json(challenge.into()))
}

#[utoipa::path(
    post,
    path = "/passkeys/register/finish",
    tag = "passkeys",
    security(("bearer_auth" = [])),
    request_body = PasskeyFinishRequest,
    responses(
        (status = CREATED, description = "Passkey linked", body = PasskeyResponse),
        (status = BAD_REQUEST, description = "Invalid credential, or ceremony expired or already used", body = ApiError),
        (status = UNAUTHORIZED, description = "Token is missing, invalid or expired", body = ApiError),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = ApiError),
    )
)]
pub async fn finish_passkey_registration(
    auth: AuthenticatedUser,
    State(link): State<LinkPasskeyService>,
    JsonBody(body): JsonBody<PasskeyFinishRequest>,
) -> Result<(StatusCode, Json<PasskeyResponse>), ApiError> {
    let credential = link
        .finish(auth.user_id, body.ceremony_id, body.credential)
        .await?;

    Ok((StatusCode::CREATED, Json(credential.into())))
}

#[utoipa::path(
    post,
    path = "/passkeys/login/start",
    tag = "passkeys",
    request_body = PasskeyLoginStartRequest,
    responses(
        (status = OK, description = "Challenge created", body = PasskeyChallengeResponse),
        (status = NOT_FOUND, description = "Account does not exist or has no passkeys", body = ApiError),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = ApiError),
    )
)]
pub async fn start_passkey_login(
    State(login): State<PasskeyLoginService>,
    JsonBody(body): JsonBody<PasskeyLoginStartRequest>,
) -> Result<Json<PasskeyChallengeResponse>, ApiError> {
    let challenge = login.start(&body.email).await?;

    Ok(Json(challenge.into()))
}

#[utoipa::path(
    post,
    path = "/passkeys/login/finish",
    tag = "passkeys",
    request_body = PasskeyFinishRequest,
    responses(
        (status = OK, description = "Signed in", body = TokenResponse),
        (status = UNAUTHORIZED, description = "Invalid signature, or ceremony expired or already used", body = ApiError),
        (status = INTERNAL_SERVER_ERROR, description = "Internal error", body = ApiError),
    )
)]
pub async fn finish_passkey_login(
    State(login): State<PasskeyLoginService>,
    JsonBody(body): JsonBody<PasskeyFinishRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let token = login.finish(body.ceremony_id, body.credential).await?;

    Ok(Json(token.into()))
}

pub async fn passkeys_demo() -> Html<&'static str> {
    Html(include_str!("passkeys_demo.html"))
}
