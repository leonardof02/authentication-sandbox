use crate::shared::domain::RepositoryError;

impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => Self::Conflict,
            _ => Self::Unexpected(Box::new(err)),
        }
    }
}
