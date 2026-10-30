-- Addict Order - read-only schema verification queries
-- Companion to docs/verify-schema.md
--
-- SAFETY
--   1. Run with a dedicated SELECT-only MySQL/MariaDB account.
--   2. Run against a test or staging instance first.
--   3. Do not paste real patient identifiers into the verification document.
--   4. This file contains no INSERT, UPDATE, DELETE, CREATE, DROP, or ALTER.
--   5. Replace the sample values below with approved test values.
--
-- USAGE
--   mysql --default-character-set=tis620 -u <readonly_user> -p <database> < verify-queries.sql
--
--   The verified Sra Bos instance uses tis620 / tis620_thai_ci for both the
--   database and the connection. Use tis620 for Thai text. Do not use utf8mb4
--   against this instance unless the DBA confirms a conversion-safe path.
--
--   Or paste sections into a MySQL client. Each section is independent after
--   the SET block.

-- ---------------------------------------------------------------------------
-- 0. Test values
-- ---------------------------------------------------------------------------

SET @hn = '0000001';
SET @cid = '0000000000000';
SET @name = 'สมชาย';
SET @vn = '0000001';
SET @officer_login = 'test_account';
SET @doctor_code = '0001';
SET @icode1 = '1000001';
SET @icode2 = '1000002';
SET @from = '2026-09-29';
SET @to = '2026-10-29';

-- ---------------------------------------------------------------------------
-- 1. Access and environment
-- ---------------------------------------------------------------------------

SELECT CURRENT_USER() AS current_user;
SHOW GRANTS FOR CURRENT_USER();

SELECT VERSION() AS server_version;
-- MariaDB exposes the read-only state as @@tx_read_only, not
-- @@transaction_read_only. The SET statement works on MariaDB 10.5.
SELECT @@tx_read_only AS tx_read_only_before;
SET SESSION TRANSACTION READ ONLY;
SELECT @@tx_read_only AS tx_read_only_after;

SHOW VARIABLES LIKE 'character_set_database';
SHOW VARIABLES LIKE 'collation_database';
SHOW VARIABLES LIKE 'character_set_connection';
SHOW VARIABLES LIKE 'collation_connection';

-- ---------------------------------------------------------------------------
-- 2. Authentication: officer
-- ---------------------------------------------------------------------------

SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'officer'
ORDER BY ORDINAL_POSITION;

SELECT COUNT(*) AS total,
       SUM(officer_login_name IS NULL OR officer_login_name = '') AS missing_login,
       SUM(officer_login_password_md5 IS NULL OR officer_login_password_md5 = '') AS missing_md5,
       SUM(officer_active IS NULL OR officer_active <> 'Y') AS not_active,
       SUM(officer_authentication_type_id IS NOT NULL
           AND officer_authentication_type_id <> 1) AS non_local_auth,
       SUM(officer_doctor_code IS NULL OR officer_doctor_code = '') AS missing_doctor_code
FROM officer;

SELECT SUM(officer_login_password_md5 REGEXP '^[0-9A-Fa-f]{32}$') AS valid_md5_shape,
       SUM(officer_login_password_md5 IS NOT NULL
           AND officer_login_password_md5 <> ''
           AND officer_login_password_md5 NOT REGEXP '^[0-9A-Fa-f]{32}$') AS bad_md5_shape
FROM officer;

-- Use an approved test account. Do not use a real clinician password.
-- The plaintext officer_login_password column has one populated row and does
-- not match the MD5 column. Never select it.
SELECT officer_login_name,
       officer_login_password_md5,
       MD5('the-test-password') AS computed_md5
FROM officer
WHERE officer_login_name = @officer_login;

SELECT COUNT(*) AS officers_with_doctor_code,
       SUM(d.code IS NULL) AS orphan_doctor_codes
FROM officer o
LEFT JOIN doctor d ON d.code = o.officer_doctor_code
WHERE o.officer_doctor_code IS NOT NULL
  AND o.officer_doctor_code <> '';

-- ---------------------------------------------------------------------------
-- 3. Patient identity and search
-- ---------------------------------------------------------------------------

SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'patient'
ORDER BY ORDINAL_POSITION;

SELECT COUNT(*) AS total,
       SUM(hn IS NULL OR hn = '') AS missing_hn,
       SUM(cid IS NULL OR cid = '') AS missing_cid,
       SUM(cid IS NOT NULL AND cid <> ''
           AND cid NOT REGEXP '^[0-9]{13}$') AS bad_cid_shape,
       SUM(pname IS NULL OR pname = '') AS missing_pname,
       SUM(fname IS NULL OR fname = '') AS missing_fname,
       SUM(lname IS NULL OR lname = '') AS missing_lname
