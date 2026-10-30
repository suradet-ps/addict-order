//! Connector errors. English and developer-facing; Thai translation happens
//! only at the Tauri command boundary.

use addict_order_core::RepositoryError;

use crate::config::ConfigError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("connection configuration failed: {0}")]
    Config(#[from] ConfigError),
    #[error("cannot connect to HOSxP: {0}")]
    Connect(sqlx::Error),
    #[error("database query failed: {0}")]
    Database(sqlx::Error),
    #[error("read-only guard rejected a statement")]
    Guard,
}

impl From<Error> for RepositoryError {
    fn from(error: Error) -> Self {
        match error {
            Error::Config(_) => RepositoryError::NotConfigured,
            Error::Connect(_) => RepositoryError::Connection,
            Error::Database(error) => RepositoryError::Query(error.to_string()),
            Error::Guard => RepositoryError::Guard,
        }
    }
}
