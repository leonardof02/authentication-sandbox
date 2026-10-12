use std::sync::Arc;

use crate::{
    credentials::domain::{Credential, CredentialType, PasswordHashError, PasswordHasher},
    shared::{application::UnitOfWorkFactory, domain::RepositoryError},
    users::domain::User,
};

const MIN_PASSWORD_CHARS: usize = 8;
const MAX_PASSWORD_CHARS: usize = 128;
const MAX_EMAIL_LEN: usize = 254;

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("Email is not valid")]
    InvalidEmail,

    #[error("Password must have between {MIN_PASSWORD_CHARS} and {MAX_PASSWORD_CHARS} characters")]
    InvalidPassword,

    #[error("Email is already taken")]
    EmailTaken,

    #[error(transparent)]
    Hashing(#[from] PasswordHashError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[derive(Clone)]
pub struct RegisterService {
    unit_of_work: Arc<dyn UnitOfWorkFactory>,
    hasher: Arc<dyn PasswordHasher>,
}

impl RegisterService {
    pub fn new(unit_of_work: Arc<dyn UnitOfWorkFactory>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self {
            unit_of_work,
            hasher,
        }
    }

    pub async fn register(&self, email: &str, password: &str) -> Result<User, RegisterError> {
        let email = normalize_email(email).ok_or(RegisterError::InvalidEmail)?;
        validate_password(password)?;

        let hash = self.hasher.hash(password).await?;

        let user = User::new(email);
        let credential = Credential::new(user.id, CredentialType::new_password(hash));

        let uow = self.unit_of_work.begin().await?;

        uow.users().create(&user).await.map_err(|err| match err {
            RepositoryError::Conflict => RegisterError::EmailTaken,
            other => RegisterError::Repository(other),
        })?;

        uow.credentials().create(&credential).await?;
        uow.commit().await?;

        Ok(user)
    }
}

fn normalize_email(email: &str) -> Option<String> {
    let email = email.trim();
    let (local, domain) = email.split_once('@')?;

    let valid = email.len() <= MAX_EMAIL_LEN
        && !local.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.chars().any(char::is_whitespace);

    valid.then(|| email.to_lowercase())
}

fn validate_password(password: &str) -> Result<(), RegisterError> {
    let len = password.chars().count();
    if (MIN_PASSWORD_CHARS..=MAX_PASSWORD_CHARS).contains(&len) {
        Ok(())
    } else {
        Err(RegisterError::InvalidPassword)
    }
}
