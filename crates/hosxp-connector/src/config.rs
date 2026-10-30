//! Encrypted storage of HOSxP connection settings.
//!
//! Every field is encrypted at rest with AES-256-GCM via `encryptman`; the
//! master key lives in the OS keychain through `encryptman-keyring`.

use std::fmt;
use std::path::Path;

use encryptman::{MasterKey, decrypt, encrypt};
use secrecy::{ExposeSecret, SecretBox, SecretString};
use serde::{Deserialize, Serialize};

/// Keychain service name and HKDF context for Addict Order.
pub const SERVICE_NAME: &str = "addict-order";

/// File name of the encrypted connection file inside the app config dir.
pub const CONFIG_FILE_NAME: &str = "addict-order.connection.json";

const CONFIG_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file I/O failed: {0}")]
    Io(String),
    #[error("config file is corrupt: {0}")]
    Corrupt(String),
    #[error("decryption of connection settings failed: {0}")]
    Decrypt(String),
    #[error("encryption of connection settings failed: {0}")]
    Encrypt(String),
    #[error("OS keychain access failed: {0}")]
    Keyring(String),
}

/// Encrypts and decrypts connection fields.
pub trait KeyStore: Send + Sync {
    fn encrypt(&self, plaintext: &str) -> Result<String, ConfigError>;
    fn decrypt(&self, ciphertext: &str) -> Result<String, ConfigError>;
}

/// Production key store backed by the OS keychain.
#[derive(Debug)]
pub struct VaultKeyStore(pub encryptman_keyring::Vault);

impl KeyStore for VaultKeyStore {
    fn encrypt(&self, plaintext: &str) -> Result<String, ConfigError> {
        self.0
            .encrypt(plaintext)
            .map_err(|e| ConfigError::Encrypt(e.to_string()))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<String, ConfigError> {
        self.0
            .decrypt(ciphertext)
            .map_err(|e| ConfigError::Decrypt(e.to_string()))
    }
}

pub fn load_vault() -> Result<VaultKeyStore, ConfigError> {
    encryptman_keyring::Vault::new(SERVICE_NAME)
        .map(VaultKeyStore)
        .map_err(|e| ConfigError::Keyring(e.to_string()))
}

/// Test key store from an in-memory master key.
#[derive(Debug)]
pub struct MasterKeyStore(pub MasterKey);

impl KeyStore for MasterKeyStore {
    fn encrypt(&self, plaintext: &str) -> Result<String, ConfigError> {
        encrypt(&self.0, plaintext).map_err(|e| ConfigError::Encrypt(e.to_string()))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<String, ConfigError> {
        decrypt(&self.0, ciphertext).map_err(|e| ConfigError::Decrypt(e.to_string()))
    }
}

/// HOSxP connection settings. Plaintext in memory only.
#[derive(Clone)]
pub struct HosxConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: SecretString,
}

impl HosxConfig {
    pub fn new(
        host: String,
        port: u16,
        database: String,
        user: String,
        password: SecretString,
    ) -> Self {
        Self {
            host,
            port,
            database,
            user,
            password,
        }
    }
}

impl PartialEq for HosxConfig {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
            && self.port == other.port
            && self.database == other.database
            && self.user == other.user
            && self.password.expose_secret() == other.password.expose_secret()
    }
}

impl fmt::Debug for HosxConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HosxConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("user", &self.user)
            .field("password", &"***")
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    version: u32,
    hosx: EncryptedHosxFields,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedHosxFields {
    host: String,
    port: String,
    database: String,
    user: String,
    password: String,
}

