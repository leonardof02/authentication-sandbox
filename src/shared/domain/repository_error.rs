use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Record not found")]
    NotFound,

    #[error("A record with the same unique data already exists")]
    Conflict,

    #[error("Unexpected error: {0}")]
    Unexpected(#[source] Box<dyn Error + Send + Sync>),
}
