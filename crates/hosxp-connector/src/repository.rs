//! [`HosxRepository`] implementation over the read-only pool.

use addict_order_core::{HosxRepository, RepositoryError, parse_thai_address};
use addict_order_models::{
    DrugItem, HospitalSource, OfficerAccount, OrderLine, PatientAddress, PatientDetail,
    PatientQuery, PatientSummary, Prescriber, PrescriberSource, VisitSummary,
};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use sqlx::Row;
use sqlx::mysql::{MySqlPool, MySqlRow};

use crate::error::Error;
use crate::queries::{
    DOCTOR_SEARCH_SQL, DRUG_SEARCH_SQL, FETCH_OFFICER_SQL, HOSPITAL_PROFILE_SQL, PATIENT_BY_HN_SQL,
    PATIENT_SEARCH_BY_CID, PATIENT_SEARCH_BY_HN, PATIENT_SEARCH_NAME_CONTAINS,
    PATIENT_SEARCH_NAME_PREFIX, PING_SQL, VISITS_BY_HN_SQL, narcotic_orders_sql,
};
use crate::readonly_guard::assert_read_only;

#[derive(Debug, Clone)]
pub struct ConnectorRepository {
    pool: MySqlPool,
}

impl ConnectorRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    fn guard(sql: &str) -> Result<(), RepositoryError> {
        assert_read_only(sql).map_err(|_| Error::Guard.into())
    }
}

