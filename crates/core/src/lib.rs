//! Business logic for Addict Order: classification, form mapping, validation,
//! and the repository contract.

pub mod auth;
pub mod mock;
pub mod patient;
pub mod prescription;
pub mod repository;

pub use auth::{md5_hex, verify_password};
pub use patient::{CID_LENGTH, HN_LENGTH, classify_patient_query, parse_thai_address};
pub use prescription::{FormError, build_lines, category_flags};
pub use repository::{HosxRepository, RepositoryError};
