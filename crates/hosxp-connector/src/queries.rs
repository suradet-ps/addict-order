//! Every SQL statement the connector can execute. Compile-time constants or
//! placeholder lists only; values are always bound.

/// Smoke test.
pub const PING_SQL: &str = "SELECT 1";

/// One local `officer` account for password verification.
///
/// Only `officer` is read. `officer_login_password` (plaintext) and `opduser`
/// never appear.
pub const FETCH_OFFICER_SQL: &str = "\
SELECT CAST(o.officer_id AS SIGNED) AS officer_id,
       o.officer_login_name AS login_name,
       COALESCE(o.officer_login_password_md5, '') AS password_md5,
       COALESCE(o.officer_active, 'N') AS active,
       CAST(COALESCE(o.officer_authentication_type_id, 1) AS SIGNED) AS auth_type_id,
       NULLIF(o.officer_doctor_code, '') AS doctor_code,
       COALESCE(
           NULLIF(TRIM(CONCAT(COALESCE(o.officer_pname, ''),
                              COALESCE(o.officer_fname, ''), ' ',
                              COALESCE(o.officer_lname, ''))), ''),
           NULLIF(TRIM(d.name), ''),
           ''
       ) AS display_name
FROM officer o
LEFT JOIN doctor d ON d.code = NULLIF(o.officer_doctor_code, '')
WHERE o.officer_login_name = ?
LIMIT 1";

pub const PATIENT_SEARCH_BY_HN: &str = "\
SELECT hn, cid,
       TRIM(CONCAT(COALESCE(pname, ''), COALESCE(fname, ''), ' ', COALESCE(lname, ''))) AS full_name_th,
       birthday, sex
FROM patient
WHERE hn = ?
LIMIT 20";

pub const PATIENT_SEARCH_BY_CID: &str = "\
SELECT hn, cid,
       TRIM(CONCAT(COALESCE(pname, ''), COALESCE(fname, ''), ' ', COALESCE(lname, ''))) AS full_name_th,
       birthday, sex
FROM patient
WHERE cid = ?
LIMIT 20";

pub const PATIENT_SEARCH_NAME_PREFIX: &str = "\
SELECT hn, cid,
       TRIM(CONCAT(COALESCE(pname, ''), COALESCE(fname, ''), ' ', COALESCE(lname, ''))) AS full_name_th,
       birthday, sex
FROM patient
WHERE fname LIKE ?
   OR lname LIKE ?
   OR TRIM(CONCAT(COALESCE(pname, ''), COALESCE(fname, ''), ' ', COALESCE(lname, ''))) LIKE ?
ORDER BY fname, lname
LIMIT 20";

pub const PATIENT_SEARCH_NAME_CONTAINS: &str = "\
SELECT hn, cid,
       TRIM(CONCAT(COALESCE(pname, ''), COALESCE(fname, ''), ' ', COALESCE(lname, ''))) AS full_name_th,
       birthday, sex
FROM patient
WHERE fname LIKE ?
   OR lname LIKE ?
   OR TRIM(CONCAT(COALESCE(pname, ''), COALESCE(fname, ''), ' ', COALESCE(lname, ''))) LIKE ?
ORDER BY fname, lname
LIMIT 20";

/// Full patient identity plus the verified subdistrict-level address join.
///
/// `thaiaddress.addressid` is varchar(6) = chwpart + amppart + tmbpart.
/// `full_name` is returned as a hint because its exact format still needs a
/// sample check; the core parser fills district/province when it can.
pub const PATIENT_BY_HN_SQL: &str = "\
SELECT p.hn, p.cid,
       TRIM(CONCAT(COALESCE(p.pname, ''), COALESCE(p.fname, ''), ' ', COALESCE(p.lname, ''))) AS full_name_th,
       p.birthday, p.sex,
       COALESCE(p.addrpart, '') AS house_no,
       COALESCE(p.moopart, '') AS moo,
       COALESCE(p.addr_soi, '') AS soi,
       COALESCE(p.road, '') AS road,
       COALESCE(t.name, '') AS subdistrict,
       COALESCE(t.full_name, '') AS address_hint,
       COALESCE(t.pocode, '') AS postal_code,
       COALESCE(
           NULLIF(p.hometel, ''),
           NULLIF(p.informtel, ''),
           NULLIF(p.worktel, ''),
           NULLIF(p.mobile_phone_number, ''),
           ''
       ) AS phone
