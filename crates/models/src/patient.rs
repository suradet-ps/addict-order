//! Patient identity and address types.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// How a patient search term should be interpreted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatientQuery {
    Hn(String),
    Cid(String),
    NamePrefix(String),
    NameContains(String),
}

/// A patient search result. Address is intentionally absent from the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientSummary {
    pub hn: String,
    pub cid: Option<String>,
    pub full_name_th: String,
    pub birth_date: Option<NaiveDate>,
    pub sex: Option<String>,
}

/// Structured patient address for the official form.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PatientAddress {
    pub house_no: String,
    pub moo: String,
    pub soi: String,
    pub road: String,
    pub subdistrict: String,
    pub district: String,
    pub province: String,
    pub postal_code: String,
}

impl PatientAddress {
    /// True when every field that the official form normally prints is present.
    pub fn is_complete(&self) -> bool {
        !self.house_no.trim().is_empty()
            && !self.subdistrict.trim().is_empty()
            && !self.district.trim().is_empty()
            && !self.province.trim().is_empty()
    }
}

/// The selected patient with the fields needed by the official form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientDetail {
    pub hn: String,
    pub cid: Option<String>,
    pub full_name_th: String,
    pub birth_date: Option<NaiveDate>,
    pub sex: Option<String>,
    pub phone: String,
    pub address: PatientAddress,
    /// Full `thaiaddress.full_name` value, kept as a UI hint when the
    /// structured district/province fields cannot be parsed.
    #[serde(default)]
    pub address_hint: String,
}

/// Masks a CID for list views while preserving its shape.
///
/// `1234567890123` becomes `1-XXXX-XXXXX-XX-3`. A non-13-digit value is
/// masked more aggressively so a malformed CID is never shown in full.
pub fn mask_cid(cid: &str) -> String {
    let digits: Vec<char> = cid.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 13 {
        return format!("{}-XXXX-XXXXX-XX-{}", digits[0], digits[12]);
    }
    if cid.is_empty() {
        String::new()
    } else {
        "X".repeat(cid.chars().count().min(13))
    }
}

/// Joins a Thai prefix and names without inserting a space after the prefix.
pub fn thai_full_name(pname: &str, fname: &str, lname: &str) -> String {
    let pname = pname.trim();
    let fname = fname.trim();
    let lname = lname.trim();
    let first = format!("{pname}{fname}");
    if lname.is_empty() {
        first.trim().to_string()
    } else {
        format!("{} {}", first.trim(), lname)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_cid_keeps_first_and_last_digit() {
        assert_eq!(mask_cid("1234567890123"), "1-XXXX-XXXXX-XX-3");
    }

    #[test]
    fn mask_cid_hides_malformed_values() {
        assert_eq!(mask_cid(""), "");
        assert_eq!(mask_cid("abc"), "XXX");
        assert_eq!(mask_cid("123"), "XXX");
    }

    #[test]
    fn thai_full_name_handles_prefix_and_missing_parts() {
        assert_eq!(thai_full_name("นาย", "สมชาย", "ใจดี"), "นายสมชาย ใจดี");
        assert_eq!(thai_full_name("", "สมชาย", ""), "สมชาย");
        assert_eq!(thai_full_name("นางสาว", "มาลี", ""), "นางสาวมาลี");
    }

    #[test]
    fn address_completeness_requires_the_main_fields() {
        let mut address = PatientAddress::default();
        assert!(!address.is_complete());
        address.house_no = "1".into();
        address.subdistrict = "ตัวอย่าง".into();
        address.district = "เมือง".into();
        address.province = "สระบุรี".into();
        assert!(address.is_complete());
    }
}
