//! Domain types and formatting for Addict Order.
//!
//! This crate is wasm-compatible: no sqlx, no tokio, no filesystem.

pub mod auth;
pub mod catalog;
pub mod patient;
pub mod prescription;
pub mod settings;
pub mod thai;
pub mod visit;

pub use auth::{ConnectionSummary, OfficerAccount, SessionUser};
pub use catalog::{Catalog, DrugItem, LawCategory, NarcoticDrugConfig};
pub use patient::{
    PatientAddress, PatientDetail, PatientQuery, PatientSummary, mask_cid, thai_full_name,
};
pub use prescription::{
    CategoryFlags, ConfirmedPrescription, HospitalProfile, HospitalSource, OrderLine, Prescriber,
    PrescriberSource, PrescriptionDraft, PrescriptionLine, ROWS_PER_FORM, ValidationIssue,
    ValidationReport, ValidationSeverity, validate_draft,
};
pub use settings::AppSettings;
pub use thai::{
    be_year, format_qty_i64, format_thai_date, format_thai_date_slash, format_time_hm, thai_digit,
    thai_month_full,
};
pub use visit::VisitSummary;