FROM patient;

SELECT COUNT(*) AS duplicate_cid_groups,
       SUM(row_count) AS duplicate_cid_rows
FROM (
  SELECT cid, COUNT(*) AS row_count
  FROM patient
  WHERE cid IS NOT NULL AND cid <> ''
  GROUP BY cid
  HAVING COUNT(*) > 1
) t;

-- Exact HN search
SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE hn = @hn
LIMIT 20;

-- Exact CID search
SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE cid = @cid
LIMIT 20;

-- Name prefix search
SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE fname LIKE CONCAT(@name, '%')
   OR lname LIKE CONCAT(@name, '%')
   OR CONCAT_WS(' ', pname, fname, lname) LIKE CONCAT(@name, '%')
LIMIT 20;

-- Name contains fallback
SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE fname LIKE CONCAT('%', @name, '%')
   OR lname LIKE CONCAT('%', @name, '%')
   OR CONCAT_WS(' ', pname, fname, lname) LIKE CONCAT('%', @name, '%')
LIMIT 20;

-- ---------------------------------------------------------------------------
-- 4. Patient address
-- ---------------------------------------------------------------------------

SELECT COLUMN_NAME, COLUMN_TYPE, CHARACTER_MAXIMUM_LENGTH
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'patient'
  AND COLUMN_NAME IN (
    'addrpart', 'moopart', 'tmbpart', 'amppart', 'chwpart',
    'road', 'addr_soi', 'hometel', 'informtel', 'worktel',
    'mobile_phone_number', 'addressid'
  );

SHOW TABLES LIKE '%address%';
SHOW TABLES LIKE '%province%';
SHOW TABLES LIKE '%amphur%';
SHOW TABLES LIKE '%district%';
SHOW TABLES LIKE '%tambon%';
SHOW TABLES LIKE '%subdistrict%';
SHOW TABLES LIKE '%zip%';
SHOW TABLES LIKE 'c_%';
SHOW TABLES LIKE 'patient_address%';

-- Verified thaiaddress columns: addressid varchar(6), name, full_name,
-- pocode, codetype. The addressid is chwpart + amppart + tmbpart (6 digits).
SHOW COLUMNS FROM thaiaddress;

-- Candidate A is rejected: thaiaddress has no moo column and the addressid
-- is only 6 digits. Do not use a 4-level LPAD join.

-- Candidate B is verified: join at the subdistrict level.
-- Coverage on the Sra Bos instance: 49,550 / 49,557 patients (99.99%).
SELECT p.hn, p.addrpart, p.moopart, p.chwpart, p.amppart, p.tmbpart,
       t.addressid, t.name AS subdistrict_name, t.full_name,
       t.pocode, t.codetype
FROM patient p
LEFT JOIN thaiaddress t
  ON t.addressid = CONCAT(
       COALESCE(p.chwpart, ''),
       COALESCE(p.amppart, ''),
       COALESCE(p.tmbpart, '')
     )
WHERE p.hn = @hn;

-- Confirm the full_name format before splitting it into district/province.
-- pocode is missing on 685 / 8,924 subdistricts (7.7%).
SELECT COUNT(*) AS total,
       SUM(pocode IS NULL OR pocode = '') AS missing_pocode
FROM thaiaddress;

-- Candidate C: separate code tables. Not needed after Candidate B passed.
-- SHOW COLUMNS FROM c_province;
-- SHOW COLUMNS FROM c_district;
-- SHOW COLUMNS FROM c_subdistrict;

-- patient_address exists but has 0 rows on this instance. Do not use it.
-- SHOW COLUMNS FROM patient_address;

SELECT COUNT(*) AS total,
       SUM(addrpart IS NULL OR addrpart = '') AS missing_house_no,
       SUM(moopart IS NULL OR moopart = '') AS missing_moo,
       SUM(tmbpart IS NULL OR tmbpart = '') AS missing_subdistrict_code,
       SUM(amppart IS NULL OR amppart = '') AS missing_district_code,
       SUM(chwpart IS NULL OR chwpart = '') AS missing_province_code
FROM patient;

