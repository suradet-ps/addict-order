# Authentication - HOSxP Officer Login

Addict Order signs operators in with their HOSxP **`officer`** account, using
the same authentication model as the sibling er-refill project. HOSxP remains
read-only. The login session lives only in the Tauri process, and the app never
writes an audit row, session token, or login log to HOSxP.

## 1. Source of truth

| Table | Use | Notes |
|---|---|---|
| `officer` | The only login source | Login name, MD5 password, active flag, auth type, doctor code, name fields |
| `opduser` | Never read | Old V3 accounts; the HOSxP gateway blacklists it |
| `onlineuser` | Not used | Online status is not identity |
| `officer_activity_log` | Not written | HOSxP already has an audit trail |
| `user_jwt` | Never read | Not part of this design |
| `emp` | Not used | No employee join in v1 |

The app must verify these facts against the real instance using
[`verify-schema.md`](verify-schema.md) V4 to V7:

- `officer_login_name` is unique and non-empty.
- `officer_login_password_md5` is a 32-character hex digest.
- `officer_active` is `'Y'` for the test account.
- `officer_authentication_type_id` is 1 or NULL treated as 1.
- `officer_doctor_code` maps to `doctor.code` when present.
- The name fields are populated or the `doctor.name` fallback works.

**Verified on Sra Bos, 2026-10-29:** 104 `officer` rows. Login names, MD5
values, and the 32-hex shape are present on all 104 rows. `officer_active` is
`'Y'` on all rows. `officer_authentication_type_id` is NULL on all rows and is
treated as local auth type 1. One officer has no doctor code; orphan doctor
codes are 0. The database and server charset is `tis620` / `tis620_thai_ci`.
The app client uses `utf8mb4` so MySQL converts TIS-620 column data to UTF-8
for sqlx; this was verified with a Thai `doctor.name` read.

## 2. Flow

```text
Launch
  |
  +-- login gate (the first screen)
        |
        +-- "ตั้งค่า" opens the settings dialog
        |     +-- connection form
        |           host, port, database, read-only user, password
        |           save: connect -> SELECT 1 -> encrypt and save -> pool
        |
        +-- saved config is re-probed at startup
        |     +-- reconnect: load vault -> connect -> SELECT 1
        |     +-- on failure: the login attempt reports the connection error
        |
        +-- officer login (username + password)
              +-- fetch one officer row by login name
              +-- check active = 'Y'
              +-- check auth type = 1 (local database)
              +-- verify MD5(password) against officer_login_password_md5
              +-- store an in-memory SessionUser
              +-- open the main shell
```

The login screen handles only officer credentials. Connection setup lives in
the settings dialog. The connection must be usable before login succeeds. A
connection change clears the session and returns the operator to the login
gate.

## 3. Password verification

The app computes `MD5(password)` in Rust and compares it with the stored
32-hex digest, case-insensitively.

Rules:

- The password is moved into a `secrecy::SecretString` for the verification
  window.
- The password is never written to disk, logged, or returned to the frontend.
- The stored hash never crosses the IPC boundary.
- A missing, short, or non-hex digest never matches.
- The comparison is case-insensitive because HOSxP stores mixed case on some
  instances.
- The plaintext `officer_login_password` column is never selected.

### Unverified assumption

The schema survey can confirm the shape (MD5, 32 hex) but not the exact string
that HOSxP hashes. The app assumes `MD5(password)` with no salt. Before the
pilot, test one approved account with read-only access.

**Verified on Sra Bos, 2026-10-29:** The plaintext
`officer_login_password` column has one populated row, and `MD5(plaintext)` does
not equal the stored MD5 column. The plaintext column is not a source of truth
and must never be selected. The source analysis of the credential algorithm was
not available, so the DBA or pharmacist must confirm the login result with an
approved test account.

If the real algorithm differs, login fails for every account and this
assumption must be corrected first.

Thai passwords would also require confirming the source encoding (UTF-8,
TIS-620, or another). The database uses `tis620`, so a Thai password test must
include the encoding check.

## 4. Session and roles

- `SessionUser` is kept in `AppState.session` in the Tauri process only.
- Restarting the app requires signing in again.
- The frontend never receives the password hash.
- Every patient, visit, drug, catalog, and print command requires a session at
  the Tauri command boundary.
- Login, connection, session lookup, and health commands do not require a
  session.
