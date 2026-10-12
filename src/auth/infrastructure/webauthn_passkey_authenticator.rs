use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use serde_json::Value;
use uuid::Uuid;
use webauthn_rs::prelude::{
    Passkey, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential, Url, Webauthn, WebauthnBuilder,
};

use crate::{
    auth::domain::{PasskeyAuthenticator, PasskeyChallenge, PasskeyError, VerifiedPasskey},
    credentials::domain::Credential,
    users::domain::User,
};

const CEREMONY_TTL: Duration = Duration::from_secs(5 * 60);

enum CeremonyState {
    Registration {
        user_id: Uuid,
        state: PasskeyRegistration,
    },
    Authentication {
        user_id: Uuid,
        state: PasskeyAuthentication,
        passkeys: Vec<(Uuid, Passkey)>,
    },
}

struct Ceremony {
    state: CeremonyState,
    expires_at: Instant,
}

#[derive(thiserror::Error)]
#[error("Invalid WebAuthn configuration: {0}")]
pub struct InvalidWebauthnConfig(String);

impl std::fmt::Debug for InvalidWebauthnConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

pub struct WebauthnPasskeyAuthenticator {
    webauthn: Webauthn,
    ceremonies: Mutex<HashMap<Uuid, Ceremony>>,
}

impl WebauthnPasskeyAuthenticator {
    pub fn new(rp_id: &str, origin: &str) -> Result<Self, InvalidWebauthnConfig> {
        let origin = Url::parse(origin).map_err(|err| InvalidWebauthnConfig(err.to_string()))?;
        let webauthn = WebauthnBuilder::new(rp_id, &origin)
            .and_then(|builder| builder.rp_name("Auth Server").build())
            .map_err(|err| InvalidWebauthnConfig(err.to_string()))?;

        Ok(Self {
            webauthn,
            ceremonies: Mutex::new(HashMap::new()),
        })
    }

    fn save(&self, state: CeremonyState) -> Uuid {
        let now = Instant::now();
        let mut ceremonies = self.ceremonies.lock().unwrap();
        ceremonies.retain(|_, ceremony| ceremony.expires_at > now);

        let id = Uuid::new_v4();
        ceremonies.insert(
            id,
            Ceremony {
                state,
                expires_at: now + CEREMONY_TTL,
            },
        );
        id
    }

    fn take(&self, id: Uuid) -> Option<CeremonyState> {
        let ceremony = self.ceremonies.lock().unwrap().remove(&id)?;
        (ceremony.expires_at > Instant::now()).then_some(ceremony.state)
    }
}

fn parse_passkey(credential: &Credential) -> Result<Passkey, PasskeyError> {
    serde_json::from_str(credential.credential_type.secret())
        .map_err(|err| PasskeyError::Internal(Box::new(err)))
}

fn to_json(value: impl serde::Serialize) -> Result<Value, PasskeyError> {
    serde_json::to_value(value).map_err(|err| PasskeyError::Internal(Box::new(err)))
}

impl PasskeyAuthenticator for WebauthnPasskeyAuthenticator {
    fn start_registration(
        &self,
        user: &User,
        existing: &[Credential],
    ) -> Result<PasskeyChallenge, PasskeyError> {
        let exclude = existing
            .iter()
            .map(|credential| parse_passkey(credential).map(|p| p.cred_id().clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let (options, state) = self
            .webauthn
            .start_passkey_registration(user.id, &user.email, &user.email, Some(exclude))
            .map_err(|err| PasskeyError::Internal(Box::new(err)))?;

        let ceremony_id = self.save(CeremonyState::Registration {
            user_id: user.id,
            state,
        });

        Ok(PasskeyChallenge {
            ceremony_id,
            options: to_json(options)?,
        })
    }

    fn finish_registration(
        &self,
        user_id: Uuid,
        ceremony_id: Uuid,
        response: Value,
    ) -> Result<String, PasskeyError> {
        let Some(CeremonyState::Registration {
            user_id: owner,
            state,
        }) = self.take(ceremony_id)
        else {
            return Err(PasskeyError::CeremonyNotFound);
        };
        if owner != user_id {
            return Err(PasskeyError::CeremonyNotFound);
        }

        let response: RegisterPublicKeyCredential = serde_json::from_value(response)
            .map_err(|err| PasskeyError::Verification(Box::new(err)))?;
        let passkey = self
            .webauthn
            .finish_passkey_registration(&response, &state)
            .map_err(|err| PasskeyError::Verification(Box::new(err)))?;

        serde_json::to_string(&passkey).map_err(|err| PasskeyError::Internal(Box::new(err)))
    }

    fn start_authentication(
        &self,
        user_id: Uuid,
        passkeys: &[Credential],
    ) -> Result<PasskeyChallenge, PasskeyError> {
        let passkeys = passkeys
            .iter()
            .map(|credential| Ok((credential.id, parse_passkey(credential)?)))
            .collect::<Result<Vec<_>, PasskeyError>>()?;
        let allowed: Vec<Passkey> = passkeys.iter().map(|(_, p)| p.clone()).collect();

        let (options, state) = self
            .webauthn
            .start_passkey_authentication(&allowed)
            .map_err(|err| PasskeyError::Internal(Box::new(err)))?;

        let ceremony_id = self.save(CeremonyState::Authentication {
            user_id,
            state,
            passkeys,
        });

        Ok(PasskeyChallenge {
            ceremony_id,
            options: to_json(options)?,
        })
    }

    fn finish_authentication(
        &self,
        ceremony_id: Uuid,
        response: Value,
    ) -> Result<VerifiedPasskey, PasskeyError> {
        let Some(CeremonyState::Authentication {
            user_id,
            state,
            mut passkeys,
        }) = self.take(ceremony_id)
        else {
            return Err(PasskeyError::CeremonyNotFound);
        };

        let response: PublicKeyCredential = serde_json::from_value(response)
            .map_err(|err| PasskeyError::Verification(Box::new(err)))?;
        let result = self
            .webauthn
            .finish_passkey_authentication(&response, &state)
            .map_err(|err| PasskeyError::Verification(Box::new(err)))?;

        let (credential_id, passkey) = passkeys
            .iter_mut()
            .find(|(_, passkey)| passkey.cred_id() == result.cred_id())
            .ok_or_else(|| PasskeyError::Verification("unknown passkey".into()))?;

        let updated_data = match passkey.update_credential(&result) {
            Some(true) => Some(
                serde_json::to_string(passkey)
                    .map_err(|err| PasskeyError::Internal(Box::new(err)))?,
            ),
            _ => None,
        };

        Ok(VerifiedPasskey {
            user_id,
            credential_id: *credential_id,
            updated_data,
        })
    }
}
