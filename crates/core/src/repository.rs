//! The repository contract between business logic and the HOSxP connector.
//!
//! The connector implements this trait. The default test suite uses a mock, so
//! it never needs a live database.

use addict_order_models::{
    DrugItem, HospitalSource, OfficerAccount, OrderLine, PatientDetail, PatientQuery,
    PatientSummary, Prescriber, VisitSummary,
};
use async_trait::async_trait;
use chrono::NaiveDate;

/// Errors crossing the repository boundary. English and developer-facing;
/// Thai user-facing translation happens only in `src-tauri/commands.rs`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepositoryError {
    #[error("HOSxP connection is not configured")]
    NotConfigured,
    #[error("cannot reach the HOSxP database")]
    Connection,
    #[error("query failed: {0}")]
    Query(String),
    #[error("read-only guard rejected a statement")]
    Guard,
    #[error("data conversion failed: {0}")]
    Conversion(String),
}

/// Read-only access to the HOSxP data Addict Order needs.
#[async_trait]
pub trait HosxRepository: Send + Sync {
    async fn ping(&self) -> Result<(), RepositoryError>;

    async fn fetch_officer(
        &self,
        login_name: &str,
    ) -> Result<Option<OfficerAccount>, RepositoryError>;

    async fn search_patients(
        &self,
        query: &PatientQuery,
    ) -> Result<Vec<PatientSummary>, RepositoryError>;

    async fn fetch_patient(&self, hn: &str) -> Result<Option<PatientDetail>, RepositoryError>;

    async fn fetch_visits(
        &self,
        hn: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<VisitSummary>, RepositoryError>;

    async fn fetch_narcotic_orders(
        &self,
        vn: &str,
        icodes: &[String],
    ) -> Result<Vec<OrderLine>, RepositoryError>;

    async fn search_drugs(&self, term: &str) -> Result<Vec<DrugItem>, RepositoryError>;

    async fn search_doctors(&self, term: &str) -> Result<Vec<Prescriber>, RepositoryError>;

    /// Optional database hospital profile. The settings profile is the print
    /// source; this is a prefill and a cross-check.
    async fn fetch_hospital_source(&self) -> Result<Option<HospitalSource>, RepositoryError>;
}