- The initial role is `STAFF` for every signed-in officer.
- HOSxP group permissions are not a usable permission source in v1.
- Role-based print restrictions are a later milestone.

### The officer is not the prescriber

The signed-in officer is the operator of the app, not automatically the
prescriber on the official form.

- The prescriber comes from the selected order or visit.
- If the order and visit have no doctor, the operator may select a doctor from
  the verified `doctor` table.
- The officer's `officer_doctor_code` may be used only when the officer is a
  doctor and the order has no doctor, and the result must be shown for
  confirmation.
- A pharmacist can prepare and print the form, but the prescriber signs it by
  hand.

## 5. Query

The only login query is `FETCH_OFFICER_SQL` in
`crates/hosxp-connector/src/queries.rs`. The final column names must match the
verified schema.

```sql
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
LIMIT 1
```

This is a single SELECT with one bound parameter. `opduser` and the plaintext
password column never appear. The top bar shows the display name. If the officer
name fields and the doctor name are both blank, the backend falls back to the
login name.

## 6. Why there are no app tables in HOSxP

The design does not create `app_user`, `app_session`, `app_login_log`,
`app_role`, or `app_role_permission` in HOSxP.

Reasons:

- `CREATE TABLE` and `CREATE VIEW` are writes, and Addict Order must never
  write to HOSxP.
- Addict Order has no separate application database in v1.
- A persistent login log would duplicate HOSxP audit data and store staff
  identifiers locally.
- A local audit store needs a separate privacy and retention decision.

If the hospital approves a local application database later, it can be added
without touching HOSxP. That is a separate product decision.

## 7. Connection settings and health

- Connection credentials are encrypted at rest with encryptman.
- The master key lives in the OS keychain through encryptman-keyring.
- The password is a `SecretString` end to end in Rust.
- The saved connection is re-probed at startup.
- The top bar shows a connection dot with three states: connected,
  disconnected, and unconfigured.
- A connection change clears the pool, the session, and the patient draft.
- The connection settings dialog is available before login.
- The app does not display the password or the host on the login summary.

## 8. Error handling

The login command returns a typed error with a Thai message.

| Kind | Thai message example |
|---|---|
| `Auth` | ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง |
| `Auth` | บัญชีนี้ถูกปิดใช้งาน กรุณาติดต่อผู้ดูแลระบบ |
| `Auth` | บัญชีนี้ใช้การยืนยันตัวตนแบบ LDAP ซึ่งยังไม่เปิดใช้งาน |
| `NotConfigured` | ยังไม่ได้ตั้งค่าการเชื่อมต่อ HOSxP |
| `Connection` | เชื่อมต่อฐานข้อมูล HOSxP ไม่สำเร็จ กรุณาตรวจสอบการตั้งค่าการเชื่อมต่อและเครือข่าย |
| `Validation` | กรุณากรอกชื่อผู้ใช้ |
| `Validation` | กรุณากรอกรหัสผ่าน |

The frontend switches on the error kind, never on the message text. The
password is never included in an error.

## 9. Frontend login gate

- The login gate is the first screen.
- The left panel uses the brand gradient and the lockbox mark.
- The right panel has one card with username and password.
- The card has a settings action for the connection dialog.
- If the connection is not configured, the card shows an amber notice.
- The submit button is disabled until the connection is configured and the
  fields are filled.
- The password field is cleared after a successful login.
- The login error is shown inside the card, with the lock icon.
- The app shows a startup splash while settings, connection, and session are
  probed.
- The login gate never displays patient data.

## 10. Acceptance checklist

- [ ] Verify `MD5(password)` on one approved test account with read-only access.
- [ ] Confirm the password encoding (ASCII, UTF-8, or TIS-620).
- [x] Confirm `officer_authentication_type_id` is NULL on all 104 rows and is
      treated as local database auth type 1.
- [x] Confirm the display-name sources: `officer_pname`, `officer_fname`,
      `officer_lname`, and the `doctor.name` fallback.
- [x] Confirm `officer_doctor_code` is missing on 1 row and the doctor join has
      0 orphans.
- [ ] Test the active, inactive, wrong-password, and LDAP rejection paths.
- [ ] Confirm the read-only MySQL user can `SELECT officer` and `doctor`.
- [ ] Confirm the login session is cleared on logout, restart, and connection
      change.
- [ ] Confirm no password or hash appears in logs, errors, or the frontend.
- [x] Confirm the prescriber is not assumed to be the signed-in officer.
