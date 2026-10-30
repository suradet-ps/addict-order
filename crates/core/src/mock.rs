//! In-memory repository for the DB-free test suite.

use std::collections::HashMap;

use addict_order_models::{
    DrugItem, HospitalSource, OfficerAccount, OrderLine, PatientDetail, PatientQuery,
    PatientSummary, Prescriber, VisitSummary,
};
use async_trait::async_trait;
use chrono::NaiveDate;

use crate::repository::{HosxRepository, RepositoryError};

/// A simple in-memory repository. Tests can seed only the data they need.
#[derive(Debug, Clone, Default)]
pub struct MockRepository {
    pub officers: Vec<OfficerAccount>,
    pub patients: Vec<PatientSummary>,
    pub details: HashMap<String, PatientDetail>,
    pub visits: Vec<VisitSummary>,
    pub orders: Vec<OrderLine>,
    pub drugs: Vec<DrugItem>,
    pub doctors: Vec<Prescriber>,
    pub hospital: Option<HospitalSource>,
}

#[async_trait]
impl HosxRepository for MockRepository {
    async fn ping(&self) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn fetch_officer(
        &self,
        login_name: &str,
    ) -> Result<Option<OfficerAccount>, RepositoryError> {
        Ok(self
            .officers
            .iter()
            .find(|officer| officer.login_name == login_name)
            .cloned())
    }

    async fn search_patients(
        &self,
        query: &PatientQuery,
    ) -> Result<Vec<PatientSummary>, RepositoryError> {
        let matches = |patient: &&PatientSummary| match query {
            PatientQuery::Hn(hn) => &patient.hn == hn,
            PatientQuery::Cid(cid) => patient.cid.as_deref() == Some(cid.as_str()),
            PatientQuery::NamePrefix(term) => patient.full_name_th.starts_with(term),
            PatientQuery::NameContains(term) => patient.full_name_th.contains(term),
        };
        Ok(self
            .patients
            .iter()
            .filter(matches)
            .take(20)
            .cloned()
            .collect())
    }

    async fn fetch_patient(&self, hn: &str) -> Result<Option<PatientDetail>, RepositoryError> {
        Ok(self.details.get(hn).cloned())
    }

    async fn fetch_visits(
        &self,
        hn: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<VisitSummary>, RepositoryError> {
        let _ = hn;
        Ok(self
            .visits
            .iter()
            .filter(|visit| visit.visit_date >= from && visit.visit_date <= to)
            .cloned()
            .collect())
    }

    async fn fetch_narcotic_orders(
        &self,
        vn: &str,
        icodes: &[String],
    ) -> Result<Vec<OrderLine>, RepositoryError> {
        Ok(self
            .orders
            .iter()
            .filter(|order| order.vn == vn && icodes.contains(&order.icode))
            .cloned()
            .collect())
    }

    async fn search_drugs(&self, term: &str) -> Result<Vec<DrugItem>, RepositoryError> {
        let term = term.to_lowercase();
        Ok(self
            .drugs
            .iter()
            .filter(|drug| {
                drug.icode.to_lowercase().contains(&term)
                    || drug.name.to_lowercase().contains(&term)
                    || drug.trade_name.to_lowercase().contains(&term)
            })
            .take(30)
            .cloned()
            .collect())
    }

    async fn search_doctors(&self, term: &str) -> Result<Vec<Prescriber>, RepositoryError> {
        let term = term.to_lowercase();
        Ok(self
            .doctors
            .iter()
            .filter(|doctor| {
                doctor.name.to_lowercase().contains(&term)
                    || doctor
                        .doctor_code
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&term)
            })
            .take(20)
            .cloned()
            .collect())
    }

    async fn fetch_hospital_source(&self) -> Result<Option<HospitalSource>, RepositoryError> {
        Ok(self.hospital.clone())
    }
}
