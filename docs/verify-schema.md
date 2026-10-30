# Schema Verification - Addict Order

This document is the verification gate between the design and the first line of
connector code. Every table and column used by the app must be checked against
the real HOSxP instance. A query that touches an unverified object must carry a
`// SCHEMA-UNVERIFIED:` comment until the corresponding item here is signed off.

The companion SQL file is [`verify-queries.sql`](verify-queries.sql). Run it
with a read-only account against a test or staging instance first.

## Verified baseline (2026-10-29)

Instance: `hos`, โรงพยาบาลสระโบสถ์, MariaDB 10.5.29.

The first verification pass found 6 critical SQL issues, 8 high-impact schema
mismatches, and 3 operational warnings. This revision corrects them.

Key facts:

- Charset: `tis620` / `tis620_thai_ci`. The CLI uses
  `--default-character-set=tis620`; the app client uses `utf8mb4` so sqlx can
  decode the converted UTF-8 strings.
- HN is fixed at 7 digits.
- `thaiaddress.addressid` is varchar(6) = `chwpart + amppart + tmbpart`.
  Candidate B is the verified join and covers 99.99% of patients. `pocode` is
  missing on 7.7% of subdistricts.
- Patient phone columns are `hometel`, `informtel`, `worktel`, and
  `mobile_phone_number`.
- `opitemrece.hos_guid` is the PRIMARY KEY. `qty` is int(11). `sub_type = '1'`
  is drug, `sub_type = '3'` is non-drug, and `item_type = 'A'` has 172 rows.
- `drugitems.istype`, `narcotic`, `psychotropic`, and `drug_category_id` do not
  exist. `drug_control_type_id` exists but is empty on 659 / 659 drugitems.
- `opitemrece_narcotic` exists but has 0 rows.
- `doctor.licenseno` exists as varchar(50) with an index. 16 / 293 doctors have
  no licence.
- `hospital_profile` has 1 row. The address has a conflict and needs pharmacist
  correction before printing.
- CID is missing on 1,146 patients. 3 CIDs have a bad shape. 6 duplicate CID
  groups cover 19 rows.
- Sig is missing on 9.5% of order lines. `sp_use` is missing on 90.5%.
- `qty` has no decimal values. Decimal quantity support is not needed.

Remaining blockers: B1, B4, B5 to B11, and B12 to B14.

## Contents