-- Verified phone columns: hometel, informtel, worktel, mobile_phone_number.
-- hometel is the primary source. mobile_phone_number has only 136 rows.
SELECT COLUMN_NAME, COLUMN_TYPE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'patient'
  AND COLUMN_NAME IN ('hometel', 'informtel', 'worktel', 'mobile_phone_number');

SELECT SUM(hometel IS NOT NULL AND hometel <> '') AS hometel_filled,
       SUM(informtel IS NOT NULL AND informtel <> '') AS informtel_filled,
       SUM(worktel IS NOT NULL AND worktel <> '') AS worktel_filled,
       SUM(mobile_phone_number IS NOT NULL AND mobile_phone_number <> '') AS mobile_filled
FROM patient;

-- ---------------------------------------------------------------------------
-- 5. Visits
-- ---------------------------------------------------------------------------

SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'ovst'
ORDER BY ORDINAL_POSITION;

SELECT v.vn, v.vstdate, v.vsttime, v.main_dep, v.spclty, v.doctor,
       d.name AS doctor_name
FROM ovst v
LEFT JOIN doctor d ON d.code = v.doctor
WHERE v.hn = @hn
  AND v.vstdate BETWEEN @from AND @to
ORDER BY v.vstdate DESC, v.vsttime DESC
LIMIT 50;

SHOW COLUMNS FROM kskdepartment;

-- Uncomment if the department join is confirmed.
-- SELECT v.vn, v.main_dep, k.department AS department_name
-- FROM ovst v
-- LEFT JOIN kskdepartment k ON k.depcode = v.main_dep
-- WHERE v.vn = @vn
-- LIMIT 5;

SELECT v.vn, v.vstdate, v.vsttime,
       MIN(o.rxdate) AS first_order_date,
       MAX(o.rxdate) AS last_order_date,
       MIN(o.rxtime) AS first_order_time,
       MAX(o.rxtime) AS last_order_time
FROM ovst v
LEFT JOIN opitemrece o ON o.vn = v.vn
WHERE v.vn = @vn
GROUP BY v.vn, v.vstdate, v.vsttime;

SELECT COUNT(*) AS total,
       SUM(an IS NULL OR an = '') AS opd_rows,
       SUM(an IS NOT NULL AND an <> '') AS ipd_rows
FROM opitemrece
WHERE vn = @vn;

-- ---------------------------------------------------------------------------
-- 6. Narcotic order lines and drug master
-- ---------------------------------------------------------------------------

SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'opitemrece'
ORDER BY ORDINAL_POSITION;

SELECT COUNT(*) AS total,
       SUM(qty > 0) AS positive_qty,
       SUM(qty <= 0) AS non_positive_qty,
       SUM(an IS NULL OR an = '') AS opd_rows,
       SUM(an IS NOT NULL AND an <> '') AS ipd_rows,
       SUM(drugusage IS NULL OR drugusage = '') AS missing_drugusage,
       SUM(sp_use IS NULL OR sp_use = '') AS missing_sp_use
FROM opitemrece
WHERE vn = @vn;

SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'drugitems'
ORDER BY ORDINAL_POSITION;

SELECT icode, name, trade_name, strength, units, dosageform,
       drug_control_type_id
FROM drugitems
WHERE icode IN (@icode1, @icode2);

-- Verified legal-category source: drug_control_type_id points to
-- drug_control_type (NC1 to NC5, HF1 to HF4). On the Sra Bos instance the
-- column is empty for all 659 drugitems, so the catalog manual field is
-- mandatory.
SHOW COLUMNS FROM drugitems LIKE '%drug_control%';
SHOW COLUMNS FROM drug_control_type;
SELECT * FROM drug_control_type LIMIT 20;

SELECT drug_control_type_id, COUNT(*) AS item_count
FROM drugitems
GROUP BY drug_control_type_id;

-- opitemrece_narcotic exists but has 0 rows. Do not use it as a source of
-- legal category or controlled-dispensing confirmation.
SELECT COUNT(*) AS opitemrece_narcotic_rows
FROM opitemrece_narcotic;

SHOW COLUMNS FROM drugusage;
SHOW COLUMNS FROM sp_use;

SELECT COUNT(*) AS total,
       SUM(name1 IS NOT NULL AND name1 <> '') AS name1_filled,
       SUM(name2 IS NOT NULL AND name2 <> '') AS name2_filled,
       SUM(name3 IS NOT NULL AND name3 <> '') AS name3_filled
FROM drugusage;

