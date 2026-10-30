//! Plain application settings persistence. No credentials and no patient data.

use std::path::Path;

pub use addict_order_models::AppSettings;
use serde::{Deserialize, Serialize};

pub const SETTINGS_FILE_NAME: &str = "addict-order.settings.json";
const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("settings file I/O failed: {0}")]
    Io(String),
    #[error("settings file is corrupt: {0}")]
    Corrupt(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct SettingsFile {
    version: u32,
    #[serde(flatten)]
    settings: AppSettings,
}

pub async fn load(path: &Path) -> Result<AppSettings, SettingsError> {
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let raw = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| SettingsError::Io(e.to_string()))?;
    let file: SettingsFile =
        serde_json::from_str(&raw).map_err(|e| SettingsError::Corrupt(e.to_string()))?;
    if file.version == SETTINGS_VERSION {
        Ok(file.settings)
    } else {
        Err(SettingsError::Corrupt(format!(
            "unsupported settings version {}",
            file.version
        )))
    }
}

pub async fn save(path: &Path, settings: &AppSettings) -> Result<(), SettingsError> {
    let file = SettingsFile {
        version: SETTINGS_VERSION,
        settings: settings.clone(),
    };
    let json =
        serde_json::to_string_pretty(&file).map_err(|e| SettingsError::Corrupt(e.to_string()))?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| SettingsError::Io(e.to_string()))?;
    }
    tokio::fs::write(path, json)
        .await
        .map_err(|e| SettingsError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use addict_order_models::{Catalog, HospitalProfile};
    use tempfile::tempdir;

    #[tokio::test]
    async fn missing_file_returns_defaults() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(SETTINGS_FILE_NAME);
        let settings = load(&path).await.expect("load");
        assert_eq!(settings, AppSettings::default());
    }

    #[tokio::test]
    async fn round_trip_preserves_settings() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(SETTINGS_FILE_NAME);
        let settings = AppSettings {
            hospital: HospitalProfile {
                name: "โรงพยาบาลทดสอบ".into(),
                ..HospitalProfile::default()
            },
            catalog: Catalog::default(),
        };
        save(&path, &settings).await.expect("save");
        let loaded = load(&path).await.expect("load");
        assert_eq!(loaded, settings);
    }

    #[tokio::test]
    async fn corrupt_file_is_rejected() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(SETTINGS_FILE_NAME);
        tokio::fs::write(&path, "{ not json").await.expect("write");
        assert!(matches!(load(&path).await, Err(SettingsError::Corrupt(_))));
    }

    #[tokio::test]
    async fn saved_file_contains_no_credentials() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(SETTINGS_FILE_NAME);
        save(&path, &AppSettings::default()).await.expect("save");
        let raw = tokio::fs::read_to_string(&path).await.expect("read");
        for forbidden in ["password", "host", "user"] {
            assert!(!raw.contains(forbidden));
        }
    }
}