fn escape_like(term: &str) -> String {
    let mut escaped = String::with_capacity(term.len());
    for c in term.chars() {
        if matches!(c, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

fn get_string(row: &MySqlRow, names: &[&str]) -> String {
    for name in names {
        if let Ok(value) = row.try_get::<Option<String>, _>(*name) {
            return value.unwrap_or_default();
        }
    }
    String::new()
}

fn get_optional_string(row: &MySqlRow, name: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(name)
        .ok()
        .flatten()
        .filter(|value| !value.trim().is_empty())
}

#[async_trait]
impl HosxRepository for ConnectorRepository {
    async fn ping(&self) -> Result<(), RepositoryError> {
        Self::guard(PING_SQL)?;
        sqlx::query(PING_SQL)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| Error::Database(error).into())
    }

    async fn fetch_officer(
        &self,
        login_name: &str,
    ) -> Result<Option<OfficerAccount>, RepositoryError> {
        Self::guard(FETCH_OFFICER_SQL)?;
        let row = sqlx::query(FETCH_OFFICER_SQL)
            .bind(login_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Database)?;
        row.map(|row| -> Result<OfficerAccount, Error> {
            Ok(OfficerAccount {
                officer_id: row.try_get("officer_id").map_err(Error::Database)?,
                login_name: row.try_get("login_name").map_err(Error::Database)?,
                password_md5: row.try_get("password_md5").map_err(Error::Database)?,
                active: row.try_get("active").map_err(Error::Database)?,
                auth_type_id: row.try_get("auth_type_id").map_err(Error::Database)?,
                doctor_code: row.try_get("doctor_code").map_err(Error::Database)?,
                display_name: row.try_get("display_name").map_err(Error::Database)?,
            })
        })
        .transpose()
        .map_err(Into::into)
    }

    async fn search_patients(
        &self,
        query: &PatientQuery,
    ) -> Result<Vec<PatientSummary>, RepositoryError> {
        match query {
            PatientQuery::Hn(hn) => {
                Self::guard(PATIENT_SEARCH_BY_HN)?;
                let rows = sqlx::query(PATIENT_SEARCH_BY_HN)
                    .bind(hn)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(Error::Database)?;
                rows.into_iter().map(map_patient_summary).collect()
            }
            PatientQuery::Cid(cid) => {
                Self::guard(PATIENT_SEARCH_BY_CID)?;
                let rows = sqlx::query(PATIENT_SEARCH_BY_CID)
                    .bind(cid)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(Error::Database)?;
                rows.into_iter().map(map_patient_summary).collect()
            }
            PatientQuery::NamePrefix(term) => {
                let pattern = format!("{}%", escape_like(term.trim()));
                Self::guard(PATIENT_SEARCH_NAME_PREFIX)?;
                let rows = sqlx::query(PATIENT_SEARCH_NAME_PREFIX)
                    .bind(&pattern)
                    .bind(&pattern)
                    .bind(&pattern)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(Error::Database)?;
                if rows.is_empty() {
                    self.search_patients_contains(term).await
                } else {
                    rows.into_iter().map(map_patient_summary).collect()
                }
            }
            PatientQuery::NameContains(term) => self.search_patients_contains(term).await,
        }
    }

    async fn fetch_patient(&self, hn: &str) -> Result<Option<PatientDetail>, RepositoryError> {
        Self::guard(PATIENT_BY_HN_SQL)?;
        let row = sqlx::query(PATIENT_BY_HN_SQL)
            .bind(hn)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Database)?;
        row.map(|row| -> Result<PatientDetail, Error> {
            let address_hint = get_string(&row, &["address_hint"]);
            let (parsed_subdistrict, parsed_district, parsed_province) =
                parse_thai_address(&address_hint);
            let subdistrict = {
                let value = get_string(&row, &["subdistrict"]);
                if value.trim().is_empty() {
                    parsed_subdistrict
                } else {
                    value
                }
            };
            Ok(PatientDetail {
                hn: row.try_get("hn").map_err(Error::Database)?,
                cid: row.try_get("cid").map_err(Error::Database)?,
                full_name_th: get_string(&row, &["full_name_th"]),
                birth_date: row.try_get("birthday").map_err(Error::Database)?,
                sex: row.try_get("sex").map_err(Error::Database)?,
                phone: get_string(&row, &["phone"]),
                address: PatientAddress {
                    house_no: get_string(&row, &["house_no"]),
                    moo: get_string(&row, &["moo"]),
                    soi: get_string(&row, &["soi"]),
                    road: get_string(&row, &["road"]),
                    subdistrict,
                    district: parsed_district,
                    province: parsed_province,
                    postal_code: get_string(&row, &["postal_code"]),
                },
                address_hint,
            })
        })
        .transpose()
        .map_err(Into::into)
    }

    async fn fetch_visits(
        &self,
        hn: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<VisitSummary>, RepositoryError> {
        Self::guard(VISITS_BY_HN_SQL)?;
        let rows = sqlx::query(VISITS_BY_HN_SQL)
            .bind(hn)
            .bind(from)
            .bind(to)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::Database)?;
        rows.into_iter()
            .map(|row| -> Result<VisitSummary, Error> {
                Ok(VisitSummary {
                    vn: row.try_get("vn").map_err(Error::Database)?,
                    visit_date: row.try_get("vstdate").map_err(Error::Database)?,
                    visit_time: row
                        .try_get::<Option<NaiveTime>, _>("vsttime")
                        .map_err(Error::Database)?,
                    department_code: get_string(&row, &["department_code"]),
                    department_name: get_string(&row, &["department_name"]),
                    doctor_code: get_optional_string(&row, "doctor_code"),
                    doctor_name: get_string(&row, &["doctor_name"]),
                })
            })
            .collect::<Result<Vec<_>, Error>>()
            .map_err(Into::into)
    }

    async fn fetch_narcotic_orders(
        &self,
        vn: &str,
        icodes: &[String],
    ) -> Result<Vec<OrderLine>, RepositoryError> {
        if icodes.is_empty() {
            return Ok(Vec::new());
        }
        let sql = narcotic_orders_sql(icodes.len());
        Self::guard(&sql)?;
        let mut query = sqlx::query(&sql).bind(vn);
        for icode in icodes {
            query = query.bind(icode);
        }
        let rows = query.fetch_all(&self.pool).await.map_err(Error::Database)?;
        rows.into_iter()
            .map(|row| -> Result<OrderLine, Error> {
                let quantity = row.try_get::<i64, _>("qty").map_err(Error::Database)?;
                Ok(OrderLine {
                    vn: row.try_get("vn").map_err(Error::Database)?,
                    hn: row.try_get("hn").map_err(Error::Database)?,
                    icode: row.try_get("icode").map_err(Error::Database)?,
                    item_no: row
                        .try_get::<Option<i64>, _>("item_no")
                        .map_err(Error::Database)?,
                    hos_guid: row.try_get("hos_guid").map_err(Error::Database)?,
                    quantity,
                    drugusage_code: get_optional_string(&row, "drugusage_code"),
                    sp_use_code: get_optional_string(&row, "sp_use_code"),
                    sig_text: get_string(&row, &["sig_text"]),
                    doctor_code: get_optional_string(&row, "doctor_code"),
                    doctor_name: get_string(&row, &["doctor_name"]),
                    doctor_license_no: get_optional_string(&row, "doctor_license_no"),
                    generic_name: get_string(&row, &["generic_name"]),
                    trade_name: get_string(&row, &["trade_name"]),
                    strength: get_string(&row, &["strength"]),
                    source_units: get_string(&row, &["source_units"]),
                    order_date: row
                        .try_get::<Option<NaiveDate>, _>("rxdate")
                        .map_err(Error::Database)?,
                    order_time: row
                        .try_get::<Option<NaiveTime>, _>("rxtime")
                        .map_err(Error::Database)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()
            .map_err(Into::into)
    }

    async fn search_drugs(&self, term: &str) -> Result<Vec<DrugItem>, RepositoryError> {
        Self::guard(DRUG_SEARCH_SQL)?;
        let pattern = format!("%{}%", escape_like(term.trim()));
        let rows = sqlx::query(DRUG_SEARCH_SQL)
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::Database)?;
        rows.into_iter()
            .map(|row| -> Result<DrugItem, Error> {
                Ok(DrugItem {
                    icode: row.try_get("icode").map_err(Error::Database)?,
                    name: get_string(&row, &["name"]),
                    trade_name: get_string(&row, &["trade_name"]),
                    strength: get_string(&row, &["strength"]),
                    units: get_string(&row, &["units"]),
                    dosageform: get_string(&row, &["dosageform"]),
                    drug_control_type_id: get_optional_string(&row, "drug_control_type_id"),
                })
            })
            .collect::<Result<Vec<_>, Error>>()
            .map_err(Into::into)
    }

    async fn search_doctors(&self, term: &str) -> Result<Vec<Prescriber>, RepositoryError> {
        Self::guard(DOCTOR_SEARCH_SQL)?;
        let pattern = format!("%{}%", escape_like(term.trim()));
        let rows = sqlx::query(DOCTOR_SEARCH_SQL)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::Database)?;
        rows.into_iter()
            .map(|row| -> Result<Prescriber, Error> {
                Ok(Prescriber {
                    doctor_code: Some(row.try_get("code").map_err(Error::Database)?),
                    name: get_string(&row, &["name"]),
                    license_no: get_string(&row, &["licenseno"]),
                    source: Some(PrescriberSource::Hosxp),
                })
            })
            .collect::<Result<Vec<_>, Error>>()
            .map_err(Into::into)
    }

    async fn fetch_hospital_source(&self) -> Result<Option<HospitalSource>, RepositoryError> {
        Self::guard(HOSPITAL_PROFILE_SQL)?;
        let row = sqlx::query(HOSPITAL_PROFILE_SQL)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::Database)?;
        Ok(row.map(|row| HospitalSource {
            name: get_string(&row, &["name", "hospital_name", "hospitalname"]),
            phone: get_string(&row, &["phone", "tel"]),
            fax: get_string(&row, &["fax"]),
            address_text: get_string(&row, &["address1", "address", "address_text"]),
        }))
    }
}

impl ConnectorRepository {
    async fn search_patients_contains(
        &self,
        term: &str,
    ) -> Result<Vec<PatientSummary>, RepositoryError> {
        Self::guard(PATIENT_SEARCH_NAME_CONTAINS)?;
        let pattern = format!("%{}%", escape_like(term.trim()));
        let rows = sqlx::query(PATIENT_SEARCH_NAME_CONTAINS)
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::Database)?;
        rows.into_iter().map(map_patient_summary).collect()
    }
}

fn map_patient_summary(row: MySqlRow) -> Result<PatientSummary, RepositoryError> {
    Ok(PatientSummary {
        hn: row.try_get("hn").map_err(Error::Database)?,
        cid: row.try_get("cid").map_err(Error::Database)?,
        full_name_th: get_string(&row, &["full_name_th"]),
        birth_date: row
            .try_get::<Option<NaiveDate>, _>("birthday")
            .map_err(Error::Database)?,
        sex: row
            .try_get::<Option<String>, _>("sex")
            .map_err(Error::Database)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_like_escapes_wildcards_and_backslash() {
        assert_eq!(escape_like("para"), "para");
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("c:\\x"), "c:\\\\x");
        assert_eq!(escape_like("พารา"), "พารา");
    }
}
