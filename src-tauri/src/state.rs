//! Tauri-managed application state.

use std::sync::Arc;

use addict_order_hosxp_connector::MySqlPool;
use addict_order_hosxp_connector::config::CONFIG_FILE_NAME;
use addict_order_models::{AppSettings, SessionUser};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::settings::SETTINGS_FILE_NAME;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionHealth {
    Unconfigured,
    Connected,
    Disconnected,
}

#[derive(Clone)]
pub struct AppState {
    pub config_dir: std::path::PathBuf,
    pub pool: Arc<Mutex<Option<MySqlPool>>>,
    pub health: Arc<Mutex<ConnectionHealth>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub session: Arc<Mutex<Option<SessionUser>>>,
}

impl AppState {
    pub fn new(config_dir: std::path::PathBuf, settings: AppSettings) -> Self {
        Self {
            config_dir,
            pool: Arc::new(Mutex::new(None)),
            health: Arc::new(Mutex::new(ConnectionHealth::Unconfigured)),
            settings: Arc::new(Mutex::new(settings)),
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn config_path(&self) -> std::path::PathBuf {
        self.config_dir.join(CONFIG_FILE_NAME)
    }

    pub fn settings_path(&self) -> std::path::PathBuf {
        self.config_dir.join(SETTINGS_FILE_NAME)
    }
}