FROM patient p
LEFT JOIN thaiaddress t
  ON t.addressid = CONCAT(
       COALESCE(p.chwpart, ''),
       COALESCE(p.amppart, ''),
       COALESCE(p.tmbpart, '')
     )
WHERE p.hn = ?
LIMIT 1";

pub const VISITS_BY_HN_SQL: &str = "\
SELECT v.vn, v.vstdate, v.vsttime,
       COALESCE(v.main_dep, '') AS department_code,
       COALESCE(k.department, '') AS department_name,
       NULLIF(v.doctor, '') AS doctor_code,
       COALESCE(d.name, '') AS doctor_name
FROM ovst v
LEFT JOIN kskdepartment k ON k.depcode = v.main_dep
LEFT JOIN doctor d ON d.code = v.doctor
WHERE v.hn = ?
  AND v.vstdate BETWEEN ? AND ?
ORDER BY v.vstdate DESC, v.vsttime DESC
LIMIT 50";

/// Drug search for the catalog picker. No `istype` filter: the verified
/// instance has no such column. The pharmacist chooses the icode.
pub const DRUG_SEARCH_SQL: &str = "\
SELECT icode,
       COALESCE(name, '') AS name,
       COALESCE(trade_name, '') AS trade_name,
       COALESCE(strength, '') AS strength,
       COALESCE(units, '') AS units,
       COALESCE(dosageform, '') AS dosageform,
       NULLIF(drug_control_type_id, '') AS drug_control_type_id
FROM drugitems
WHERE icode LIKE ?
   OR name LIKE ?
   OR trade_name LIKE ?
ORDER BY name
LIMIT 30";

pub const DOCTOR_SEARCH_SQL: &str = "\
SELECT code, name, COALESCE(licenseno, '') AS licenseno
FROM doctor
WHERE code LIKE ?
   OR name LIKE ?
ORDER BY name
LIMIT 20";

/// The hospital profile source is a prefill only. The printed profile is the
/// operator-edited settings value.
pub const HOSPITAL_PROFILE_SQL: &str = "SELECT * FROM hospital_profile LIMIT 1";

const NARCOTIC_ORDERS_SELECT: &str = "\
SELECT o.vn, o.hn, o.icode,
       CAST(o.item_no AS SIGNED) AS item_no,
       o.hos_guid,
       CAST(o.qty AS SIGNED) AS qty,
       o.drugusage, o.sp_use, o.doctor,
       o.rxdate, o.rxtime,
       COALESCE(d.name, '') AS generic_name,
       COALESCE(d.trade_name, '') AS trade_name,
       COALESCE(d.strength, '') AS strength,
       COALESCE(d.units, '') AS source_units,
       COALESCE(
           NULLIF(du.name3, ''),
           NULLIF(du.name2, ''),
           NULLIF(du.name1, ''),
           NULLIF(su.name3, ''),
           NULLIF(su.name2, ''),
           NULLIF(su.name1, ''),
           ''
       ) AS sig_text,
       COALESCE(o.drugusage, '') AS drugusage_code,
       COALESCE(o.sp_use, '') AS sp_use_code,
       NULLIF(o.doctor, '') AS doctor_code,
       COALESCE(doc.name, '') AS doctor_name,
       NULLIF(doc.licenseno, '') AS doctor_license_no
