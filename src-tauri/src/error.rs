//! The command error contract: a machine-readable kind plus a Thai message.

use addict_order_core::RepositoryError;
use addict_order_hosxp_connector::Error as ConnectorError;
use addict_order_hosxp_connector::config::ConfigError;
use serde::Serialize;

use crate::settings::SettingsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandErrorKind {
    NotConfigured,
    Connection,
    Guard,
    Query,
    Validation,
    Auth,
    Storage,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub kind: CommandErrorKind,
    pub message: String,
}

impl CommandError {
    pub fn new(kind: CommandErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn not_configured() -> Self {
        Self::new(CommandErrorKind::NotConfigured, "ยังไม่ได้ตั้งค่าการเชื่อมต่อ HOSxP")
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(
            CommandErrorKind::Validation,
            format!("ข้อมูลไม่ถูกต้อง: {}", message.into()),
        )
    }

    pub fn unauthorized() -> Self {
        Self::new(CommandErrorKind::Auth, "กรุณาเข้าสู่ระบบก่อนใช้งาน")
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(CommandErrorKind::Auth, message)
    }

    pub fn storage(message: impl Into<String>) -> Self {
        dev_log(&format!("storage failure: {}", message.into()));
        Self::new(CommandErrorKind::Storage, "บันทึกหรืออ่านการตั้งค่าไม่สำเร็จ")
    }
}

impl From<RepositoryError> for CommandError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::NotConfigured => Self::not_configured(),
            RepositoryError::Connection => Self::new(
                CommandErrorKind::Connection,
                "เชื่อมต่อฐานข้อมูล HOSxP ไม่สำเร็จ กรุณาตรวจสอบการตั้งค่าและเครือข่าย",
            ),
            RepositoryError::Query(detail) => {
                dev_log(&format!("query failed: {detail}"));
                Self::new(
                    CommandErrorKind::Query,
                    "ดึงข้อมูลจาก HOSxP ไม่สำเร็จ กรุณาลองใหม่อีกครั้ง",
                )
            }
            RepositoryError::Guard => Self::new(
                CommandErrorKind::Guard,
                "คำสั่งถูกปฏิเสธโดยระบบป้องกันการเขียนข้อมูล",
            ),
            RepositoryError::Conversion(detail) => {
                dev_log(&format!("conversion failed: {detail}"));
                Self::new(CommandErrorKind::Query, "ข้อมูลจาก HOSxP อยู่ในรูปแบบที่ไม่รองรับ")
            }
        }
    }
}

impl From<ConnectorError> for CommandError {
    fn from(error: ConnectorError) -> Self {
        match error {
            ConnectorError::Config(inner) => inner.into(),
            ConnectorError::Connect(inner) => {
                dev_log(&format!("connect failed: {inner}"));
                Self::new(
                    CommandErrorKind::Connection,
                    "เชื่อมต่อฐานข้อมูล HOSxP ไม่สำเร็จ กรุณาตรวจสอบการตั้งค่าและเครือข่าย",
                )
            }
            ConnectorError::Database(inner) => {
                dev_log(&format!("database error: {inner}"));
                Self::new(
                    CommandErrorKind::Query,
                    "ดึงข้อมูลจาก HOSxP ไม่สำเร็จ กรุณาลองใหม่อีกครั้ง",
                )
            }
            ConnectorError::Guard => Self::new(
                CommandErrorKind::Guard,
                "คำสั่งถูกปฏิเสธโดยระบบป้องกันการเขียนข้อมูล",
            ),
        }
    }
}

impl From<ConfigError> for CommandError {
    fn from(error: ConfigError) -> Self {
        Self::storage(error.to_string())
    }
}

impl From<SettingsError> for CommandError {
    fn from(error: SettingsError) -> Self {
        Self::storage(error.to_string())
    }
}

pub(crate) fn dev_log(message: &str) {
    if cfg!(debug_assertions) {
        eprintln!("[addict-order] {message}");
    } else {
        let _ = message;
    }
}