-- Main order query.
-- Verified on Sra Bos: o.hos_guid is the PRIMARY KEY and doc.licenseno
-- exists. Keep them. If d.trade_name is absent on another instance, remove
-- that column and use the catalog trade name.
SELECT o.vn, o.hn, o.icode, CAST(o.qty AS CHAR) AS qty,
       o.drugusage, o.sp_use, o.doctor, o.rxdate, o.rxtime,
       o.item_no, o.hos_guid,
       d.name AS generic_name, d.trade_name, d.strength, d.units,
       du.name1 AS drugusage_name1,
       du.name2 AS drugusage_name2,
       du.name3 AS drugusage_name3,
       su.name1 AS sp_use_name1,
       su.name2 AS sp_use_name2,
       su.name3 AS sp_use_name3,
       doc.name AS doctor_name,
       doc.licenseno AS doctor_license_no
FROM opitemrece o
JOIN drugitems d ON d.icode = o.icode
LEFT JOIN drugusage du ON du.drugusage = o.drugusage
LEFT JOIN sp_use su ON su.sp_use = o.sp_use
LEFT JOIN doctor doc ON doc.code = o.doctor
WHERE o.vn = @vn
  AND o.icode IN (@icode1, @icode2)
  AND o.qty > 0
  AND (o.an IS NULL OR o.an = '')
  AND o.sub_type = '1'
  AND IFNULL(o.item_type, '') <> 'A'
ORDER BY o.item_no, o.icode;

SELECT
  SUM(1) AS all_lines,
  SUM(sub_type = '1') AS sub_type_drug_lines,
  SUM(IFNULL(item_type, '') = 'A') AS automatic_charge_lines,
  SUM(qty > 0) AS positive_lines,
  SUM(an IS NULL OR an = '') AS opd_lines
FROM opitemrece
WHERE vn = @vn;

-- hos_guid is the PRIMARY KEY (varchar(38), NOT NULL). Missing or duplicate
-- values are impossible; this query is retained as a guard.
SELECT COUNT(*) AS total_lines,
       COUNT(DISTINCT hos_guid) AS distinct_hos_guid,
       SUM(hos_guid IS NULL OR hos_guid = '') AS missing_hos_guid
FROM opitemrece
WHERE vn = @vn;

SELECT icode, drugusage, sp_use, doctor,
       COUNT(*) AS line_count,
       SUM(qty) AS total_qty
FROM opitemrece
WHERE vn = @vn
  AND icode IN (@icode1)
  AND qty > 0
  AND (an IS NULL OR an = '')
  AND sub_type = '1'
  AND IFNULL(item_type, '') <> 'A'
GROUP BY icode, drugusage, sp_use, doctor
HAVING COUNT(*) > 1;

SELECT o.icode, d.name, d.units,
       o.qty, CAST(o.qty AS CHAR) AS qty_text,
       SUM(o.qty <> FLOOR(o.qty)) AS decimal_quantity_rows
FROM opitemrece o
JOIN drugitems d ON d.icode = o.icode
WHERE o.vn = @vn
  AND o.icode IN (@icode1, @icode2)
  AND o.qty > 0
GROUP BY o.icode, d.name, d.units, o.qty;

SELECT COUNT(*) AS configured_lines,
       SUM(o.drugusage IS NULL OR o.drugusage = '') AS missing_drugusage_code,
       SUM(o.sp_use IS NULL OR o.sp_use = '') AS missing_sp_use_code,
       SUM(du.name1 IS NULL OR du.name1 = '') AS missing_name1,
       SUM(du.name2 IS NULL OR du.name2 = '') AS missing_name2,
       SUM(du.name3 IS NULL OR du.name3 = '') AS missing_name3
FROM opitemrece o
LEFT JOIN drugusage du ON du.drugusage = o.drugusage
WHERE o.vn = @vn
  AND o.icode IN (@icode1, @icode2)
  AND o.qty > 0
  AND (o.an IS NULL OR o.an = '')
  AND o.sub_type = '1'
  AND IFNULL(o.item_type, '') <> 'A';

SELECT o.icode, d.name, COUNT(*) AS lines, SUM(o.qty) AS total_qty
FROM opitemrece o
JOIN drugitems d ON d.icode = o.icode
WHERE o.vn = @vn
  AND o.qty > 0
  AND (o.an IS NULL OR o.an = '')
  AND o.sub_type = '1'
  AND IFNULL(o.item_type, '') <> 'A'
GROUP BY o.icode, d.name
ORDER BY d.name;

