use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::types::CredentialType;

#[derive(Debug, Clone)]
pub struct Credential {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_type: CredentialType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Credential {
    pub fn new(user_id: Uuid, credential_type: CredentialType) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            credential_type,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    pub fn delete(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
}
