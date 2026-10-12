use std::{error::Error, sync::Arc, time::Duration};

use axum::extract::FromRef;
use sqlx::SqlitePool;

use crate::{
    auth::{
        application::{LinkPasskeyService, LoginService, PasskeyLoginService, RegisterService},
        domain::{PasskeyAuthenticator, TokenService},
        infrastructure::{JwtTokenService, WebauthnPasskeyAuthenticator},
    },
    credentials::{
        domain::{CredentialRepository, PasswordHasher},
        infrastructure::{Argon2PasswordHasher, SqliteCredentialRepository},
    },
    shared::infrastructure::database::unit_of_work::SqliteUnitOfWorkFactory,
    users::{domain::UserRepository, infrastructure::SqliteUserRepository},
};

const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: SqlitePool,
    pub users: Arc<dyn UserRepository>,
    pub credentials: Arc<dyn CredentialRepository>,
    pub tokens: Arc<dyn TokenService>,
    pub register: RegisterService,
    pub login: LoginService,
    pub link_passkey: LinkPasskeyService,
    pub passkey_login: PasskeyLoginService,
}

/// `rp_id` is the domain passkeys are bound to (e.g. `localhost`); `origin` is the exact site URL.
pub struct WebauthnConfig {
    pub rp_id: String,
    pub origin: String,
}

impl AppState {
    pub fn new(
        db: SqlitePool,
        jwt_secret: &str,
        webauthn: &WebauthnConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let unit_of_work = Arc::new(SqliteUnitOfWorkFactory::new(db.clone()));
        let users: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(db.clone()));
        let credentials: Arc<dyn CredentialRepository> =
            Arc::new(SqliteCredentialRepository::new(db.clone()));
        let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher);
        let tokens: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(
            jwt_secret.as_bytes(),
            ACCESS_TOKEN_TTL,
        )?);
        let passkeys: Arc<dyn PasskeyAuthenticator> = Arc::new(WebauthnPasskeyAuthenticator::new(
            &webauthn.rp_id,
            &webauthn.origin,
        )?);

        Ok(Self {
            register: RegisterService::new(unit_of_work, hasher.clone()),
            login: LoginService::new(users.clone(), credentials.clone(), hasher, tokens.clone()),
            link_passkey: LinkPasskeyService::new(
                users.clone(),
                credentials.clone(),
                passkeys.clone(),
            ),
            passkey_login: PasskeyLoginService::new(
                users.clone(),
                credentials.clone(),
                passkeys,
                tokens.clone(),
            ),
            users,
            credentials,
            tokens,
            db,
        })
    }
}