-- ---------------------------------------------------------------------------
-- 7. Prescriber and licence
-- ---------------------------------------------------------------------------

SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'doctor'
ORDER BY ORDINAL_POSITION;

SHOW COLUMNS FROM doctor LIKE '%licen%';
SHOW COLUMNS FROM doctor LIKE '%license%';

-- Uncomment the licence column that exists.
-- SELECT COUNT(*) AS total,
--        SUM(licenseno IS NULL OR licenseno = '') AS missing_licence
-- FROM doctor;

SELECT o.vn,
       o.doctor AS order_doctor,
       v.doctor AS visit_doctor,
       od.name AS order_doctor_name,
       vd.name AS visit_doctor_name,
       od.licenseno AS order_licence,
       vd.licenseno AS visit_licence
FROM opitemrece o
JOIN ovst v ON v.vn = o.vn
LEFT JOIN doctor od ON od.code = o.doctor
LEFT JOIN doctor vd ON vd.code = v.doctor
WHERE o.vn = @vn
LIMIT 20;

SELECT COUNT(*) AS configured_lines,
       SUM(o.doctor IS NULL OR o.doctor = '') AS missing_order_doctor,
       SUM(v.doctor IS NULL OR v.doctor = '') AS missing_visit_doctor
FROM opitemrece o
JOIN ovst v ON v.vn = o.vn
WHERE o.vn = @vn
  AND o.icode IN (@icode1, @icode2)
  AND o.qty > 0
  AND (o.an IS NULL OR o.an = '')
  AND o.sub_type = '1'
  AND IFNULL(o.item_type, '') <> 'A';

SELECT o.officer_login_name, o.officer_doctor_code,
       d.name AS doctor_name, d.licenseno AS licence_no
FROM officer o
LEFT JOIN doctor d ON d.code = o.officer_doctor_code
WHERE o.officer_login_name = @officer_login;

-- ---------------------------------------------------------------------------
-- 8. Hospital profile
-- ---------------------------------------------------------------------------

SHOW TABLES LIKE 'hospital%';
SHOW TABLES LIKE 'hosp%';
SHOW TABLES LIKE 'hos%';

-- Verified: hospital_profile exists with 1 row. Inspect it and check the
-- address against the hospital name before printing.
SHOW COLUMNS FROM hospital_profile;
SELECT * FROM hospital_profile LIMIT 5;

-- ---------------------------------------------------------------------------
-- 9. Catalog completeness
-- ---------------------------------------------------------------------------

SELECT icode, name, trade_name, strength, units, dosageform,
       drug_control_type_id
FROM drugitems
WHERE icode IN (@icode1, @icode2);

-- ---------------------------------------------------------------------------
-- 10. Indexes, plans, and encoding
-- ---------------------------------------------------------------------------

SHOW INDEX FROM patient;
SHOW INDEX FROM ovst;
SHOW INDEX FROM opitemrece;
SHOW INDEX FROM drugitems;
SHOW INDEX FROM doctor;

EXPLAIN SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE hn = @hn;

EXPLAIN SELECT vn, vstdate, vsttime, doctor, main_dep
FROM ovst
WHERE hn = @hn
  AND vstdate BETWEEN @from AND @to;

EXPLAIN SELECT o.vn, o.icode, o.qty, o.drugusage, o.sp_use, o.doctor
FROM opitemrece o
WHERE o.vn = @vn
  AND o.icode IN (@icode1, @icode2)
  AND o.qty > 0;

SET @thai_name = 'สมชาย';
SELECT hn, pname, fname, lname
FROM patient
WHERE fname LIKE CONCAT(@thai_name, '%')
LIMIT 5;

-- Order validity and cancellation.
SHOW COLUMNS FROM opitemrece LIKE '%status%';
SHOW COLUMNS FROM opitemrece LIKE '%confirm%';
SHOW COLUMNS FROM opitemrece LIKE '%cancel%';
SHOW COLUMNS FROM opitemrece LIKE '%void%';

-- If a status column exists, replace <candidate_column> and run this query.
-- SELECT <candidate_column> AS status_value, COUNT(*) AS line_count
-- FROM opitemrece
-- WHERE vn = @vn
-- GROUP BY <candidate_column>;

-- ---------------------------------------------------------------------------
-- 11. Print and legal checks
-- ---------------------------------------------------------------------------

-- These are not SQL checks. Complete the P1 to P20 table in
-- docs/verify-schema.md section 11 with the pharmacist and hospital policy.