pub async fn save_encrypted(
    config_path: &Path,
    store: &dyn KeyStore,
    cfg: &HosxConfig,
) -> Result<(), ConfigError> {
    let file = ConfigFile {
        version: CONFIG_VERSION,
        hosx: EncryptedHosxFields {
            host: store.encrypt(&cfg.host)?,
            port: store.encrypt(&cfg.port.to_string())?,
            database: store.encrypt(&cfg.database)?,
            user: store.encrypt(&cfg.user)?,
            password: store.encrypt(cfg.password.expose_secret())?,
        },
    };
    let json =
        serde_json::to_string_pretty(&file).map_err(|e| ConfigError::Corrupt(e.to_string()))?;
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| ConfigError::Io(e.to_string()))?;
    }
    tokio::fs::write(config_path, json)
        .await
        .map_err(|e| ConfigError::Io(e.to_string()))
}

pub async fn load(
    config_path: &Path,
    store: &dyn KeyStore,
) -> Result<Option<HosxConfig>, ConfigError> {
    if !config_path.exists() {
        return Ok(None);
    }
    let raw = tokio::fs::read_to_string(config_path)
        .await
        .map_err(|e| ConfigError::Io(e.to_string()))?;
    let file: ConfigFile =
        serde_json::from_str(&raw).map_err(|e| ConfigError::Corrupt(e.to_string()))?;
    if file.version != CONFIG_VERSION {
        return Err(ConfigError::Corrupt(format!(
            "unsupported config version {}",
            file.version
        )));
    }
    let fields = file.hosx;
    let port = store
        .decrypt(&fields.port)?
        .parse()
        .map_err(|_| ConfigError::Decrypt("stored port is not a number".into()))?;
    Ok(Some(HosxConfig {
        host: store.decrypt(&fields.host)?,
        port,
        database: store.decrypt(&fields.database)?,
        user: store.decrypt(&fields.user)?,
        password: SecretBox::from(store.decrypt(&fields.password)?),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use encryptman::generate_master_key;
    use tempfile::tempdir;

    fn test_store() -> MasterKeyStore {
        MasterKeyStore(generate_master_key().expect("os RNG available during tests"))
    }

    fn sample_config() -> HosxConfig {
        HosxConfig::new(
            "10.0.0.5".into(),
            3306,
            "hos".into(),
            "addict_order_ro".into(),
            SecretBox::from("s3cret!p@ss".to_string()),
        )
    }

    #[tokio::test]
    async fn round_trip_encrypts_and_decrypts_all_fields() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        let store = test_store();
        save_encrypted(&path, &store, &sample_config())
            .await
            .expect("save succeeds");
        let loaded = load(&path, &store).await.expect("load succeeds");
        assert_eq!(loaded, Some(sample_config()));
    }

    #[tokio::test]
    async fn missing_file_returns_none() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        assert_eq!(
            load(&path, &test_store()).await.expect("load succeeds"),
            None
        );
    }

    #[tokio::test]
    async fn corrupt_file_returns_error() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        tokio::fs::write(&path, "not json at all")
            .await
            .expect("write in test");
        assert!(load(&path, &test_store()).await.is_err());
    }

    #[tokio::test]
    async fn wrong_key_fails_to_decrypt() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        save_encrypted(&path, &test_store(), &sample_config())
            .await
            .expect("save succeeds");
        assert!(matches!(
            load(&path, &test_store()).await,
            Err(ConfigError::Decrypt(_))
        ));
    }

    #[tokio::test]
    async fn stored_file_contains_no_plaintext() {
        let dir = tempdir().expect("tempdir in test");
        let path = dir.path().join(CONFIG_FILE_NAME);
        save_encrypted(&path, &test_store(), &sample_config())
            .await
            .expect("save succeeds");
        let raw = tokio::fs::read_to_string(&path)
            .await
            .expect("read in test");
        for secret in ["s3cret!p@ss", "addict_order_ro", "10.0.0.5"] {
            assert!(
                !raw.contains(secret),
                "plaintext {secret:?} must not appear on disk"
            );
        }
    }

    #[test]
    fn debug_impl_masks_password() {
        let debug = format!("{:?}", sample_config());
        assert!(!debug.contains("s3cret!p@ss"));
        assert!(debug.contains("***"));
    }
}
