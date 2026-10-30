//! Patient search classification and Thai address parsing.

use addict_order_models::PatientQuery;

/// The verified HN length on Sra Bos.
pub const HN_LENGTH: usize = 7;
/// A Thai national ID is 13 digits.
pub const CID_LENGTH: usize = 13;

/// Classifies the operator's term before it reaches the connector.
pub fn classify_patient_query(term: &str) -> PatientQuery {
    let term = term.trim();
    if term.is_empty() {
        return PatientQuery::NameContains(String::new());
    }
    if term.chars().all(|c| c.is_ascii_digit()) {
        return match term.len() {
            CID_LENGTH => PatientQuery::Cid(term.to_string()),
            HN_LENGTH => PatientQuery::Hn(term.to_string()),
            _ => PatientQuery::NamePrefix(term.to_string()),
        };
    }
    PatientQuery::NamePrefix(term.to_string())
}

/// Best-effort parser for `thaiaddress.full_name`.
///
/// The exact format on Sra Bos is not yet confirmed. The parser handles the
/// common full and abbreviated forms:
///
/// - `ตำบลหนองปลาไหล อำเภอเมือง จังหวัดสระบุรี 18230`
/// - `ต.หนองปลาไหล อ.เมือง จ.สระบุรี 18230`
///
/// If a label is absent, that field is returned empty. The operator can edit
/// the address in the form preview. The parser never guesses a name.
pub fn parse_thai_address(full_name: &str) -> (String, String, String) {
    let text = full_name.trim();
    let subdistrict = find_labeled(text, &["ตำบล", "แขวง", "ต."]).unwrap_or_default();
    let district = find_labeled(text, &["อำเภอ", "เขต", "อ."]).unwrap_or_default();
    let province = find_labeled(text, &["จังหวัด", "จ."]).unwrap_or_default();
    (subdistrict, district, strip_postal_code(&province))
}

fn find_labeled(text: &str, labels: &[&str]) -> Option<String> {
    for label in labels {
        if let Some(position) = text.find(label) {
            let rest = text[position + label.len()..].trim_start();
            let value: String = rest.chars().take_while(|c| !c.is_whitespace()).collect();
            let value = value.trim_matches(|c: char| c == ',' || c == '.');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn strip_postal_code(province: &str) -> String {
    province
        .trim()
        .trim_end_matches(|c: char| c.is_ascii_digit())
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_hn_cid_and_names() {
        assert_eq!(
            classify_patient_query("0000001"),
            PatientQuery::Hn("0000001".into())
        );
        assert_eq!(
            classify_patient_query("1234567890123"),
            PatientQuery::Cid("1234567890123".into())
        );
        assert_eq!(
            classify_patient_query("สมชาย"),
            PatientQuery::NamePrefix("สมชาย".into())
        );
        assert_eq!(
            classify_patient_query(""),
            PatientQuery::NameContains(String::new())
        );
    }

    #[test]
    fn parses_full_thai_address_labels() {
        let (subdistrict, district, province) =
            parse_thai_address("ตำบลหนองปลาไหล อำเภอเมือง จังหวัดสระบุรี 18230");
        assert_eq!(subdistrict, "หนองปลาไหล");
        assert_eq!(district, "เมือง");
        assert_eq!(province, "สระบุรี");
    }

    #[test]
    fn parses_abbreviated_thai_address_labels() {
        let (subdistrict, district, province) =
            parse_thai_address("ต.หนองปลาไหล อ.เมือง จ.สระบุรี 18230");
        assert_eq!(subdistrict, "หนองปลาไหล");
        assert_eq!(district, "เมือง");
        assert_eq!(province, "สระบุรี");
    }

    #[test]
    fn missing_labels_return_empty_fields() {
        let (subdistrict, district, province) = parse_thai_address("ไม่ทราบที่อยู่");
        assert!(subdistrict.is_empty());
        assert!(district.is_empty());
        assert!(province.is_empty());
    }
}
