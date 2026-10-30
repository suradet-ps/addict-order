//! Prescription domain types and validation.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::catalog::LawCategory;
use crate::patient::{PatientAddress, PatientDetail};
use crate::visit::VisitSummary;

/// The official form has five drug rows in v1.
pub const ROWS_PER_FORM: usize = 5;

/// One raw order line read from HOSxP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderLine {
    pub vn: String,
    pub hn: String,
    pub icode: String,
    pub item_no: Option<i64>,
    pub hos_guid: String,
    pub quantity: i64,
    #[serde(default)]
    pub drugusage_code: Option<String>,
    #[serde(default)]
    pub sp_use_code: Option<String>,
    #[serde(default)]
    pub sig_text: String,
    #[serde(default)]
    pub doctor_code: Option<String>,
    #[serde(default)]
    pub doctor_name: String,
    #[serde(default)]
    pub doctor_license_no: Option<String>,
    #[serde(default)]
    pub generic_name: String,
    #[serde(default)]
    pub trade_name: String,
    #[serde(default)]
    pub strength: String,
    #[serde(default)]
    pub source_units: String,
    #[serde(default)]
    pub order_date: Option<NaiveDate>,
    #[serde(default)]
    pub order_time: Option<NaiveTime>,
}

/// One printable line after applying the catalog metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionLine {
    pub line_id: String,
    pub icode: String,
    pub generic_name: String,
    #[serde(default)]
    pub trade_name: String,
    pub strength_text: String,
    pub sig: String,
    pub quantity: i64,
    pub print_unit: String,
    pub law_category: LawCategory,
    pub source_vn: String,
    #[serde(default)]
    pub source_item_no: Option<i64>,
    #[serde(default)]
    pub source_doctor_code: Option<String>,
}

/// Where the prescriber identity came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrescriberSource {
    Hosxp,
    Manual,
}

/// The licensed prescriber printed on the official form.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Prescriber {
    pub doctor_code: Option<String>,
    pub name: String,
    pub license_no: String,
    pub source: Option<PrescriberSource>,
}

impl Prescriber {
    pub fn is_print_ready(&self) -> bool {
        !self.name.trim().is_empty() && !self.license_no.trim().is_empty()
    }
}

/// The hospital block printed on the official form.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HospitalProfile {
    pub name: String,
    pub house_no: String,
    pub moo: String,
    pub soi: String,
    pub road: String,
    pub subdistrict: String,
    pub district: String,
    pub province: String,
    pub postal_code: String,
    pub phone: String,
    pub fax: String,
}

impl HospitalProfile {
    pub fn is_print_ready(&self) -> bool {
        !self.name.trim().is_empty()
    }

    pub fn address_is_complete(&self) -> bool {
        !self.house_no.trim().is_empty()
            && !self.subdistrict.trim().is_empty()
            && !self.district.trim().is_empty()
            && !self.province.trim().is_empty()
    }

    pub fn address(&self) -> PatientAddress {
        PatientAddress {
            house_no: self.house_no.clone(),
            moo: self.moo.clone(),
            soi: self.soi.clone(),
            road: self.road.clone(),
            subdistrict: self.subdistrict.clone(),
            district: self.district.clone(),
            province: self.province.clone(),
            postal_code: self.postal_code.clone(),
        }
    }
}

/// A database hospital profile used as a settings prefill. The printed
/// profile remains the operator-edited [`HospitalProfile`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HospitalSource {
    pub name: String,
    pub phone: String,
    pub fax: String,
    pub address_text: String,
}

/// The two legal-category checkboxes on the official form.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryFlags {
    pub narcotic_2: bool,
    pub psychotropic_2: bool,
}

impl CategoryFlags {
    pub fn from_lines(lines: &[PrescriptionLine]) -> Self {
        let mut flags = Self::default();
        for line in lines {
            match line.law_category {
                LawCategory::Narcotic2 => flags.narcotic_2 = true,
                LawCategory::Psychotropic2 => flags.psychotropic_2 = true,
            }
        }
        flags
    }

    pub fn any(self) -> bool {
        self.narcotic_2 || self.psychotropic_2
    }
}

/// The draft prescription shown on screen and printed after confirmation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionDraft {
    pub patient: PatientDetail,
    pub visit: VisitSummary,
    pub prescriber: Prescriber,
    pub hospital: HospitalProfile,
    pub prescription_date: NaiveDate,
    pub lines: Vec<PrescriptionLine>,
    pub category_flags: CategoryFlags,
}

