use async_trait::async_trait;
use sqlx::{SqlitePool, SqliteTransaction};
use tokio::sync::Mutex;

use crate::{
    credentials::{domain::CredentialRepository, infrastructure::SqliteCredentialRepository},
    shared::{
        application::{UnitOfWork, UnitOfWorkFactory},
        domain::RepositoryError,
    },
    users::{domain::UserRepository, infrastructure::SqliteUserRepository},
};

pub struct SqliteUnitOfWorkFactory {
    pool: SqlitePool,
}

impl SqliteUnitOfWorkFactory {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOfWorkFactory for SqliteUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn UnitOfWork>, RepositoryError> {
        let tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        Ok(Box::new(SqliteUnitOfWork { tx: Mutex::new(tx) }))
    }
}

struct SqliteUnitOfWork {
    tx: Mutex<SqliteTransaction<'static>>,
}

#[async_trait]
impl UnitOfWork for SqliteUnitOfWork {
    fn users(&self) -> Box<dyn UserRepository + '_> {
        Box::new(SqliteUserRepository::in_transaction(&self.tx))
    }

    fn credentials(&self) -> Box<dyn CredentialRepository + '_> {
        Box::new(SqliteCredentialRepository::in_transaction(&self.tx))
    }

    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        self.tx.into_inner().commit().await?;
        Ok(())
    }
}
