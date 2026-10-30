//! MySQL/MariaDB connection pool with an enforced read-only session and the
//! verified `tis620` connection charset.

use std::time::Duration;

use secrecy::ExposeSecret;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions, MySqlSslMode};

use crate::config::HosxConfig;
use crate::error::Error;
use crate::readonly_guard::READ_ONLY_SESSION_SQL;

const MAX_CONNECTIONS: u32 = 5;
const DEFAULT_SSL_MODE: MySqlSslMode = MySqlSslMode::Preferred;
const MYSQL_STATEMENT_TIMEOUT_SQL: &str = "SET SESSION max_execution_time = 5000";
const MARIADB_STATEMENT_TIMEOUT_SQL: &str = "SET SESSION max_statement_time = 5";

pub async fn connect(cfg: &HosxConfig) -> Result<MySqlPool, Error> {
    let options = MySqlConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .database(&cfg.database)
        .username(&cfg.user)
        // The HOSxP columns are tis620. sqlx decodes text as UTF-8, so ask
        // MySQL to convert TIS-620 column data to UTF-8 on the wire. The CLI
        // verification still uses --default-character-set=tis620; this is the
        // application-side equivalent for a UTF-8 Rust string decoder.
        .charset("utf8mb4")
        .ssl_mode(configured_ssl_mode())
        .password(cfg.password.expose_secret());

    MySqlPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(5))
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query(READ_ONLY_SESSION_SQL)
                    .execute(&mut *conn)
                    .await?;
                // Best effort: MySQL and MariaDB use different variables.
                let _ = sqlx::query(MYSQL_STATEMENT_TIMEOUT_SQL)
                    .execute(&mut *conn)
                    .await;
                let _ = sqlx::query(MARIADB_STATEMENT_TIMEOUT_SQL)
                    .execute(&mut *conn)
                    .await;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .map_err(Error::Connect)
}

fn configured_ssl_mode() -> MySqlSslMode {
    ssl_mode_from(std::env::var("ADDICT_ORDER_HOSXP_SSL_MODE").ok().as_deref())
}

fn ssl_mode_from(raw: Option<&str>) -> MySqlSslMode {
    match raw.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("required") => MySqlSslMode::Required,
        Some("verify_ca") => MySqlSslMode::VerifyCa,
        Some("verify_identity") => MySqlSslMode::VerifyIdentity,
        _ => DEFAULT_SSL_MODE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssl_mode_defaults_to_preferred() {
        assert!(matches!(ssl_mode_from(None), MySqlSslMode::Preferred));
        assert!(matches!(
            ssl_mode_from(Some("nonsense")),
            MySqlSslMode::Preferred
        ));
    }

    #[test]
    fn ssl_mode_accepts_explicit_modes() {
        assert!(matches!(
            ssl_mode_from(Some("required")),
            MySqlSslMode::Required
        ));
        assert!(matches!(
            ssl_mode_from(Some(" VERIFY_CA ")),
            MySqlSslMode::VerifyCa
        ));
        assert!(matches!(
            ssl_mode_from(Some("verify_identity")),
            MySqlSslMode::VerifyIdentity
        ));
    }
}
