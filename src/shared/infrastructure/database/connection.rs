use std::ops::{Deref, DerefMut};

use sqlx::{Sqlite, SqliteConnection, SqlitePool, SqliteTransaction, pool::PoolConnection};
use tokio::sync::{Mutex, MutexGuard};

pub(crate) enum SqliteSource<'a> {
    Pool(SqlitePool),
    Transaction(&'a Mutex<SqliteTransaction<'static>>),
}

impl SqliteSource<'_> {
    pub(crate) async fn acquire(&self) -> sqlx::Result<SqliteConn<'_>> {
        match self {
            Self::Pool(pool) => Ok(SqliteConn::Pool(pool.acquire().await?)),
            Self::Transaction(tx) => Ok(SqliteConn::Transaction(tx.lock().await)),
        }
    }
}

pub(crate) enum SqliteConn<'a> {
    Pool(PoolConnection<Sqlite>),
    Transaction(MutexGuard<'a, SqliteTransaction<'static>>),
}

impl Deref for SqliteConn<'_> {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Pool(conn) => conn,
            Self::Transaction(tx) => tx,
        }
    }
}

impl DerefMut for SqliteConn<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Pool(conn) => conn,
            Self::Transaction(tx) => tx,
        }
    }
}