impl PrescriptionDraft {
    pub fn row_count(&self) -> usize {
        self.lines.len()
    }

    pub fn snapshot(&self, confirmed_at: NaiveDateTime) -> ConfirmedPrescription {
        ConfirmedPrescription {
            draft: self.clone(),
            confirmed_at,
        }
    }
}

/// An immutable snapshot captured before printing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedPrescription {
    pub draft: PrescriptionDraft,
    pub confirmed_at: NaiveDateTime,
}

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

/// One validation issue shown in the inspector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub field: String,
    pub message: String,
}

impl ValidationIssue {
    pub fn error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn warning(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            field: field.into(),
            message: message.into(),
        }
    }
}

/// The full validation result.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == ValidationSeverity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == ValidationSeverity::Warning)
    }

    pub fn is_valid(&self) -> bool {
        self.errors().next().is_none()
    }

    pub fn error_count(&self) -> usize {
        self.errors().count()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings().count()
    }
}

/// Validates a draft against the official form requirements.
pub fn validate_draft(draft: &PrescriptionDraft) -> ValidationReport {
    let mut issues = Vec::new();

    if draft.patient.full_name_th.trim().is_empty() {
        issues.push(ValidationIssue::error("patient.name", "ไม่พบชื่อผู้รับการรักษา"));
    }
    if draft
        .patient
        .cid
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        issues.push(ValidationIssue::error(
            "patient.cid",
            "ไม่พบเลขบัตรประชาชนหรือเลขบัตรราชการอื่น",
        ));
    }
    if !draft.patient.address.is_complete() {
        issues.push(ValidationIssue::warning(
            "patient.address",
            "ที่อยู่ผู้ป่วยไม่ครบ กรุณาตรวจสอบก่อนพิมพ์",
        ));
    }
    if draft.patient.phone.trim().is_empty() {
        issues.push(ValidationIssue::warning("patient.phone", "ไม่พบโทรศัพท์ผู้ป่วย"));
    }

    if draft.prescriber.name.trim().is_empty() {
        issues.push(ValidationIssue::error(
            "prescriber.name",
            "ไม่พบชื่อผู้ออกใบสั่งจ่าย",
        ));
    }
    if draft.prescriber.license_no.trim().is_empty() {
        issues.push(ValidationIssue::error(
            "prescriber.license",
            "ไม่พบเลขใบประกอบวิชาชีพของผู้ออกใบสั่งจ่าย",
        ));
    }

    if draft.hospital.name.trim().is_empty() {
        issues.push(ValidationIssue::error("hospital.name", "ไม่พบชื่อสถานพยาบาล"));
    }
    if !draft.hospital.address_is_complete() {
        issues.push(ValidationIssue::warning(
            "hospital.address",
            "ที่อยู่สถานพยาบาลไม่ครบ กรุณาตรวจสอบก่อนพิมพ์",
        ));
    }
    if draft.hospital.phone.trim().is_empty() {
        issues.push(ValidationIssue::warning(
            "hospital.phone",
            "ไม่พบโทรศัพท์สถานพยาบาล",
        ));
    }

    if draft.lines.is_empty() {
        issues.push(ValidationIssue::error(
            "lines",
            "ไม่พบรายการยาใน visit ที่เลือก",
        ));
    }
    if draft.lines.len() > ROWS_PER_FORM {
        issues.push(ValidationIssue::error(
            "lines.count",
            format!(
                "ใบสั่งจ่ายรองรับ {} รายการ แต่พบ {} รายการ กรุณาแยกเป็นหลายใบ",
                ROWS_PER_FORM,
                draft.lines.len()
            ),
        ));
    }
    for (index, line) in draft.lines.iter().enumerate() {
        let row = index + 1;
        if line.generic_name.trim().is_empty() {
            issues.push(ValidationIssue::error(
                format!("line.{row}.generic"),
                format!("รายการที่ {}: ไม่มีชื่อตัวยาสำคัญ", row),
            ));
        }
        if line.strength_text.trim().is_empty() {
            issues.push(ValidationIssue::error(
                format!("line.{row}.strength"),
                format!("รายการที่ {}: ไม่มีความแรง", row),
            ));
        }
        if line.sig.trim().is_empty() {
            issues.push(ValidationIssue::error(
                format!("line.{row}.sig"),
                format!("รายการที่ {}: ไม่มีขนาดการใช้", row),
            ));
        }
        if line.quantity <= 0 {
            issues.push(ValidationIssue::error(
                format!("line.{row}.quantity"),
                format!("รายการที่ {}: จำนวนต้องมากกว่า 0", row),
            ));
        }
        if line.print_unit.trim().is_empty() {
            issues.push(ValidationIssue::error(
                format!("line.{row}.unit"),
                format!("รายการที่ {}: ไม่มีหน่วยพิมพ์", row),
            ));
        }
    }
    if !draft.category_flags.any() {
        issues.push(ValidationIssue::error(
            "category",
            "ยังไม่ระบุประเภททางกฎหมายของรายการยา",
        ));
    }

    ValidationReport { issues }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::LawCategory;
    use crate::patient::PatientAddress;

    fn draft() -> PrescriptionDraft {
        PrescriptionDraft {
            patient: PatientDetail {
                hn: "0000001".into(),
                cid: Some("1234567890123".into()),
                full_name_th: "นายสมชาย ใจดี".into(),
                birth_date: None,
                sex: None,
                phone: "0800000000".into(),
                address: PatientAddress {
                    house_no: "1".into(),
                    moo: "2".into(),
                    soi: "".into(),
                    road: "".into(),
                    subdistrict: "ตัวอย่าง".into(),
                    district: "เมือง".into(),
                    province: "สระบุรี".into(),
                    postal_code: "18000".into(),
                },
                address_hint: String::new(),
            },
            visit: VisitSummary {
                vn: "690001".into(),
                visit_date: NaiveDate::from_ymd_opt(2026, 10, 29).expect("date"),
                visit_time: None,
                department_code: "001".into(),
                department_name: "OPD".into(),
                doctor_code: Some("0001".into()),
                doctor_name: "นพ.วิชัย".into(),
            },
            prescriber: Prescriber {
                doctor_code: Some("0001".into()),
                name: "นพ.วิชัย".into(),
                license_no: "12345".into(),
                source: Some(PrescriberSource::Hosxp),
            },
            hospital: HospitalProfile {
                name: "โรงพยาบาลสระโบสถ์".into(),
                house_no: "1".into(),
                moo: "1".into(),
                subdistrict: "ตัวอย่าง".into(),
                district: "สระโบสถ์".into(),
                province: "ลพบุรี".into(),
                ..HospitalProfile::default()
            },
            prescription_date: NaiveDate::from_ymd_opt(2026, 10, 29).expect("date"),
            lines: vec![PrescriptionLine {
                line_id: "guid".into(),
                icode: "1000001".into(),
                generic_name: "Morphine sulfate".into(),
                trade_name: "MST".into(),
                strength_text: "30 mg".into(),
                sig: "1 tab twice daily".into(),
                quantity: 10,
                print_unit: "Tab".into(),
                law_category: LawCategory::Narcotic2,
                source_vn: "690001".into(),
                source_item_no: Some(1),
                source_doctor_code: Some("0001".into()),
            }],
            category_flags: CategoryFlags {
                narcotic_2: true,
                psychotropic_2: false,
            },
        }
    }

    #[test]
    fn complete_draft_is_valid() {
        assert!(validate_draft(&draft()).is_valid());
    }

    #[test]
    fn more_than_five_lines_is_an_error() {
        let mut draft = draft();
        let line = draft.lines[0].clone();
        draft.lines = vec![line; 6];
        let report = validate_draft(&draft);
        assert!(!report.is_valid());
        assert!(report.errors().any(|issue| issue.field == "lines.count"));
    }

    #[test]
    fn missing_cid_and_licence_are_errors() {
        let mut draft = draft();
        draft.patient.cid = None;
        draft.prescriber.license_no.clear();
        let report = validate_draft(&draft);
        assert!(report.errors().any(|issue| issue.field == "patient.cid"));
        assert!(
            report
                .errors()
                .any(|issue| issue.field == "prescriber.license")
        );
    }

    #[test]
    fn incomplete_address_is_a_warning_not_an_error() {
        let mut draft = draft();
        draft.patient.address.district.clear();
        let report = validate_draft(&draft);
        assert!(report.is_valid());
        assert!(
            report
                .warnings()
                .any(|issue| issue.field == "patient.address")
        );
    }

    #[test]
    fn category_flags_follow_the_lines() {
        let lines = draft().lines;
        let flags = CategoryFlags::from_lines(&lines);
        assert!(flags.narcotic_2);
        assert!(!flags.psychotropic_2);
    }
}
