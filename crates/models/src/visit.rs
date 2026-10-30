//! Visit summary types.

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

/// One OPD visit for the patient.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitSummary {
    pub vn: String,
    pub visit_date: NaiveDate,
    pub visit_time: Option<NaiveTime>,
    pub department_code: String,
    pub department_name: String,
    pub doctor_code: Option<String>,
    pub doctor_name: String,
}

impl VisitSummary {
    /// A compact label for the visit picker.
    pub fn label(&self) -> String {
        let time = self
            .visit_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_default();
        let department = if self.department_name.trim().is_empty() {
            self.department_code.as_str()
        } else {
            self.department_name.as_str()
        };
        if time.is_empty() {
            format!("{} · {}", self.visit_date, department)
        } else {
            format!("{} {} · {}", self.visit_date, time, department)
        }
    }
}
