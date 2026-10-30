//! Authentication types shared by the connector, the Tauri boundary, and the
//! login screen. The password hash never crosses the IPC boundary.

use std::fmt;

use serde::{Deserialize, Serialize};

/// One `officer` row as read from HOSxP. Backend-only.
#[derive(Clone, PartialEq, Eq)]
pub struct OfficerAccount {
    pub officer_id: i64,
    pub login_name: String,
    pub password_md5: String,
    pub active: String,
    pub auth_type_id: i64,
    pub doctor_code: Option<String>,
    pub display_name: String,
}

impl fmt::Debug for OfficerAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OfficerAccount")
            .field("officer_id", &self.officer_id)
            .field("login_name", &self.login_name)
            .field("password_md5", &"<redacted>")
            .field("active", &self.active)
            .field("auth_type_id", &self.auth_type_id)
            .field("doctor_code", &self.doctor_code)
            .field("display_name", &self.display_name)
            .finish()
    }
}

/// The sanitized user returned to the frontend after a successful login.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUser {
    pub officer_id: i64,
    pub login_name: String,
    pub display_name: String,
    pub doctor_code: Option<String>,
    pub role: String,
}

/// Non-secret connection fields shown in the settings dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSummary {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
}
