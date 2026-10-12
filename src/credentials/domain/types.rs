#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialType {
    Password(String),
    Passkey(String),
    Totp(String),
}

impl CredentialType {
    pub fn new_password(password: String) -> Self {
        Self::Password(password)
    }

    pub fn new_passkey(passkey: String) -> Self {
        Self::Passkey(passkey)
    }

    pub fn new_totp(totp: String) -> Self {
        Self::Totp(totp)
    }

    pub fn kind(&self) -> CredentialKind {
        match self {
            Self::Password(_) => CredentialKind::Password,
            Self::Passkey(_) => CredentialKind::Passkey,
            Self::Totp(_) => CredentialKind::Totp,
        }
    }

    pub fn secret(&self) -> &str {
        match self {
            Self::Password(secret) | Self::Passkey(secret) | Self::Totp(secret) => secret,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialKind {
    Password,
    Passkey,
    Totp,
}
