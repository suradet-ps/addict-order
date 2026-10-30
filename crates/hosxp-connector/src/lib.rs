//! The only crate allowed to talk to MySQL/MariaDB.
//!
//! Read-only is enforced by the database grant, the session mode, the SQL
//! guard, and bound parameters. The connection charset is `tis620` on the
//! verified Sra Bos instance.

pub mod config;
pub mod error;
pub mod pool;
pub mod queries;
pub mod readonly_guard;
pub mod repository;

pub use config::{
    CONFIG_FILE_NAME, ConfigError, HosxConfig, KeyStore, MasterKeyStore, SERVICE_NAME,
    VaultKeyStore, load, load_vault, save_encrypted,
};
pub use error::Error;
pub use pool::connect;
pub use repository::ConnectorRepository;
pub use sqlx::mysql::MySqlPool;
