//! Operator-configurable settings shared by the backend and the frontend.
//! Credentials are not here; they live in the encrypted connection file.

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::prescription::HospitalProfile;

/// Everything the operator configures besides the connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    /// The printed hospital header. This is the primary print source; the
    /// database profile is a prefill and may be overridden here.
    pub hospital: HospitalProfile,
    /// The pharmacist-managed narcotic catalog.
    pub catalog: Catalog,
}
