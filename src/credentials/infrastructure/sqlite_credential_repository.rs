use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{SqlitePool, SqliteTransaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    credentials::domain::{Credential, CredentialKind, CredentialRepository, CredentialType},
    shared::{domain::RepositoryError, infrastructure::database::connection::SqliteSource},
};

pub struct SqliteCredentialRepository<'a> {
    source: SqliteSource<'a>,
}

impl SqliteCredentialRepository<'static> {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            source: SqliteSource::Pool(pool),
        }
    }
}

impl<'a> SqliteCredentialRepository<'a> {
    pub(crate) fn in_transaction(tx: &'a Mutex<SqliteTransaction<'static>>) -> Self {
        Self {
            source: SqliteSource::Transaction(tx),
        }
    }
}

struct CredentialRow {
    id: Uuid,
    user_id: Uuid,
    kind: String,
    secret: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl TryFrom<CredentialRow> for Credential {
    type Error = RepositoryError;

    fn try_from(row: CredentialRow) -> Result<Self, Self::Error> {
        let credential_type = match row.kind.as_str() {
            "password" => CredentialType::Password(row.secret),
            "passkey" => CredentialType::Passkey(row.secret),
            "totp" => CredentialType::Totp(row.secret),
            other => {
                return Err(RepositoryError::Unexpected(
                    format!("unknown credential kind: {other}").into(),
                ));
            }
        };

        Ok(Credential {
            id: row.id,
            user_id: row.user_id,
            credential_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }
}

fn kind_as_str(kind: CredentialKind) -> &'static str {
    match kind {
        CredentialKind::Password => "password",
        CredentialKind::Passkey => "passkey",
        CredentialKind::Totp => "totp",
    }
}

#[async_trait]
impl CredentialRepository for SqliteCredentialRepository<'_> {
    async fn create(&self, credential: &Credential) -> Result<(), RepositoryError> {
        let mut conn = self.source.acquire().await?;

        let kind = kind_as_str(credential.credential_type.kind());
        let secret = credential.credential_type.secret();

        sqlx::query!(
            "INSERT INTO credentials (id, user_id, kind, secret, created_at, updated_at, deleted_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            credential.id,
            credential.user_id,
            kind,
            secret,
            credential.created_at,
            credential.updated_at,
            credential.deleted_at,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Credential>, RepositoryError> {
        let mut conn = self.source.acquire().await?;

        let row = sqlx::query_as!(
            CredentialRow,
            r#"SELECT id         AS "id: Uuid",
                      user_id    AS "user_id: Uuid",
                      kind,
                      secret,
                      created_at AS "created_at: DateTime<Utc>",
                      updated_at AS "updated_at: DateTime<Utc>",
                      deleted_at AS "deleted_at: DateTime<Utc>"
               FROM credentials
               WHERE id = ?"#,
            id,
        )
        .fetch_optional(&mut *conn)
        .await?;

        row.map(Credential::try_from).transpose()
    }

    async fn find_active_by_user(&self, user_id: Uuid) -> Result<Vec<Credential>, RepositoryError> {
        let mut conn = self.source.acquire().await?;

        let rows = sqlx::query_as!(
            CredentialRow,
            r#"SELECT id         AS "id: Uuid",
                      user_id    AS "user_id: Uuid",
                      kind,
                      secret,
                      created_at AS "created_at: DateTime<Utc>",
                      updated_at AS "updated_at: DateTime<Utc>",
                      deleted_at AS "deleted_at: DateTime<Utc>"
               FROM credentials
               WHERE user_id = ? AND deleted_at IS NULL
               ORDER BY created_at"#,
            user_id,
        )
        .fetch_all(&mut *conn)
        .await?;

        rows.into_iter().map(Credential::try_from).collect()
    }

    async fn find_active_by_user_and_kind(
        &self,
        user_id: Uuid,
        kind: CredentialKind,
    ) -> Result<Vec<Credential>, RepositoryError> {
        let mut conn = self.source.acquire().await?;

        let kind = kind_as_str(kind);

        let rows = sqlx::query_as!(
            CredentialRow,
            r#"SELECT id         AS "id: Uuid",
                      user_id    AS "user_id: Uuid",
                      kind,
                      secret,
                      created_at AS "created_at: DateTime<Utc>",
                      updated_at AS "updated_at: DateTime<Utc>",
                      deleted_at AS "deleted_at: DateTime<Utc>"
               FROM credentials
               WHERE user_id = ? AND kind = ? AND deleted_at IS NULL
               ORDER BY created_at"#,
            user_id,
            kind,
        )
        .fetch_all(&mut *conn)
        .await?;

        rows.into_iter().map(Credential::try_from).collect()
    }

    async fn count_active_by_user(&self, user_id: Uuid) -> Result<i64, RepositoryError> {
        let mut conn = self.source.acquire().await?;

        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count: i64"
               FROM credentials
               WHERE user_id = ? AND deleted_at IS NULL"#,
            user_id,
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(count)
    }

    async fn update(&self, credential: &Credential) -> Result<(), RepositoryError> {
        let mut conn = self.source.acquire().await?;

        let secret = credential.credential_type.secret();

        let result = sqlx::query!(
            "UPDATE credentials
             SET secret = ?, updated_at = ?, deleted_at = ?
             WHERE id = ?",
            secret,
            credential.updated_at,
            credential.deleted_at,
            credential.id,
        )
        .execute(&mut *conn)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