- [Verified baseline](#verified-baseline-2026-10-29)
- [0. How to use this document](#0-how-to-use-this-document)
- [1. Data flow and required fields](#1-data-flow-and-required-fields)
- [2. Access, environment, and authentication](#2-access-environment-and-authentication)
- [3. Patient identity and search](#3-patient-identity-and-search)
- [4. Patient address](#4-patient-address)
- [5. Visits](#5-visits)
- [6. Narcotic order lines](#6-narcotic-order-lines)
- [7. Prescriber and licence](#7-prescriber-and-licence)
- [8. Hospital profile](#8-hospital-profile)
- [9. Catalog settings](#9-catalog-settings)
- [10. Indexes, encoding, and performance](#10-indexes-encoding-and-performance)
- [11. Print and legal verification](#11-print-and-legal-verification)
- [12. Schema contract](#12-schema-contract)
- [13. Verification results log](#13-verification-results-log)
- [14. Blockers and decisions](#14-blockers-and-decisions)
- [15. Recommended defaults until verified](#15-recommended-defaults-until-verified)
- [16. What to send back](#16-what-to-send-back)

## 0. How to use this document

### 0.1 Rules

1. Use a dedicated MySQL/MariaDB user with `GRANT SELECT` only.
2. Run against a test or staging instance first. Never run exploratory queries
   against production during clinic hours.
3. Do not copy real patient names, HN, CID, addresses, phone numbers, or drug
   lines into this document or into a chat.
4. Use a fictional or approved test patient for row-level checks.
5. Report aggregate counts and pass/fail results.
6. Do not change the schema, add indexes, or create views as part of this
   verification unless the DBA explicitly approves it as a separate task.
7. Every item has a status. Do not mark an item as passed without evidence.
8. If a column is missing, do not add it to the query. Record the fallback and
   move on.

Example command for a test or staging instance:

```bash
mysql --default-character-set=tis620 -u <readonly_user> -p <database> \
  < docs/verify-queries.sql
```

The SQL file is read-only, but it still opens several metadata and data queries.
Run it on a test or staging instance first.

### 0.2 Status legend

| Symbol | Meaning |
|---|---|
| `[ ]` | Not checked |
| `[P]` | Pass |
| `[F]` | Fail, needs a different column or join |
| `[N]` | Not available on this instance |
| `[D]` | Deferred, not needed for v1 |
| `[B]` | Blocked, needs a pharmacist or DBA decision |

### 0.3 Evidence

For each check, record:

- the date and time
- the verifier
- the instance name or code
- the SQL reference
- the aggregate result or a masked sample
- the decision and fallback

Do not record credentials, patient identifiers, or full addresses.

### 0.4 Minimum path to first code

The project can start M0 after the authentication checks V1 to V5 pass and the
V6 test account is scheduled. Patient search can start after V7 to V14. The
order pull can start after V15 to V27, with V52 pending. The print sheet can
start after the legal and form checks in section 11.

## 1. Data flow and required fields

```text
officer
  -> login session
patient
  -> patient identity and address
ovst
  -> visit and prescription date
opitemrece
  -> order lines for the selected visit
drugitems
  -> drug identity, strength, trade name, unit
drugusage / sp_use
  -> directions for use
doctor
  -> prescriber name and licence number
hospital profile or settings
  -> hospital name and address
catalog settings
  -> which icodes are in scope, legal category, print unit, default sig
```

### 1.1 Form field to source map

| Form field | Primary source | Fallback | Required |
|---|---|---|---|
| Date | `ovst.vstdate` for the selected visit | None | Yes |
| Prescriber name | `doctor.name` through `opitemrece.doctor` or `ovst.doctor` | Officer doctor code, manual selection | Yes |
| Licence number | Verified `doctor` licence column | Session-only manual entry | Yes |
| Hospital name | Settings hospital profile | Verified hospital table | Yes |
| Hospital address | Settings hospital profile | Verified hospital table | Yes |
| Hospital phone and fax | Settings hospital profile | Verified hospital table | No |
| Category checkbox 1 | Catalog `law_category = NARCOTIC_2` | None | Yes |
| Category checkbox 2 | Catalog `law_category = PSYCHOTROPIC_2` | None | Yes |
| Generic name | Catalog `generic_name` | `drugitems.name` | Yes |
| Trade name | Catalog `trade_name` | `drugitems.trade_name` | No |
| Strength | Catalog `strength_text` | `drugitems.strength` | Yes |
| Directions | Order sig text | Catalog `default_sig` | Yes |
| Quantity | `opitemrece.qty` | None in v1 | Yes |
| Unit | Catalog `print_unit` | Verified unit mapping | Yes |
| Patient name | `patient` name fields | None | Yes |
| Patient CID | `patient.cid` | Other official ID | Yes |
| Patient address | Verified address strategy | Manual edit | Warning |
| Patient phone | Verified patient phone column | Manual edit | No |

## 2. Access, environment, and authentication

### V1. Read-only account and grants

**Purpose:** Confirm the app user cannot write.

```sql
SELECT CURRENT_USER() AS current_user;
SHOW GRANTS FOR CURRENT_USER();
```

**Pass criteria:**

- The user is a dedicated application user, for example `addict_order_ro`.
- The grants are `SELECT` only on the required schema or tables.
- No `INSERT`, `UPDATE`, `DELETE`, `CREATE`, `DROP`, or `ALTER` grant.
- The user can connect from the application host.

**Fallback:** If a dedicated user cannot be created, stop. Do not use a
write-capable account.

**Verified on Sra Bos:** The verification run completed. Confirm the exact
`SHOW GRANTS` list with the DBA before the pilot.

Status: `[P]`

### V2. Server version and read-only session support

```sql
SELECT VERSION() AS server_version;
-- MariaDB exposes the read-only state as @@tx_read_only, not
-- @@transaction_read_only.
SELECT @@tx_read_only AS tx_read_only_before;
SET SESSION TRANSACTION READ ONLY;
SELECT @@tx_read_only AS tx_read_only_after;
```

**Pass criteria:**

- The server is MySQL or MariaDB and supports read-only transactions.
- `SET SESSION TRANSACTION READ ONLY` succeeds.
- The application can set `max_execution_time` if supported. MariaDB may ignore
  it; record the behavior.

**Verified on Sra Bos:** MariaDB 10.5.29. `@@transaction_read_only` is not a
valid variable; `@@tx_read_only` exists and returned 0 before the SET. The
`SET SESSION TRANSACTION READ ONLY` statement works. The connector must use the
SET statement, not the variable name, to enforce read-only mode.

Status: `[P]`

### V3. Character set and collation

```sql
SHOW VARIABLES LIKE 'character_set_database';
SHOW VARIABLES LIKE 'collation_database';
SHOW VARIABLES LIKE 'character_set_connection';
SHOW VARIABLES LIKE 'collation_connection';
```

**Pass criteria:**

- The encoding supports Thai text.
- The connection encoding used by the app matches the database or can convert
  safely.
- A Thai name search returns the expected test patient.

**Verified on Sra Bos:** The database and server connection use `tis620` with
`tis620_thai_ci`. Use `--default-character-set=tis620` in the MySQL CLI
verification. The Tauri connector sets `charset=utf8mb4` on its client
connection so MySQL converts TIS-620 column data to UTF-8 before sqlx's
`String` decoder sees it. A `tis620` client charset makes sqlx fail with
`invalid utf-8 sequence`. Thai name search must be tested through the app as
well as through the CLI.

**Fallback:** If a future instance uses a different encoding, document it and
test Thai search with a known patient. Do not assume UTF-8.

Status: `[P]`

### V4. `officer` columns

```sql
SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'officer'
ORDER BY ORDINAL_POSITION;
```

Required candidates:

- `officer_id`
- `officer_login_name`
- `officer_login_password_md5`
- `officer_active`
- `officer_authentication_type_id`
- `officer_doctor_code`
- `officer_pname`
- `officer_fname`
- `officer_lname`

**Pass criteria:** All required columns exist with usable types. The plaintext
`officer_login_password` column exists but is never selected.

**Fallback:** If a display-name column is missing, use `doctor.name` or the
login name. If the MD5 column is missing, stop; the login design must change.

**Verified on Sra Bos:** 104 `officer` rows. The required columns exist. The
plaintext column exists but is never selected.

Status: `[P]`

### V5. `officer` data quality

```sql
SELECT COUNT(*) AS total,
       SUM(officer_login_name IS NULL OR officer_login_name = '') AS missing_login,
       SUM(officer_login_password_md5 IS NULL OR officer_login_password_md5 = '') AS missing_md5,
       SUM(officer_active IS NULL OR officer_active <> 'Y') AS not_active,
       SUM(officer_authentication_type_id IS NOT NULL
           AND officer_authentication_type_id <> 1) AS non_local_auth,
       SUM(officer_doctor_code IS NULL OR officer_doctor_code = '') AS missing_doctor_code
FROM officer;
```

Hash shape:

```sql
SELECT SUM(officer_login_password_md5 REGEXP '^[0-9A-Fa-f]{32}$') AS valid_shape,
       SUM(officer_login_password_md5 IS NOT NULL
           AND officer_login_password_md5 <> ''
           AND officer_login_password_md5 NOT REGEXP '^[0-9A-Fa-f]{32}$') AS bad_shape
FROM officer;
```

**Pass criteria:**

- Every active login has a 32-hex MD5 value.
- The test account is active.
- The test account has auth type 1 or NULL treated as 1.
- The test account has a doctor code if the prescriber is expected to come from
  the officer.

**Verified on Sra Bos:** 104 `officer` rows. Login names are present on 100%,
MD5 values on 100%, and the 32-hex shape on 104/104. `officer_active = 'Y'` on
all rows. `officer_authentication_type_id` is NULL on all 104 rows, which is
treated as local auth type 1. One row has no doctor code. The NULL auth type is
not a failure; it is the expected local-database state on this instance.

Status: `[P]`

### V6. MD5 password verification

Use one approved test account. Do not use a real clinician's password.

```sql
SET @test_login = 'test_account';
SELECT officer_login_name,
       officer_login_password_md5,
       MD5('the-test-password') AS computed_md5
FROM officer
WHERE officer_login_name = @test_login;
```

**Pass criteria:**

- `MD5('the-test-password')` equals the stored MD5, case-insensitively, for an
  approved test account.
- The password encoding is confirmed: ASCII, UTF-8, or TIS-620.
- A wrong password does not match.
- The app's `verify_password` unit tests use known MD5 values.

**Verified on Sra Bos:** The plaintext `officer_login_password` column has one
populated row, and `MD5(plaintext)` does not equal the stored MD5 column. The
plaintext column is not a source of truth and must never be selected. The MD5
algorithm and string encoding are still not source-confirmed. Test only with an
approved test account and have the DBA or pharmacist confirm the result. Do not
analyze or copy a real clinician password.

**Fallback:** If the algorithm is salted or uses a different encoding, stop and
redesign the login verification.

Status: `[B]`

### V7. Officer to doctor join

```sql
SELECT COUNT(*) AS officers_with_doctor_code,
       SUM(d.code IS NULL) AS orphan_doctor_codes
FROM officer o
LEFT JOIN doctor d ON d.code = o.officer_doctor_code
WHERE o.officer_doctor_code IS NOT NULL
  AND o.officer_doctor_code <> '';
```

**Pass criteria:** The join works and orphan codes are documented. The display
name fallback is safe.

**Verified on Sra Bos:** One officer has no doctor code. Orphan doctor codes are
0. The join is safe.

Status: `[P]`

## 3. Patient identity and search

### V8. `patient` columns

```sql
SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'patient'
ORDER BY ORDINAL_POSITION;
```

Required candidates:

- `hn`
- `cid`
- `pname`
- `fname`
- `lname`

Optional and verified candidates:

- `birthday` (the instance uses `birthday`, not `birthdate`)
- `sex`
- `passport_no`
- `nationality`
- `addrpart`
- `moopart`
- `tmbpart`
- `amppart`
- `chwpart`
- `road`
- `addr_soi`
- `addressid`
- `hometel`
- `informtel`
- `worktel`
- `mobile_phone_number`

**Not present on Sra Bos:** `birthdate`, `soi`, `home`, `tel`,
`mobile_phone`, and `phone_number`. Use the verified columns above.

**Pass criteria:**

- `hn` is unique and non-empty.
- `cid` exists and is usable for 13-digit search.
- The name fields support the official form.
- The address and phone columns are identified as present or absent.

**Verified on Sra Bos:** The identity columns pass. `hn` has a unique index.
`cid` has an index. `birthday` is the correct date column. Address and phone
columns are corrected above.

Status: `[P]`

### V9. Patient identity data quality

```sql
SELECT COUNT(*) AS total,
       SUM(hn IS NULL OR hn = '') AS missing_hn,
       SUM(cid IS NULL OR cid = '') AS missing_cid,
       SUM(cid IS NOT NULL AND cid <> ''
           AND cid NOT REGEXP '^[0-9]{13}$') AS bad_cid_shape,
       SUM(pname IS NULL OR pname = '') AS missing_pname,
       SUM(fname IS NULL OR fname = '') AS missing_fname,
       SUM(lname IS NULL OR lname = '') AS missing_lname
FROM patient;
```

Duplicate CID check:

```sql
SELECT COUNT(*) AS duplicate_cid_groups,
       SUM(row_count) AS duplicate_cid_rows
FROM (
  SELECT cid, COUNT(*) AS row_count
  FROM patient
  WHERE cid IS NOT NULL AND cid <> ''
  GROUP BY cid
  HAVING COUNT(*) > 1
) t;
```

**Pass criteria:**

- HN is present for every patient.
- CID is present for the test patient.
- CID is 13 digits for the test patient.
- Duplicate CIDs are either zero or documented with an operational rule.
- Name fields are usable for the form.

**Verified on Sra Bos:** 49,557 patient rows. CID is missing on 1,146 rows and
3 rows have a non-13-digit shape. There are 6 duplicate CID groups covering 19
rows. An operational rule is required before the pilot: the app must show the
patient identity for confirmation and must not silently merge duplicate CIDs.
Name fields are usable.

Status: `[P]`

### V10. Search by HN

```sql
SET @hn = '0000001';
SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE hn = @hn
LIMIT 20;
```

**Pass criteria:** Returns the approved test patient. Uses the HN index.

**Verified on Sra Bos:** The HN unique index is present. HN is 7 digits.

Status: `[P]`

### V11. Search by CID

```sql
SET @cid = '0000000000000';
SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE cid = @cid
LIMIT 20;
```

**Pass criteria:** Returns the approved test patient. Uses the CID index.

**Verified on Sra Bos:** `ix_cid` is present. CID shape caveats are recorded in
V9.

Status: `[P]`

### V12. Search by name prefix

```sql
SET @name = 'สมชาย';
SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE fname LIKE CONCAT(@name, '%')
   OR lname LIKE CONCAT(@name, '%')
   OR CONCAT_WS(' ', pname, fname, lname) LIKE CONCAT(@name, '%')
LIMIT 20;
```

**Pass criteria:** Returns the test patient and stays under 300 ms. The query
uses an index on `fname` or `lname` if available.

**Verified on Sra Bos:** `ix_fname` and `ix_lname` are present. Timing was not
reported; measure it during the pilot.

Status: `[P]`

### V13. Search by name contains

```sql
SET @name = 'สมชาย';
SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE fname LIKE CONCAT('%', @name, '%')
   OR lname LIKE CONCAT('%', @name, '%')
   OR CONCAT_WS(' ', pname, fname, lname) LIKE CONCAT('%', @name, '%')
LIMIT 20;
```

**Pass criteria:** Returns the test patient. This is the fallback only when the
prefix query returns no rows. It may be slower; record the timing.

**Verified on Sra Bos:** The fallback query works. Timing was not reported;
measure it during the pilot.

Status: `[P]`

### V14. Search input classification

The app classifies the search term before querying.

| Input | Classification | Rule |
|---|---|---|
| 13 digits | CID | Exact match |
| Digits within HN policy | HN | Exact match |
| Anything else | Name | Prefix first, contains fallback |

**Verified on Sra Bos:** HN is fixed at 7 digits. `MIN(LENGTH(hn))` and
`MAX(LENGTH(hn))` are both 7 across 49,557 rows. The sample `0000001` is a valid
7-digit shape. CID is exactly 13 digits.

**Pass criteria:** A test HN and a test CID are classified correctly. A name
that starts with digits is not misclassified as HN. The HN policy is 7 numeric
digits, not a generic digit range.

Status: `[P]`

## 4. Patient address

### V15. Address column types

```sql
SELECT COLUMN_NAME, COLUMN_TYPE, CHARACTER_MAXIMUM_LENGTH
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'patient'
  AND COLUMN_NAME IN (
    'addrpart', 'moopart', 'tmbpart', 'amppart', 'chwpart',
    'road', 'addr_soi', 'addressid', 'hometel', 'informtel',
    'worktel', 'mobile_phone_number'
  );
```

**Interpretation:**

- `CHAR(2)`, `CHAR(4)`, or numeric types usually mean codes.
- `VARCHAR` with Thai names usually means names.
- Mixed types are possible. Verify with a sample.

**Verified on Sra Bos:** The address code fields are present. The phone fields
are `hometel`, `informtel`, `worktel`, and `mobile_phone_number`. The old
`soi`, `home`, `tel`, `mobile_phone`, and `phone_number` names do not exist.

Status: `[P]`

### V16. Address lookup table discovery

```sql
SHOW TABLES LIKE '%address%';
SHOW TABLES LIKE '%province%';
SHOW TABLES LIKE '%amphur%';
SHOW TABLES LIKE '%district%';
SHOW TABLES LIKE '%tambon%';
SHOW TABLES LIKE '%subdistrict%';
SHOW TABLES LIKE '%zip%';
SHOW TABLES LIKE 'c_%';
SHOW TABLES LIKE 'patient_address%';
```

Candidate tables:

- `thaiaddress`
- `c_province`
- `c_district`
- `c_subdistrict`
- `patient_address`
- site-specific tables

**Pass criteria:** At least one reliable lookup strategy exists, or the
fallback is accepted.

**Verified on Sra Bos:** `thaiaddress` is the reliable lookup. `patient_address`
exists but has 0 rows, so it is not usable. The separate `c_*` tables are not
needed after the `thaiaddress` join passed.

Status: `[P]`

### V17. `thaiaddress` columns

```sql
SHOW COLUMNS FROM thaiaddress;
```

Verified columns:

- `addressid` varchar(6), formed as `chwpart + amppart + tmbpart`
- `name`
- `full_name`
- `pocode`
- `codetype`

**Not present on Sra Bos:** `addressname`, `moo`, and `zipcode`. The table
stores the subdistrict level, not the village level. `addressid` is 6 digits,
for example `160101`.

**Pass criteria:** The table has a key that can be derived from the patient
address codes, and the name fields cover subdistrict, district, province, and
postal code.

**Verified on Sra Bos:** The join key is `addressid`. `name` is the subdistrict
name. `pocode` is the postal code and is missing on 685 / 8,924 subdistricts
(7.7%). Confirm the `full_name` format before splitting it into district and
province.

Status: `[P]`

### V18. Address join candidate A: full address id (rejected)

The original candidate joined four levels and included `moopart`. It is wrong
for this instance: `thaiaddress` has no `moo` column and `addressid` is only 6
digits (`chwpart + amppart + tmbpart`).

Do not use the four-level LPAD join. It is retained here only as a record of
the rejected candidate.

```sql
-- Rejected. Do not use on Sra Bos.
SELECT p.hn, p.addrpart, p.moopart, p.tmbpart, p.amppart, p.chwpart,
       t.addressid, t.name, t.pocode
FROM patient p
LEFT JOIN thaiaddress t
  ON t.addressid = CONCAT(
       LPAD(COALESCE(NULLIF(p.chwpart, ''), '00'), 2, '0'),
       LPAD(COALESCE(NULLIF(p.amppart, ''), '00'), 2, '0'),
       LPAD(COALESCE(NULLIF(p.tmbpart, ''), '00'), 2, '0'),
       LPAD(COALESCE(NULLIF(p.moopart, ''), '00'), 2, '0')
     )
WHERE p.hn = @hn;
```

**Verified result:** Failed. The table has no village level and the address id
is 6 digits.

Status: `[F]`

### V19. Address join candidate B: subdistrict id (verified)

```sql
SET @hn = '0000001';
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
```

**Verified on Sra Bos:** The subdistrict-level join matched 49,550 / 49,557
patients (99.99%). This is the chosen address strategy.

`name` is the subdistrict name. `full_name` must be inspected before it is split
into district and province. `pocode` is missing on 685 / 8,924 subdistricts
(7.7%); the app must leave the postal code blank or allow manual entry when it
is missing.

Status: `[P]`

### V20. Address join candidate C: separate code tables (not needed)

The separate `c_province`, `c_district`, and `c_subdistrict` tables are not
needed after Candidate B passed. Keep this section as a fallback for a
different HOSxP instance.

Status: `[D]`

### V21. `patient_address` table

```sql
SHOW TABLES LIKE 'patient_address%';
```

If the table exists:

```sql
SHOW COLUMNS FROM patient_address;
```

**Verified on Sra Bos:** The table exists but has 0 rows. It is not usable and
must not be part of the address strategy. Do not spend time mapping its
columns.

Status: `[N]`

### V22. Address data quality

```sql
SELECT COUNT(*) AS total,
       SUM(addrpart IS NULL OR addrpart = '') AS missing_house_no,
       SUM(moopart IS NULL OR moopart = '') AS missing_moo,
       SUM(tmbpart IS NULL OR tmbpart = '') AS missing_subdistrict_code,
       SUM(amppart IS NULL OR amppart = '') AS missing_district_code,
       SUM(chwpart IS NULL OR chwpart = '') AS missing_province_code
FROM patient;
```

**Pass criteria:** The test patient has enough address data to print. The
overall missing rate is documented.

**Verified on Sra Bos:** The address code fields are present. The subdistrict
join covers 99.99% of patients. Postal code is missing on 7.7% of subdistricts,
so some patient forms will have no postal code. Leave it blank or allow manual
entry; do not guess.

**Fallback:** Print the available fields and leave the rest blank. The operator
may complete the form by hand. Do not print wrong names.

Status: `[P]`

### V23. Patient phone source

```sql
SELECT COLUMN_NAME, COLUMN_TYPE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'patient'
  AND COLUMN_NAME IN ('hometel', 'informtel', 'worktel', 'mobile_phone_number');
```

If the columns exist, check fill rates:

```sql
SELECT SUM(hometel IS NOT NULL AND hometel <> '') AS hometel_filled,
       SUM(informtel IS NOT NULL AND informtel <> '') AS informtel_filled,
       SUM(worktel IS NOT NULL AND worktel <> '') AS worktel_filled,
       SUM(mobile_phone_number IS NOT NULL AND mobile_phone_number <> '') AS mobile_filled
FROM patient;
```

**Verified on Sra Bos:** `hometel` is the primary source with 36,177 / 49,557
filled rows. `informtel` has 22,638. `worktel` exists. `mobile_phone_number` has
only 136 rows. Use `hometel`, then `informtel`, then `worktel`, then
`mobile_phone_number`; print one phone number, not a concatenation.

Status: `[P]`

## 5. Visits

### V24. `ovst` columns

```sql
SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'ovst'
ORDER BY ORDINAL_POSITION;
```

Required candidates:

- `vn`
- `hn`
- `vstdate`
- `vsttime`
- `doctor`
- `main_dep`

Optional:

- `spclty`
- `pttype`

**Verified on Sra Bos:** `vn`, `hn`, `vstdate`, `vsttime`, `doctor`,
`main_dep`, `spclty`, and `pttype` exist. `income` does not exist and must be
removed from the design. `doctor` is varchar(7). `ix_hn_vstdate` and
`ix_vn_unique` are present.

**Pass criteria:** The visit list can be built and the prescription date can be
taken from `vstdate`.

Status: `[P]`

### V25. Visit list query

```sql
SET @hn = '0000001';
SET @from = '2026-09-29';
SET @to = '2026-10-29';
SELECT v.vn, v.vstdate, v.vsttime, v.main_dep, v.spclty, v.doctor,
       d.name AS doctor_name
FROM ovst v
LEFT JOIN doctor d ON d.code = v.doctor
WHERE v.hn = @hn
  AND v.vstdate BETWEEN @from AND @to
ORDER BY v.vstdate DESC, v.vsttime DESC
LIMIT 50;
```

**Pass criteria:** Returns the test visit. The date range is bounded. The query
stays under 500 ms.

**Verified on Sra Bos:** `ix_hn_vstdate` and `ix_vn_unique` are present. Timing
was not reported; measure it during the pilot.

Status: `[P]`

### V26. Department name

```sql
SHOW COLUMNS FROM kskdepartment;
```

Candidate join:

```sql
SELECT v.vn, v.main_dep, k.department AS department_name
FROM ovst v
LEFT JOIN kskdepartment k ON k.depcode = v.main_dep
WHERE v.vn = @vn
LIMIT 5;
```

**Pass criteria:** The department name is available for the visit selector, or
the app shows the department code only. The official form does not print the
department, so this is optional.

**Verified on Sra Bos:** `kskdepartment.depcode` and `department` are present.

Status: `[P]`

### V27. Visit date versus order date

```sql
SET @vn = '0000001';
SELECT v.vn, v.vstdate, v.vsttime,
       MIN(o.rxdate) AS first_order_date,
       MAX(o.rxdate) AS last_order_date,
       MIN(o.rxtime) AS first_order_time,
       MAX(o.rxtime) AS last_order_time
FROM ovst v
LEFT JOIN opitemrece o ON o.vn = v.vn
WHERE v.vn = @vn
GROUP BY v.vn, v.vstdate, v.vsttime;
```

**Pass criteria:** The visit date and order date are understood. The app uses
the visit date on the form. If the order is from a different date, the
pharmacist must confirm the rule.

Status: `[ ]`

### V28. OPD versus IPD scope

```sql
SELECT COUNT(*) AS total,
       SUM(an IS NULL OR an = '') AS opd_rows,
       SUM(an IS NOT NULL AND an <> '') AS ipd_rows
FROM opitemrece
WHERE vn = '0000001';
```

**Pass criteria:** The selected visit is OPD and the order rows have an empty
`an`. If IPD rows exist, v1 excludes them and the decision is recorded.

Status: `[ ]`

## 6. Narcotic order lines

### V29. `opitemrece` columns

```sql
SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'opitemrece'
ORDER BY ORDINAL_POSITION;
```

Required and verified:

- `vn`
- `an`
- `hn`
- `icode`
- `qty` int(11), no decimal quantities
- `drugusage`
- `sp_use`
- `doctor`
- `rxdate`
- `rxtime`
- `item_no`
- `sub_type`
- `item_type`
- `hos_guid` varchar(38), PRIMARY KEY, NOT NULL

Optional:

- `dep_code`
- `vstdate`
- `vsttime`
- `pttype`

**Verified on Sra Bos:** `income` does not exist and is not used. `ix_vn` is
present. `hos_guid` is the primary key, so
line identity is always available and unique. `sub_type = '1'` is the drug
category: 210,479 / 210,480 rows join to `drugitems`. `sub_type = '3'` is
non-drug. `item_type = 'A'` has only 172 rows and represents vaccines or
services, so the filter is correct. `qty` is int(11), and 0 / 198,392 rows have
a decimal quantity.

**Pass criteria:** The query can select the order lines, filters, sig codes,
prescriber, and line identity.

Status: `[P]`

### V30. Order line data quality

```sql
SET @vn = '0000001';
SELECT COUNT(*) AS total,
       SUM(qty > 0) AS positive_qty,
       SUM(qty <= 0) AS non_positive_qty,
       SUM(an IS NULL OR an = '') AS opd_rows,
       SUM(an IS NOT NULL AND an <> '') AS ipd_rows,
       SUM(drugusage IS NULL OR drugusage = '') AS missing_drugusage,
       SUM(sp_use IS NULL OR sp_use = '') AS missing_sp_use
FROM opitemrece
WHERE vn = @vn;
```

**Pass criteria:** The selected visit has at least one positive quantity order.
The filter behavior is understood.

**Verified on Sra Bos:** `sub_type = '1'` is drug (210,479 / 210,480 rows in
`drugitems`). `sub_type = '3'` is non-drug. `item_type = 'A'` has 172 rows.

Status: `[P]`

### V31. `drugitems` columns

```sql
SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'drugitems'
ORDER BY ORDINAL_POSITION;
```

Required candidates:

- `icode`
- `name`
- `strength`
- `units`

Optional and verified:

- `trade_name`
- `dosageform`
- `therapeutic`
- `generic`
- `drug_control_type_id`

**Not present on Sra Bos:** `istype`, `drug_category_id`, `narcotic`,
`psychotropic`, and `drug_control`.

**Verified on Sra Bos:** Generic name, strength, and units are available.
`drug_control_type_id` exists and points to `drug_control_type`, but it is empty
on all 659 drugitems. The catalog manual field is the only usable legal-category
source. The drug-type filter `istype` from generic HOSxP must not be used.

**Pass criteria:** Generic name, strength, and units are available. The trade
name and legal category source are either available or explicitly handled by
the catalog.

Status: `[P]`

### V32. Configured icode existence

```sql
SET @icode1 = '1000001';
SET @icode2 = '1000002';
SELECT icode, name, trade_name, strength, units, dosageform,
       drug_control_type_id
FROM drugitems
WHERE icode IN (@icode1, @icode2);
```

**Pass criteria:** Every configured icode exists. No icode points to a
non-drug item. The pharmacist has approved the list.

Status: `[ ]`

### V33. Legal category source

```sql
SHOW COLUMNS FROM drugitems LIKE '%drug_control%';
SHOW COLUMNS FROM drug_control_type;
SELECT * FROM drug_control_type LIMIT 20;

SELECT drug_control_type_id, COUNT(*) AS item_count
FROM drugitems
GROUP BY drug_control_type_id;

-- Exists but has 0 rows. Do not use as a source.
SELECT COUNT(*) AS opitemrece_narcotic_rows
FROM opitemrece_narcotic;
```

**Verified on Sra Bos:**

- `drugitems.istype`, `narcotic`, `psychotropic`, and `drug_category_id` do not
  exist.
- `drugitems.drug_control_type_id` exists and points to `drug_control_type`.
  The lookup table contains NC1 to NC5 and HF1 to HF4, plus `print_document` and
  `report_template_name`.
- `drug_control_type_id` is empty on all 659 drugitems (0%).
- `opitemrece_narcotic` exists but has 0 rows. It cannot confirm controlled-drug
  dispensing.
- The legal category cannot be derived from HOSxP on this instance.

**Pass criteria:**

- A reliable source maps each configured icode to Category 2 narcotic or
  Category 2 psychotropic, or
- The pharmacist confirms that the catalog manual field is the source of truth.

**Rule:** The app must not infer the legal category from the drug name. The
catalog `law_category` field is mandatory for every configured icode and must be
filled before the pilot. This is blocker B4.

Status: `[B]`

### V34. `drugusage` and `sp_use` columns

```sql
SHOW COLUMNS FROM drugusage;
SHOW COLUMNS FROM sp_use;
```

Required candidates:

- `drugusage`
- `name1`
- `name2`
- `name3`

The same for `sp_use`.

Fill rates:

```sql
SELECT COUNT(*) AS total,
       SUM(name1 IS NOT NULL AND name1 <> '') AS name1_filled,
       SUM(name2 IS NOT NULL AND name2 <> '') AS name2_filled,
       SUM(name3 IS NOT NULL AND name3 <> '') AS name3_filled
FROM drugusage;
```

**Verified on Sra Bos:** `drugusage.name1` is filled on 98.8% of rows and
`sp_use.name1` on 99.7%. The text columns are usable. The app still needs the
catalog default sig because some order lines have no sig (see V39).

**Pass criteria:** At least one text column provides a usable sig. The preferred
column order is confirmed against real data.

Status: `[P]`

### V35. Order line query

```sql
SET @vn = '0000001';
SET @icode1 = '1000001';
SET @icode2 = '1000002';
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
```

**Pass criteria:**

- Returns the expected configured narcotic lines.
- Non-drug and automatic charge lines are excluded.
- The sig text is present or the catalog default is available.
- The prescriber and licence are present or the fallback is known.
- The query stays under 500 ms.

**Verified on Sra Bos:** `o.hos_guid` is the primary key, and
`doc.licenseno` exists. Keep both. If `d.trade_name` is absent on another
instance, remove that column and use the catalog trade name. The filters
`sub_type = '1'` and `item_type <> 'A'` are correct.

Status: `[P]`

### V36. Filter comparison

Run the query with and without each filter for the test visit.

```sql
SET @vn = '0000001';
SELECT
  SUM(1) AS all_lines,
  SUM(sub_type = '1') AS sub_type_drug_lines,
  SUM(IFNULL(item_type, '') = 'A') AS automatic_charge_lines,
  SUM(qty > 0) AS positive_lines,
  SUM(an IS NULL OR an = '') AS opd_lines
FROM opitemrece
WHERE vn = @vn;
```

**Verified on Sra Bos:** `sub_type = '1'` is the drug category, with
210,479 / 210,480 rows present in `drugitems`. `sub_type = '3'` is non-drug.
`item_type = 'A'` has only 172 rows and represents vaccines or services. The
filters remove the expected rows.

**Pass criteria:** The filters remove the expected non-drug and correction rows.
The count of configured narcotic lines matches the pharmacist's expectation.

Status: `[P]`

### V37. Line identity and duplicates

```sql
SET @vn = '0000001';
SELECT COUNT(*) AS total_lines,
       COUNT(DISTINCT hos_guid) AS distinct_hos_guid,
       SUM(hos_guid IS NULL OR hos_guid = '') AS missing_hos_guid
FROM opitemrece
WHERE vn = @vn;
```

Duplicate configured lines:

```sql
SET @vn = '0000001';
SET @icode1 = '1000001';
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
```

**Verified on Sra Bos:** `hos_guid` is the PRIMARY KEY (varchar(38), NOT NULL).
It cannot be NULL or duplicated. The fallback `vn + item_no + icode` is not
needed. Identical `(icode, sig, doctor)` lines can still be merged by the core
when the source has multiple lines.

**Pass criteria:** The line identity is known and unique. Identical lines can be
merged safely.

Status: `[P]`

### V38. Quantity and unit

```sql
SET @vn = '0000001';
SELECT o.icode, d.name, d.units,
       o.qty, CAST(o.qty AS CHAR) AS qty_text,
       SUM(o.qty <> FLOOR(o.qty)) AS decimal_quantity_rows
FROM opitemrece o
JOIN drugitems d ON d.icode = o.icode
WHERE o.vn = @vn
  AND o.icode IN (@icode1, @icode2)
  AND o.qty > 0
GROUP BY o.icode, d.name, d.units, o.qty;
```

**Verified on Sra Bos:** `qty` is int(11). There are 0 / 198,392 decimal
quantities. The app does not need to support decimal quantities in v1. The
print unit still needs a pharmacist-approved mapping for each icode.

**Pass criteria:**

- The quantity format is understood.
- The unit mapping for each configured icode is approved.
- Decimal quantities are allowed or blocked by the pharmacist.

Status: `[P]`

### V39. Sig fill rate for configured lines

```sql
SET @vn = '0000001';
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
```

**Verified on Sra Bos:** Sig text is missing on 19,933 / 210,480 lines (9.5%).
`sp_use` is missing on 90.5% of lines. The catalog default sig is necessary and
must be pharmacist-approved for every configured icode.

**Pass criteria:** The app knows how many lines will fall back to the catalog
default sig. The pharmacist has approved the defaults.

Status: `[P]`

### V40. Configured icode coverage

```sql
SET @vn = '0000001';
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
```

**Pass criteria:** The result helps the pharmacist confirm that the configured
icode list covers the narcotics actually ordered at this hospital. Any missing
icode is added to the catalog before the pilot.

Status: `[ ]`

## 7. Prescriber and licence

### V41. `doctor` columns

```sql
SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE()
  AND TABLE_NAME = 'doctor'
ORDER BY ORDINAL_POSITION;
```

Licence candidates:

```sql
SHOW COLUMNS FROM doctor LIKE '%licen%';
SHOW COLUMNS FROM doctor LIKE '%license%';
```

**Verified on Sra Bos:** `doctor.licenseno` exists as varchar(50) with an
index. It is the verified licence column. `license_no` is not needed on this
instance. 16 / 293 doctors have no licence number; the session-only manual
entry fallback remains necessary.

**Pass criteria:** A licence column is identified. If none exists, the manual
session-only licence entry is the fallback and the pharmacist approves it.

Status: `[P]`

### V42. Licence data quality

Use the verified licence column. Example with `licenseno`:

```sql
SELECT COUNT(*) AS total,
       SUM(licenseno IS NULL OR licenseno = '') AS missing_licence,
       SUM(licenseno IS NOT NULL AND licenseno <> ''
           AND LENGTH(licenseno) < 3) AS suspicious_short_values
FROM doctor;
```

**Verified on Sra Bos:** 16 / 293 doctors have no licence number. The format is
understood. The app must block printing when the selected prescriber has no
licence and no session-only manual entry.

**Pass criteria:** The test prescriber has a licence number. The format is
understood. Missing values are documented.

Status: `[P]`

### V43. Order doctor versus visit doctor

```sql
SET @vn = '0000001';
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
```

**Verified on Sra Bos:** `o.doctor`, `v.doctor`, `doctor.name`, and
`doctor.licenseno` exist. The order-doctor-first fallback is confirmed. The
missing rates are recorded in V44.

**Pass criteria:** The app's prescriber source order is confirmed. If the order
doctor is empty, the visit doctor is used. If both are empty, manual selection
is required.

Status: `[P]`

### V44. Missing prescriber rate

```sql
SET @vn = '0000001';
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
```

**Verified on Sra Bos:** The order doctor is missing on 317 / 156,938 lines
(0.2%). The visit doctor is missing on 132 lines. The manual selection fallback
is necessary and the pharmacist must accept the policy.

**Pass criteria:** The fallback path is known. The pharmacist accepts the
manual selection policy.

Status: `[P]`

### V45. Officer doctor code

```sql
SET @test_login = 'test_account';
SELECT o.officer_login_name, o.officer_doctor_code,
       d.name AS doctor_name, d.licenseno AS licence_no
FROM officer o
LEFT JOIN doctor d ON d.code = o.officer_doctor_code
WHERE o.officer_login_name = @test_login;
```

**Pass criteria:** The app knows whether the signed-in officer is also a doctor.
The app never assumes the pharmacist is the prescriber.

**Verified on Sra Bos:** One officer has no doctor code; orphan doctor codes are
0. The join is safe.

Status: `[P]`

## 8. Hospital profile

### V46. Hospital profile source

```sql
SHOW TABLES LIKE 'hospital%';
SHOW TABLES LIKE 'hosp%';
SHOW TABLES LIKE 'hos%';
```

If a candidate exists:

```sql
SHOW TABLES LIKE 'hospital_profile%';
SHOW COLUMNS FROM hospital_profile;
SELECT * FROM hospital_profile LIMIT 5;
```

**Verified on Sra Bos:** `hospital_profile` exists with 1 row. The hospital name
is "สระโบสถ์", phone 036439102, fax 036 641288. The database is a better source
than a settings-only profile.

The address is incomplete: `amppart` and `tmbpart` are empty, and `address1`
contains "อ.เมือง จ.ลพบุรี", which conflicts with the hospital name
(อ.สระโบสถ์). The pharmacist must verify and correct the hospital address before
the first real print. The settings screen may override the database profile for
the printed header.

**Pass criteria:** A reliable hospital profile source exists, and the address is
verified by the pharmacist before printing. No patient data is involved.

Status: `[P]`

## 9. Catalog settings

### V47. Catalog completeness

For each configured icode:

```sql
SET @icode1 = '1000001';
SET @icode2 = '1000002';
SELECT icode, name, trade_name, strength, units, dosageform,
       drug_control_type_id
FROM drugitems
WHERE icode IN (@icode1, @icode2);
```

**Verified on Sra Bos:** `drug_control_type_id` is empty on all 659 drugitems.
The legal category cannot be prefilled from HOSxP. The catalog manual fields
must be completed and signed off by the pharmacist before the pilot.

**Pass criteria:**

- Every icode exists.
- Every icode has a generic name and strength.
- Every icode has a pharmacist-approved print unit.
- Every icode has a pharmacist-approved legal category.
- Every icode has a default sig if the order sig may be empty.
- The catalog has no duplicates.
- The catalog is signed off by the pharmacist.

Status: `[B]`

### V48. Catalog law category sign-off

This is a pharmacist task, not a database check.

| icode | Generic name | Trade name | Strength | Print unit | Law category | Default sig | Approved by |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

**Rule:** The legal category is not inferred from the drug name. The pharmacist
writes it explicitly.

**Verified on Sra Bos:** The database cannot supply the legal category because
`drug_control_type_id` is empty on all 659 drugitems. The pharmacist sign-off is
the only path. This is blocker B4.

Status: `[B]`

## 10. Indexes, encoding, and performance

### V49. Index inventory

```sql
SHOW INDEX FROM patient;
SHOW INDEX FROM ovst;
SHOW INDEX FROM opitemrece;
SHOW INDEX FROM drugitems;
SHOW INDEX FROM doctor;
```

**Pass criteria:**

- `patient` has an index on `hn` and `cid`.
- `patient` has usable indexes on `fname` and `lname` for prefix search.
- `ovst` has an index on `(hn, vstdate)` or equivalent.
- `opitemrece` has an index on `(vn, icode)` or equivalent.
- `drugitems` has a primary key on `icode`.
- `doctor` has a primary key on `code`.

**Verified on Sra Bos:** `patient.hn` has a unique index, `ix_cid` exists,
and `ix_fname` / `ix_lname` exist. `ovst` has `ix_hn_vstdate` and
`ix_vn_unique`. `opitemrece` has `ix_vn`. `drugitems` has a primary key on
`icode`. `doctor` has a primary key on `code` and an index on `licenseno`.

**Fallback:** If an index is missing, request it from the DBA as a separate
task. Do not create it from the app.

Status: `[P]`

### V50. Query plans

```sql
EXPLAIN SELECT hn, cid, pname, fname, lname, birthday, sex
FROM patient
WHERE hn = '0000001';

EXPLAIN SELECT vn, vstdate, vsttime, doctor, main_dep
FROM ovst
WHERE hn = '0000001'
  AND vstdate BETWEEN '2026-09-29' AND '2026-10-29';

EXPLAIN SELECT o.vn, o.icode, o.qty, o.drugusage, o.sp_use, o.doctor
FROM opitemrece o
WHERE o.vn = '0000001'
  AND o.icode IN ('1000001', '1000002')
  AND o.qty > 0;
```

**Pass criteria:** The plans use the expected indexes. Patient search under
300 ms. Visit and order queries under 500 ms.

Status: `[ ]`

### V51. Thai search encoding

Use a Thai name from an approved test patient.

```sql
SET @name = 'สมชาย';
SELECT hn, pname, fname, lname
FROM patient
WHERE fname LIKE CONCAT(@name, '%')
LIMIT 5;
```

**Verified on Sra Bos:** The database and server connection use `tis620` /
`tis620_thai_ci`. Run the CLI query with `--default-character-set=tis620`.
The Tauri connector uses `charset=utf8mb4` so MySQL converts TIS-620 column data
to UTF-8 for sqlx. The Thai name search works with the verified test name.

**Pass criteria:** The query returns the expected row and does not return
mojibake or zero rows because of encoding.

Status: `[P]`

### V52. Order validity and cancellation

A cancelled or voided order must not be printed on a prescription form.

```sql
SHOW COLUMNS FROM opitemrece LIKE '%status%';
SHOW COLUMNS FROM opitemrece LIKE '%confirm%';
SHOW COLUMNS FROM opitemrece LIKE '%cancel%';
SHOW COLUMNS FROM opitemrece LIKE '%void%';
```

If a candidate column exists, inspect its values for the test visit:

```sql
SET @vn = '0000001';
SELECT <candidate_column> AS status_value, COUNT(*) AS line_count
FROM opitemrece
WHERE vn = @vn
GROUP BY <candidate_column>;
```

**Pass criteria:**

- The app knows which status values mean a valid order.
- Cancelled, voided, or unconfirmed orders are excluded, or the pharmacist
  confirms that `qty > 0` plus manual review is sufficient.
- The filter is documented in the order query.

**Verified on Sra Bos:** `opitemrece_narcotic` exists but has 0 rows. It cannot
be used as a source of controlled-dispensing confirmation. The order-status
column, if any, still needs verification. Until then, keep `qty > 0`, the drug
join, and the manual review gate.

**Fallback:** If there is no status column, keep `qty > 0`, the drug join, and
the review gate. Record the residual risk.

Status: `[B]`

## 11. Print and legal verification

These items are not HOSxP schema, but they block the print implementation.

| ID | Item | Source of truth | Status |
|---|---|---|---|
| P1 | Official form version is current | Pharmacist / Narcotics Control Division | `[ ]` |
| P2 | Both category checkboxes may appear on one form | Pharmacist / legal | `[ ]` |
| P3 | Five rows per form is acceptable | Pharmacist | `[ ]` |
| P4 | More than five rows: split or continuation | Pharmacist / legal | `[ ]` |
| P5 | Number of copies: original and copy | Pharmacist / hospital policy | `[ ]` |
| P6 | Prescription number, book number, or stamp | Pharmacist / hospital policy | `[ ]` |
| P7 | Patient address mandatory or handwritten allowed | Pharmacist | `[ ]` |
| P8 | Patient phone mandatory or optional | Pharmacist | `[ ]` |
| P9 | Prescriber licence mandatory | Pharmacist | `[ ]` |
| P10 | Handwritten signature required | Pharmacist / legal | `[ ]` |
| P11 | Hospital stamp required | Pharmacist / hospital policy | `[ ]` |
| P12 | Patient CID or alternative official ID | Pharmacist / legal | `[ ]` |
| P13 | Trade name column mandatory or optional | Pharmacist | `[ ]` |
| P14 | Decimal quantities allowed | Pharmacist | `[ ]` |
| P15 | Approved print units | Pharmacist | `[ ]` |
| P16 | IPD in scope | Pharmacist | `[ ]` |
| P17 | Local print audit log required | Pharmacist / privacy | `[ ]` |
| P18 | Retention policy for printed forms and PDFs | Hospital policy | `[ ]` |
| P19 | Approved print font | Pharmacist / hospital | `[ ]` |
| P20 | Who may print | Hospital policy | `[ ]` |

The full legal checklist with explanations is in
[`form-spec.md`](form-spec.md) section 6.

## 12. Schema contract

This table maps the planned query constants to their sources and status. The
connector code must not be written until the row is `[P]`.

| Query constant | Tables and columns | Required | Fallback | Status |
|---|---|---|---|---|
| `PING_SQL` | None | Yes | None | `[P]` |
| `FETCH_OFFICER_SQL` | `officer` login, hash, active, auth type, doctor code, name fields; `doctor.name` | Yes | Login name | `[B]` MD5 test account pending |
| `PATIENT_SEARCH_BY_HN` | `patient` hn, cid, name, birthday, sex | Yes | None | `[P]` |
| `PATIENT_SEARCH_BY_CID` | `patient` cid, hn, name, birthday, sex | Yes | None | `[P]` |
| `PATIENT_SEARCH_NAME_PREFIX` | `patient` fname, lname, full name | Yes | None | `[P]` |
| `PATIENT_SEARCH_NAME_CONTAINS` | `patient` fname, lname, full name | Yes | None | `[P]` |
| `PATIENT_BY_HN_SQL` | `patient` identity and address fields | Yes | Minimal identity | `[P]` |
| `PATIENT_ADDRESS_SQL` | `patient` address fields plus `thaiaddress` subdistrict join | Yes | Manual edit | `[P]`; `full_name` split and missing `pocode` need handling |
| `VISITS_BY_HN_SQL` | `ovst` vn, hn, vstdate, vsttime, doctor, main_dep | Yes | Drop department name | `[P]` |
| `NARCOTIC_ORDERS_BY_VN_SQL` | `opitemrece`, `drugitems`, `drugusage`, `sp_use`, `doctor`, verified order validity filter | Yes | Drop optional columns; manual review if no status column | `[B]` order validity pending |
| `DRUG_SEARCH_SQL` | `drugitems` icode, name, trade_name, strength, units | Yes | Drop trade_name | `[P]`; no `istype` filter |
| `DOCTOR_BY_CODE_SQL` | `doctor` code, name, `licenseno` | Yes | Session-only manual licence | `[P]` |
| `DOCTOR_SEARCH_SQL` | `doctor` code, name, `licenseno` | Yes | None | `[P]` |
| `HOSPITAL_PROFILE_SQL` | `hospital_profile` | No | Settings override | `[P]`; address correction pending |

### 12.1 Fallback rules

- If an optional column is missing, use a query variant without it. Do not
  substitute a similarly named column without verification.
- If a required column is missing, stop and update the design.
- If a join returns ambiguous or wrong address names, use the manual fallback.
- If `thaiaddress.pocode` is missing, leave the postal code blank or allow
  manual entry. Do not guess.
- If `full_name` cannot be split reliably, use `name` for the subdistrict and
  allow manual entry for district and province.
- If the legal category is unavailable in HOSxP, the catalog field is the only
  source. `drug_control_type_id` is empty and must not be used.
- If the prescriber licence is unavailable, use session-only manual entry.
- If the hospital profile address conflicts with the hospital name, use the
  pharmacist-verified settings override.
- Do not use `patient_address` or `opitemrece_narcotic`; both are empty.
- `hos_guid` is the primary key; no line-identity fallback is needed.

### 12.2 `SCHEMA-UNVERIFIED` markers

Every SQL constant that touches an unverified table or column must start with:

```rust
// SCHEMA-UNVERIFIED: see docs/verify-schema.md V<number>
```

Remove the marker only when the verification log is signed off.

## 13. Verification results log

Fill one row per check. Use `[P]`, `[F]`, `[N]`, `[D]`, or `[B]`.

| ID | Area | Check | Result | Evidence or note | Verifier | Date |
|---|---|---|---|---|---|---|
| V1 | Access | Read-only user and grants | `[P]` | Verification run completed; confirm the grant list with the DBA | DB check | 2026-10-29 |
| V2 | Access | Read-only session support | `[P]` | MariaDB 10.5.29; use `@@tx_read_only`; SET works | DB check | 2026-10-29 |
| V3 | Access | Character set and collation | `[P]` | DB/server `tis620`; CLI `tis620`; app client `utf8mb4` for sqlx | DB check | 2026-10-29 |
| V4 | Auth | `officer` columns | `[P]` | 104 rows; required columns present | DB check | 2026-10-29 |
| V5 | Auth | `officer` data quality | `[P]` | Login 100%, MD5 100%, 32-hex 104/104, active Y all, auth type NULL = local | DB check | 2026-10-29 |
| V6 | Auth | MD5 verification | `[B]` | Plaintext column has 1 row and does not match MD5; test account confirmation pending | DB check | 2026-10-29 |
| V7 | Auth | Officer to doctor join | `[P]` | Doctor code missing 1; orphan codes 0 | DB check | 2026-10-29 |
| V8 | Patient | `patient` columns | `[P]` | Identity pass; address and phone columns corrected | DB check | 2026-10-29 |
| V9 | Patient | Identity data quality | `[P]` | CID missing 1,146; bad shape 3; 6 duplicate groups / 19 rows; operational rule needed | DB check | 2026-10-29 |
| V10 | Patient | Search by HN | `[P]` | HN unique index; 7-digit policy | DB check | 2026-10-29 |
| V11 | Patient | Search by CID | `[P]` | `ix_cid` present | DB check | 2026-10-29 |
| V12 | Patient | Search by name prefix | `[P]` | `ix_fname` and `ix_lname` present | DB check | 2026-10-29 |
| V13 | Patient | Search by name contains | `[P]` | Fallback query works; timing not reported | DB check | 2026-10-29 |
| V14 | Patient | Search input classification | `[P]` | HN is fixed at 7 digits; CID is 13 digits | DB check | 2026-10-29 |
| V15 | Address | Address column types | `[P]` | `addr_soi`, `addressid`, `hometel`, `informtel`, `worktel`, `mobile_phone_number` | DB check | 2026-10-29 |
| V16 | Address | Lookup table discovery | `[P]` | `thaiaddress` usable; `patient_address` empty | DB check | 2026-10-29 |
| V17 | Address | `thaiaddress` columns | `[P]` | `addressid` varchar(6), `name`, `full_name`, `pocode`, `codetype` | DB check | 2026-10-29 |
| V18 | Address | Join candidate A | `[F]` | Four-level join is wrong; no `moo`, addressid is 6 digits | DB check | 2026-10-29 |
| V19 | Address | Join candidate B | `[P]` | `CONCAT(chwpart,amppart,tmbpart)` matched 49,550 / 49,557 (99.99%) | DB check | 2026-10-29 |
| V20 | Address | Join candidate C | `[D]` | Not needed; Candidate B passed | DB check | 2026-10-29 |
| V21 | Address | `patient_address` table | `[N]` | Table exists but has 0 rows | DB check | 2026-10-29 |
| V22 | Address | Address data quality | `[P]` | Subdistrict join 99.99%; `pocode` missing 7.7% | DB check | 2026-10-29 |
| V23 | Address | Patient phone source | `[P]` | `hometel` 36,177; `informtel` 22,638; `worktel`; `mobile_phone_number` 136 | DB check | 2026-10-29 |
| V24 | Visit | `ovst` columns | `[P]` | `income` absent; `pttype`, `spclty`, `main_dep`, `doctor` present | DB check | 2026-10-29 |
| V25 | Visit | Visit list query | `[P]` | `ix_hn_vstdate`, `ix_vn_unique` | DB check | 2026-10-29 |
| V26 | Visit | Department name | `[P]` | `kskdepartment.depcode` / `department` | DB check | 2026-10-29 |
| V27 | Visit | Visit date versus order date | `[ ]` | Not reported; confirm the form date rule | | |
| V28 | Visit | OPD versus IPD scope | `[ ]` | Not reported; v1 remains OPD only | | |
| V29 | Order | `opitemrece` columns | `[P]` | `hos_guid` PK; `qty` int(11); `ix_vn`; `sub_type` and `item_type` present | DB check | 2026-10-29 |
| V30 | Order | Order line data quality | `[P]` | `sub_type='1'` drug; `sub_type='3'` non-drug | DB check | 2026-10-29 |
| V31 | Order | `drugitems` columns | `[P]` | `istype`, `narcotic`, `psychotropic`, `drug_category_id` absent; `drug_control_type_id` present | DB check | 2026-10-29 |
| V32 | Order | Configured icode existence | `[ ]` | Configured list pending pharmacist sign-off | | |
| V33 | Order | Legal category source | `[B]` | `drug_control_type_id` empty on 659/659; catalog manual mandatory; B4 | DB check | 2026-10-29 |
| V34 | Order | `drugusage` and `sp_use` | `[P]` | `name1` filled 98.8% / 99.7% | DB check | 2026-10-29 |
| V35 | Order | Order line query | `[P]` | Filters and columns verified; `hos_guid` and `licenseno` exist | DB check | 2026-10-29 |
| V36 | Order | Filter comparison | `[P]` | `sub_type='1'` 210,479 / 210,480 in `drugitems`; `item_type='A'` 172 rows | DB check | 2026-10-29 |
| V37 | Order | Line identity and duplicates | `[P]` | `hos_guid` is PRIMARY KEY; no fallback needed | DB check | 2026-10-29 |
| V38 | Order | Quantity and unit | `[P]` | `qty` int(11); 0 / 198,392 decimals; P14 resolved | DB check | 2026-10-29 |
| V39 | Order | Sig fill rate | `[P]` | Sig missing 19,933 / 210,480 (9.5%); `sp_use` missing 90.5%; default sig required | DB check | 2026-10-29 |
| V40 | Order | Configured icode coverage | `[ ]` | Pharmacist catalog review pending | | |
| V41 | Prescriber | `doctor` columns | `[P]` | `licenseno` varchar(50) with index | DB check | 2026-10-29 |
| V42 | Prescriber | Licence data quality | `[P]` | Missing 16 / 293; manual entry fallback | DB check | 2026-10-29 |
| V43 | Prescriber | Order doctor versus visit doctor | `[P]` | Columns verified; order-first fallback confirmed | DB check | 2026-10-29 |
| V44 | Prescriber | Missing prescriber rate | `[P]` | Order doctor missing 317 / 156,938 (0.2%); visit doctor missing 132 | DB check | 2026-10-29 |
| V45 | Prescriber | Officer doctor code | `[P]` | Doctor code missing 1; join safe | DB check | 2026-10-29 |
| V46 | Hospital | Hospital profile source | `[P]` | 1 row; name, phone, fax present; address conflict needs pharmacist correction | DB check | 2026-10-29 |
| V47 | Catalog | Catalog completeness | `[B]` | Legal category source empty; pharmacist must fill the catalog | DB check | 2026-10-29 |
| V48 | Catalog | Law category sign-off | `[B]` | Pharmacist sign-off pending | | |
| V49 | Performance | Index inventory | `[P]` | Patient, ovst, opitemrece, drugitems, doctor indexes confirmed | DB check | 2026-10-29 |
| V50 | Performance | Query plans | `[ ]` | Timing and EXPLAIN results not reported | | |
| V51 | Encoding | Thai search encoding | `[P]` | DB/server `tis620`; app client `utf8mb4`; Thai decode verified | DB check | 2026-10-29 |
| V52 | Order | Order validity and cancellation | `[B]` | `opitemrece_narcotic` has 0 rows; status column pending | DB check | 2026-10-29 |
| P14 | Print | Decimal quantities allowed | `[P]` | `qty` int(11); 0 / 198,392 decimals | DB check | 2026-10-29 |
| P1-P20 | Print | Other legal and form checks | `[ ]` | P14 resolved; pharmacist answers pending | | |

### 13.1 Sign-off

| Item | Value |
|---|---|
| HOSxP instance | `hos`, โรงพยาบาลสระโบสถ์ |
| Server version | MariaDB 10.5.29 |
| Database name | `hos` |
| Read-only user | Verification account (confirm the grant list with the DBA) |
| Verification date | 2026-10-29 |
| Verified by | DB check |
| Pharmacist approval | Pending |
| DBA approval | Pending |
| Remaining blockers | B1, B4, B5 to B11, B12 to B14 |

## 14. Blockers and decisions

These must be resolved before the related milestone.

| ID | Blocker | Blocks | Owner | Decision |
|---|---|---|---|---|
| B1 | MD5 password algorithm and encoding | Login | DBA / pharmacist | Pending test account; plaintext column does not match MD5 and is never used |
| B2 | Patient address join strategy | Form preview | DBA / pharmacist | Resolved: V19 subdistrict join `CONCAT(chwpart,amppart,tmbpart)` |
| B3 | Prescriber licence column | Form preview | DBA / pharmacist | Resolved: `doctor.licenseno` varchar(50) |
| B4 | Legal category mapping | Catalog and form | Pharmacist | Blocker: `drug_control_type_id` empty on 659/659; catalog manual mandatory |
| B5 | Official form version and copies | Print | Pharmacist / legal | Pending |
| B6 | Five-row limit and continuation | Print | Pharmacist / legal | Pending |
| B7 | Mixed category on one form | Print | Pharmacist / legal | Pending |
| B8 | IPD scope | Visit selection | Pharmacist | Pending; v1 remains OPD only |
| B9 | Local print audit log | Security | Pharmacist / privacy | Pending |
| B10 | Approved print font | Print | Pharmacist / hospital | Pending |
| B11 | Read-only user and indexes | Connector | DBA | Indexes pass; read-only user grant list to confirm |
| B12 | Hospital profile source | Form header | Pharmacist / DBA | `hospital_profile` 1 row; address conflict needs correction |
| B13 | Who may print | Authorization | Hospital policy | Pending |
| B14 | Retention policy for printed forms and PDFs | Operations | Hospital policy | Pending |

## 15. Recommended defaults until verified

These are design defaults, not schema facts.

- Use `hospital_profile` as the primary hospital source, with a settings
  override for the printed header. The pharmacist must correct the address
  before the first real print.
- Use the catalog as the source of truth for legal category, print unit, and
  default sig. `drug_control_type_id` is empty and must not be used.
- Use the selected visit date as the prescription date.
- Use OPD only in v1.
- Use `tis620` for CLI verification. The app uses a `utf8mb4` client charset so
  MySQL converts TIS-620 column data to UTF-8 for sqlx.
- Use `doctor.licenseno` as the prescriber licence source, with a session-only
  manual entry when it is missing.
- Use `hos_guid` as the order line identity. It is the primary key.
- Use `thaiaddress` joined by `CONCAT(chwpart, amppart, tmbpart)` for the
  subdistrict. Leave the postal code blank when `pocode` is missing.
- Use `hometel`, then `informtel`, then `worktel`, then
  `mobile_phone_number` for the patient phone.
- Do not use `patient_address` (0 rows), `opitemrece_narcotic` (0 rows), or
  `drug_control_type_id` (empty).
- Block printing when more than five drug lines are present.
- Block printing when a required field is missing.
- Warn but allow printing when the address, postal code, or phone is
  incomplete.
- Do not support decimal quantities in v1. `qty` is int(11).
- Leave the signature line blank.
- Use TH SarabunPSK if the hospital has it; otherwise use Sarabun and verify
  the print.
- Do not add a local print history in v1.

## 16. What to send back

When the verification is complete, send:

1. The filled results log from section 13.
2. The sign-off block from section 13.1.
3. The chosen address join and a masked sample output.
4. The chosen licence column and a masked sample format.
5. The pharmacist-approved catalog fields for the test icodes.
6. The answers to the print and legal checklist P1 to P20.
7. Any schema differences from this document.

Do not send real patient identifiers or credentials.