FROM opitemrece o
JOIN drugitems d ON d.icode = o.icode
LEFT JOIN drugusage du ON du.drugusage = o.drugusage
LEFT JOIN sp_use su ON su.sp_use = o.sp_use
LEFT JOIN doctor doc ON doc.code = o.doctor
WHERE o.vn = ?
  AND o.icode IN (";

const NARCOTIC_ORDERS_FILTER: &str = "\
)
  AND o.qty > 0
  AND (o.an IS NULL OR o.an = '')
  AND o.sub_type = '1'
  AND IFNULL(o.item_type, '') <> 'A'
ORDER BY o.item_no, o.icode";

/// Builds the order query with one `?` per configured icode.
pub fn narcotic_orders_sql(icodes_len: usize) -> String {
    let placeholders = std::iter::repeat_n("?", icodes_len)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{NARCOTIC_ORDERS_SELECT}{placeholders}{NARCOTIC_ORDERS_FILTER}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::readonly_guard::assert_read_only;

    #[test]
    fn every_constant_passes_the_read_only_guard() {
        for sql in [
            PING_SQL,
            FETCH_OFFICER_SQL,
            PATIENT_SEARCH_BY_HN,
            PATIENT_SEARCH_BY_CID,
            PATIENT_SEARCH_NAME_PREFIX,
            PATIENT_SEARCH_NAME_CONTAINS,
            PATIENT_BY_HN_SQL,
            VISITS_BY_HN_SQL,
            DRUG_SEARCH_SQL,
            DOCTOR_SEARCH_SQL,
            HOSPITAL_PROFILE_SQL,
        ] {
            assert!(assert_read_only(sql).is_ok(), "guard must accept: {sql}");
        }
        for count in [1_usize, 2, 5, 20] {
            let sql = narcotic_orders_sql(count);
            assert!(assert_read_only(&sql).is_ok(), "guard must accept: {sql}");
        }
    }

    #[test]
    fn order_query_binds_one_placeholder_per_icode_plus_vn() {
        for count in [1_usize, 2, 5, 20] {
            let sql = narcotic_orders_sql(count);
            assert_eq!(sql.matches('?').count(), count + 1);
        }
    }

    #[test]
    fn order_query_shape_is_stable() {
        let sql = narcotic_orders_sql(2);
        assert!(sql.contains("FROM opitemrece o"));
        assert!(sql.contains("JOIN drugitems d ON d.icode = o.icode"));
        assert!(sql.contains("IN (?, ?)"));
        assert!(sql.contains("o.qty > 0"));
        assert!(sql.contains("(o.an IS NULL OR o.an = '')"));
        assert!(sql.contains("o.sub_type = '1'"));
        assert!(sql.contains("IFNULL(o.item_type, '') <> 'A'"));
        assert!(sql.contains("o.hos_guid"));
        assert!(!sql.contains("istype"));
    }

    #[test]
    fn officer_query_uses_only_safe_columns() {
        assert!(FETCH_OFFICER_SQL.contains("FROM officer o"));
        assert!(FETCH_OFFICER_SQL.contains("officer_login_password_md5"));
        assert!(!FETCH_OFFICER_SQL.contains("officer_login_password "));
        assert!(!FETCH_OFFICER_SQL.contains("opduser"));
        assert!(FETCH_OFFICER_SQL.contains("LEFT JOIN doctor d"));
        assert!(FETCH_OFFICER_SQL.contains("LIMIT 1"));
        assert!(!FETCH_OFFICER_SQL.contains(';'));
    }

    #[test]
    fn patient_queries_use_birthday_and_thaiaddress_join() {
        assert!(PATIENT_BY_HN_SQL.contains("p.birthday"));
        assert!(!PATIENT_BY_HN_SQL.contains("birthdate"));
        assert!(PATIENT_BY_HN_SQL.contains("FROM patient p"));
        assert!(PATIENT_BY_HN_SQL.contains("LEFT JOIN thaiaddress t"));
        assert!(PATIENT_BY_HN_SQL.contains("t.addressid = CONCAT("));
        assert!(PATIENT_BY_HN_SQL.contains("t.pocode"));
        assert!(PATIENT_BY_HN_SQL.contains("p.hometel"));
    }
}
