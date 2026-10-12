use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{SqlitePool, SqliteTransaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    shared::domain::RepositoryError,
    shared::infrastructure::database::connection::SqliteSource,
    users::domain::{User, UserRepository},
};

pub struct SqliteUserRepository<'a> {
    source: SqliteSource<'a>,
}

impl SqliteUserRepository<'static> {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            source: SqliteSource::Pool(pool),
        }
    }
}

impl<'a> SqliteUserRepository<'a> {
    pub(crate) fn in_transaction(tx: &'a Mutex<SqliteTransaction<'static>>) -> Self {
        Self {
            source: SqliteSource::Transaction(tx),
        }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository<'_> {
    async fn create(&self, user: &User) -> Result<(), RepositoryError> {
        let mut conn = self.source.acquire().await?;

        sqlx::query!(
            "INSERT INTO users (id, email, email_verified, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
            user.id,
            user.email,
            user.email_verified,
            user.created_at,
            user.updated_at,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
        let mut conn = self.source.acquire().await?;

        let user = sqlx::query_as!(
            User,
            r#"SELECT id             AS "id: Uuid",
                      email,
                      email_verified AS "email_verified: bool",
                      created_at     AS "created_at: DateTime<Utc>",
                      updated_at     AS "updated_at: DateTime<Utc>"
               FROM users
               WHERE id = ?"#,
            id,
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let mut conn = self.source.acquire().await?;

        // `email` is `COLLATE NOCASE`, so the lookup is case-insensitive.
        let user = sqlx::query_as!(
            User,
            r#"SELECT id             AS "id: Uuid",
                      email,
                      email_verified AS "email_verified: bool",
                      created_at     AS "created_at: DateTime<Utc>",
                      updated_at     AS "updated_at: DateTime<Utc>"
               FROM users
               WHERE email = ?"#,
            email,
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(user)
    }

    async fn update(&self, user: &User) -> Result<(), RepositoryError> {
        let mut conn = self.source.acquire().await?;

        let result = sqlx::query!(
            "UPDATE users
             SET email = ?, email_verified = ?, updated_at = ?
             WHERE id = ?",
            user.email,
            user.email_verified,
            user.updated_at,
            user.id,
        )
        .execute(&mut *conn)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let mut conn = self.source.acquire().await?;

        // Credentials are removed by `ON DELETE CASCADE`.
        let result = sqlx::query!("DELETE FROM users WHERE id = ?", id)
            .execute(&mut *conn)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), RepositoryError>;

    #[sqlx::test]
    async fn create_and_find_by_id(pool: SqlitePool) -> TestResult {
        let repo = SqliteUserRepository::new(pool);
        let user = User::new("ana@example.com".into());

        repo.create(&user).await?;
        let found = repo.find_by_id(user.id).await?.expect("user should exist");

        assert_eq!(found.id, user.id);
        assert_eq!(found.email, "ana@example.com");
        assert!(!found.email_verified);
        assert_eq!(found.created_at, user.created_at);
        Ok(())
    }

    #[sqlx::test]
    async fn find_by_email_ignores_case(pool: SqlitePool) -> TestResult {
        let repo = SqliteUserRepository::new(pool);
        let user = User::new("Ana@Example.com".into());
        repo.create(&user).await?;

        let found = repo.find_by_email("ana@example.com").await?;

        assert_eq!(found.map(|u| u.id), Some(user.id));
        Ok(())
    }

    #[sqlx::test]
    async fn duplicate_email_is_a_conflict(pool: SqlitePool) -> TestResult {
        let repo = SqliteUserRepository::new(pool);
        repo.create(&User::new("ana@example.com".into())).await?;

        let err = repo.create(&User::new("ANA@example.com".into())).await;

        assert!(matches!(err, Err(RepositoryError::Conflict)));
        Ok(())
    }

    #[sqlx::test]
    async fn update_persists_changes(pool: SqlitePool) -> TestResult {
        let repo = SqliteUserRepository::new(pool);
        let mut user = User::new("ana@example.com".into());
        repo.create(&user).await?;

        user.email_verified = true;
        user.updated_at = Utc::now();
        repo.update(&user).await?;

        let found = repo.find_by_id(user.id).await?.unwrap();
        assert!(found.email_verified);
        Ok(())
    }

    #[sqlx::test]
    async fn update_missing_user_is_not_found(pool: SqlitePool) -> TestResult {
        let repo = SqliteUserRepository::new(pool);

        let err = repo.update(&User::new("nobody@example.com".into())).await;

        assert!(matches!(err, Err(RepositoryError::NotFound)));
        Ok(())
    }

    #[sqlx::test]
    async fn delete_removes_user(pool: SqlitePool) -> TestResult {
        let repo = SqliteUserRepository::new(pool);
        let user = User::new("ana@example.com".into());
        repo.create(&user).await?;

        assert!(repo.delete(user.id).await?);
        assert!(!repo.delete(user.id).await?);
        assert!(repo.find_by_id(user.id).await?.is_none());
        Ok(())
    }
}
