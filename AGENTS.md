# AGENTS.md - Addict Order

This file tells coding agents and future contributors how to work on this
repository without breaking its legal, clinical, or security invariants. Read
this file, [`docs/architecture.md`](docs/architecture.md),
[`docs/auth.md`](docs/auth.md), and [`docs/DESIGN.md`](docs/DESIGN.md) before
writing code.

## 1. What this project is

Addict Order is a Tauri 2 desktop application for a hospital pharmacy. It reads
a HOSxP MySQL/MariaDB database in strict read-only mode, finds a patient by HN,
CID, or Thai name, loads the selected OPD visit, pulls the configured narcotic
icodes from that visit, and builds the official Thai **Y.S. 2 / W.C. 2**
prescription form (ใบสั่งจ่าย ย.ส. ๒/ว.จ. ๒) for the prescriber to sign.

The target form is a prescription, not a ward requisition. Use the Thai term
"ใบสั่งจ่าย" in user-facing text and on paper.

### Design goals

- One official document, printed correctly.
- Read-only HOSxP access with no exceptions.
- No transcription of data that HOSxP already holds.
- A pharmacist can verify every field before printing.
- The printed paper is the legal record. The app is only a document builder.
- The screen preview and the printed output share one DOM and one stylesheet.

### Non-goals

- Not a prescribing system or a clinical decision support system.
- Not a stock, inventory, or ward requisition system.
- Not a system that writes to HOSxP.
- Not a system that stores patient history on disk in v1.
- Not a system that prints signatures.
- Not a system that infers the legal category of a drug from its name.

## 2. Hard rules

These rules are non-negotiable.

1. **Never write to HOSxP.** No `INSERT`, `UPDATE`, `DELETE`, `ALTER`, `CREATE`,
   `DROP`, or any DDL. Not in production, not in tests, not "just once".
   All database access goes through `crates/hosxp-connector`, and that crate
   must remain SELECT-only end to end.
2. **Never log PHI or secrets.** Patient names, HN, CID, addresses, phone
   numbers, drug lines, usernames, passwords, and connection details must never
   appear in logs or error messages. Log timing and error classes only.
3. **Parameterized queries only.** Never build SQL by string concatenation.
   Dynamic `IN (?, ?, ...)` lists are built by repeating `?`; values are always
   bound.
4. **No schema guessing.** Every table or column that has not been verified
   against the real instance must carry a `// SCHEMA-UNVERIFIED:` comment and
   must be tracked in [`docs/verify-schema.md`](docs/verify-schema.md).
   Remove the marker only after the verification entry is signed off.
5. **Login reads `officer` only.** Never query `opduser`, never select
   `officer_login_password` (plaintext), never log credentials, and never
   persist a login token or login log to HOSxP.
6. **The app never prints a signature.** The prescriber signs the printed form
   by hand. Do not render a signature image, a signature font, or a generated
   signature.
7. **The official form is not a brand surface.** Do not put the app logo, brand
   colors, gradients, or marketing copy on the printed form. The printed form is
   black on white and follows the official layout.
8. **Catalog legal category is pharmacist-verified.** Each configured icode
   must carry an explicit `law_category` value of `NARCOTIC_2` or
   `PSYCHOTROPIC_2`. Do not infer it from the drug name, therapeutic group, or
   any unverified HOSxP column.
9. **Print only a confirmed snapshot.** The print button is disabled until the
   pharmacist confirms the patient, visit, prescriber, licence, hospital,
   lines, and date. Any edit after confirmation invalidates the snapshot.
10. **No patient data on disk in v1.** Settings may hold the hospital profile,
    the drug catalog, and print preferences. They must never hold patient
    identity, visit data, or prescription lines.

## 3. Workspace layout

```text
addict-order/
├── src-tauri/              # Tauri 2 shell: commands, state, settings, error mapping
├── crates/
│   ├── models/             # Domain types, Thai/BE formatting, form labels
│   ├── core/               # Validation, form builder, snapshot, repository trait, mock
│   └── hosxp-connector/    # The only crate allowed to talk to MySQL
├── app/                    # Leptos 0.8 CSR/WASM frontend
│   ├── src/
│   │   ├── components/
│   │   ├── api.rs
│   │   ├── state.rs
│   │   └── lib.rs
│   ├── style/
│   └── assets/
├── docs/
├── icon-master.svg
└── Cargo.toml
```

Dependency direction:

```text
app -> addict-order-core -> addict-order-models
hosxp-connector implements a repository trait defined in addict-order-core
```

`addict-order-core` must be testable with a mock repository. The default test
suite must not require a database.

## 4. Domain model

The exact Rust types will be finalized after schema verification. The intended
shape is:

- `PatientSummary`
  - `hn`, `cid`, `full_name_th`, `birth_date`, `sex`
  - used for search results; no address in the list
- `PatientDetail`
  - patient summary plus `phone` and `address: PatientAddress`
- `PatientAddress`
  - `house_no`, `moo`, `soi`, `road`, `subdistrict`, `district`, `province`,
    `postal_code`
  - every field is a string because HOSxP stores addresses inconsistently
- `VisitSummary`
  - `vn`, `hn`, `visit_date`, `visit_time`, `department_code`,
    `department_name`, `doctor_code`, `doctor_name`
- `NarcoticDrugConfig`
  - `icode`
  - `generic_name`
  - `trade_name`
  - `strength_text`
  - `print_unit` (Tab, Cap, Amp, Vial, Patch, mL, ...)
  - `law_category` (`NARCOTIC_2` or `PSYCHOTROPIC_2`)
  - `default_sig`
  - `sort_order`
  - `active`
- `PrescriptionLine`
  - `line_id` (source line identity, not printed)
  - `icode`
  - `generic_name`
  - `trade_name`
  - `strength_text`
  - `sig`
  - `quantity`
  - `print_unit`
  - `law_category`
  - `source` (`HosxpOrder` or `Manual`)
  - `source_vn`, `source_item_no`, `source_doctor_code`
  - `note` (internal only, never printed)
- `Prescriber`
  - `doctor_code`, `name`, `license_no`, `license_type`
- `HospitalProfile`
  - `name`, `house_no`, `moo`, `soi`, `road`, `subdistrict`, `district`,
    `province`, `postal_code`, `phone`, `fax`
- `PrescriptionDraft`
  - `patient`, `visit`, `prescriber`, `hospital`, `prescription_date`,
    `lines`, `category_flags`
- `ConfirmedPrescription`
  - an immutable snapshot of every field above plus `confirmed_at`
- `PrintPage`
  - `number`, `count`, `lines` (at most five)

### Thai and Buddhist-era formatting

All dates on screen and on paper are Buddhist era. HOSxP stores Gregorian
dates. Conversion happens at the display boundary only, in
`addict-order-models::thai`.

Use one shared implementation for:

- Thai month names.
- Buddhist year conversion.
- Thai digit formatting for row numbers.
- Quantity formatting with a maximum number of decimals.
- Address line assembly.

## 5. HOSxP integration

All schema assumptions are tracked in
[`docs/verify-schema.md`](docs/verify-schema.md). The connector must not be
written until the relevant section is verified.

### 5.1 Authentication

- Read one `officer` row by `officer_login_name`.
- Check `officer_active = 'Y'`.
- Check `IFNULL(officer_authentication_type_id, 1) = 1`.
- Compare `MD5(password)` with `officer_login_password_md5`,
  case-insensitively.
- Display name comes from `officer_pname`, `officer_fname`, `officer_lname`,
  falling back to `doctor.name` through `officer_doctor_code`, then to the login
  name.
- Never select `officer_login_password`.
- Never query `opduser`.
- The MD5 assumption is not source-confirmed. It must be tested with one real
  account before the pilot.
- The plaintext `officer_login_password` column has one populated row on Sra Bos
  and does not match the MD5 column. Never select it.
- The verified instance uses `tis620` / `tis620_thai_ci`. The CLI verification
  uses `--default-character-set=tis620`; the app client sets `charset=utf8mb4`
  so MySQL converts TIS-620 column data to UTF-8 for sqlx's `String` decoder.

### 5.2 Patient search

- Search by exact HN. HN is fixed at 7 digits on Sra Bos.
- Search by exact CID when the term is 13 digits.
- Search by name with a prefix match first, then a contains-match fallback.
- Limit results to 20.
- Escape `\`, `%`, and `_` in the operator's term before wrapping it in
  `%...%`.
- Mask CID in the result list. Show the full CID only on the selected patient
  card and the form preview.
- Use `hometel`, then `informtel`, then `worktel`, then `mobile_phone_number`
  for the patient phone. The old `tel`, `mobile_phone`, and `phone_number`
  columns do not exist.
- Use `thaiaddress` joined by `CONCAT(chwpart, amppart, tmbpart)` for the
  subdistrict. The table has no village level. `pocode` is missing on 7.7% of
  subdistricts.

### 5.3 Visits

- Read OPD visits from `ovst` by HN and a bounded date range.
- Default to today, with a 30-day lookback selector.
- The prescription date is the selected visit date. Do not allow free
  backdating.
- If IPD support is added later, it must be a separate verified milestone.

### 5.4 Narcotic order lines

- Filter by the selected `vn` and the configured icode list.
- Filter `qty > 0`.
- Filter `sub_type = '1'`.
- Filter `IFNULL(item_type, '') <> 'A'`.
- Filter `(an IS NULL OR an = '')` for OPD.
- Join `drugitems` so non-drug charge lines never appear.
- Use `hos_guid` for line identity. It is the PRIMARY KEY on Sra Bos; no
  fallback is needed.
- Keep lines with different sigs separate. Merge only identical
  `(icode, sig, doctor)` groups and sum the quantity.
- Do not filter by `dep_code`. The prescription can be ordered from any
  department.
- `qty` is int(11). Do not support decimal quantities in v1.
- Do not use `opitemrece_narcotic`; it has 0 rows.
- The legal category cannot be read from `drugitems`; use the catalog manual
  field.

### 5.5 Prescriber

- Prefer `opitemrece.doctor`.
- Fall back to `ovst.doctor`.
- If both are empty, require the operator to select a doctor.
- The logged-in officer is not automatically the prescriber. Use the officer's
  `doctor_code` only when the officer is a doctor and the order has no doctor,
  and show the result for confirmation.
- The licence number comes from `doctor.licenseno` (varchar(50), indexed on
  Sra Bos).
- If the licence number is missing, block printing until the operator enters it
  for this session. Do not store it in plain settings. 16 / 293 doctors have no
  licence.
- The order doctor is missing on 0.2% of lines, and the visit doctor is missing
  on 132 lines. The manual fallback is required.

### 5.6 Hospital profile

- Use `hospital_profile` as the primary source. It has 1 row on Sra Bos.
- The verified row has the hospital name, phone, and fax, but the address is
  incomplete and conflicts with the hospital name. The pharmacist must correct
  it before the first real print.
- Allow a settings override for the printed hospital header.
- The hospital profile is not patient data and may be stored in the plain
  settings file.

### 5.7 Verified schema blockers

- `drugitems.drug_control_type_id` is empty on all 659 drugitems. The catalog
  `law_category` is mandatory and must be signed off by the pharmacist.
- `opitemrece_narcotic` has 0 rows. Do not use it.
- `patient_address` has 0 rows. Do not use it.
- CID is missing on 1,146 patients. There are 3 bad CID shapes and 6 duplicate
  CID groups covering 19 rows. The app must show identity for confirmation and
  must not merge duplicates.
- Sig is missing on 9.5% of order lines. The catalog default sig is required.
- `thaiaddress.pocode` is missing on 7.7% of subdistricts. Leave the postal
  code blank or allow manual entry.

## 6. Strict read-only enforcement

Use the same layered model as er-refill:

1. **Database user:** a dedicated MySQL user with `GRANT SELECT` only on the
   required tables. This is the real security boundary.
2. **Session:** `SET SESSION TRANSACTION READ ONLY;` after connect, plus a
   best-effort `SET SESSION max_execution_time = 5000`.
3. **Application guard:** `readonly_guard.rs` rejects any statement that is not
   a single `SELECT` or `WITH`. It is comment-aware and semicolon-aware.
4. **Queries:** compile-time constants and bound parameters only.
5. **Tauri capabilities:** `core:default` plus the dialog plugin only. No fs or
   shell scopes.

Every SQL constant must have a guard test asserting that it is read-only.

## 7. Login and session

- The login gate is the first screen.
- Connection setup lives in the settings dialog, not on the login card.
- The saved connection is re-probed at startup.
- The session is memory-only and cleared on logout, restart, or connection
  change.
- The frontend never receives the password hash.
- Every patient, visit, drug, and print command requires a session at the Tauri
  command boundary.
- The initial role is `STAFF` for every signed-in officer. Role-based print
  restrictions are a later milestone.

## 8. Form workflow

1. The operator signs in.
2. The operator searches for a patient.
3. The operator selects a visit.
4. The backend reads the configured narcotic order lines for that visit.
5. `addict-order-core` maps the order lines to form lines using the configured
   catalog metadata.
6. The operator reviews the form preview:
   - patient identity and address
   - prescriber and licence
   - hospital profile
   - prescription date
   - legal category checkboxes
   - up to five drug rows
7. The operator confirms the snapshot.
8. The print button prints the confirmed snapshot with `window.print()`.
9. Any edit after confirmation clears the snapshot and disables printing.

### Validation before confirmation

Required:

- patient name
- CID or an alternative official ID
- prescriber name
- prescriber licence number
- hospital name
- prescription date
- at least one drug line
- at most five drug lines in v1
- generic name, strength, sig, quantity greater than zero, and print unit for
  every line
- a legal category for every line

Warnings that do not block printing:

- incomplete patient address
- missing patient phone
- missing hospital phone or fax
- missing trade name

The form can be completed by hand for warning fields. Required fields must be
complete before the confirmation dialog opens.

### Print gate

- The confirmation dialog shows a summary of the patient, visit, prescriber,
  licence, hospital, date, row count, and category flags.
- Confirming stores an immutable snapshot.
- The print button is enabled only for that snapshot.
- The print action re-checks that the current draft equals the snapshot.
- The printed form is the same DOM and CSS as the preview.
- The print stylesheet hides all application chrome and prints A4 portrait.
- The app never prints a signature.

## 9. Print standard

The official form is A4 portrait, one page, with five drug rows. The exact
layout is documented in [`docs/form-spec.md`](docs/form-spec.md) and
demonstrated in [`docs/print-template.html`](docs/print-template.html).

Print rules:

- Black text on white paper.
- No app logo, brand color, gradient, or shadow.
- TH SarabunPSK or a verified close substitute.
- Title 18 pt bold.
- Body 15 pt.
- Table and patient block 14 pt.
- Notes and legal box 12 pt.
- The signature line stays blank for the prescriber.
- The form code "แบบ ใบสั่งจ่าย ย.ส. ๒/ว.จ. ๒" stays in the top-left corner.
- The bottom legal box is part of the official form and must not be removed.

If a prescription has more than five lines, v1 blocks printing and asks the
operator to split it into separate forms. A continuation policy must be
confirmed by the pharmacist before multi-page support is added.

## 10. Security and privacy

- Connection credentials are encrypted at rest with encryptman. The master key
  lives in the OS keychain through encryptman-keyring.
- The connection password is a `secrecy::SecretString` end to end in Rust,
  zeroized on drop, and `Debug`-redacted.
- The plain settings file holds the hospital profile, catalog, and print
  preferences only. It never holds credentials or patient data.
- The print snapshot is memory-only.
- The frontend never touches the database. It calls Tauri commands only.
- CID is masked in search results.
- Patient data is cleared on logout and on visit change.
- Do not add a local audit log or print history without a separate privacy and
  retention decision.
- If an idle lock is added, it must clear the patient draft and the print
  snapshot.

## 11. Milestones

| Milestone | Scope |
|---|---|
| **D0** | Read the official form, study er-refill, document the design |
| **D1** | Visual system, form spec, schema verification package |
| **D2** | Verify HOSxP schema and remove `SCHEMA-UNVERIFIED` markers |
| **M0** | Workspace scaffold, Tauri 2 plus Leptos 0.8, login gate, connection smoke test |
| **M1** | Patient search, patient card, visit selection |
| **M2** | Narcotic catalog settings, drug search, configured icode list |
| **M3** | Order pull, form builder, validation, confirmation snapshot |
| **M4** | Official A4 print sheet and print gate |
| **M5** | Packaging, pharmacy pilot, feedback |

Current status: M0 to M4 are implemented in the scaffold. M5 and the schema
blockers B1, B4, B5 to B11, and B12 to B14 remain open.

## 12. Testing expectations

Default suite, no database:

- Patient search input classification: HN, CID, name.
- CID masking.
- Thai and Buddhist-era formatting.
- Address line assembly.
- Catalog validation: every icode has a law category and print unit.
- Order line mapping and merging.
- Quantity and sig validation.
- Category flag derivation.
- Five-row pagination and the more-than-five error.
- Confirmation snapshot equality.
- Read-only guard tests for every SQL constant.
- Officer password verification with known MD5 digests.

Live database tests:

- Gated behind the connector's `integration-tests` feature.
- Run only against a test or staging HOSxP instance.
- Never run against production.
- Read `ADDICT_ORDER_TEST_DATABASE_URL` from the environment.
- Do not commit credentials or test patient data.

Frontend:

- Host tests for pure logic and state transitions.
- `wasm-bindgen-test` for the IPC contract with a mocked `invoke`.

## 13. Style and conventions

- Rust style: Rust 2024, rustfmt, and clippy with warnings denied. The sibling
  er-refill `AGENTS-RUST.md` is the reference when it is available.
- Use `thiserror` at crate boundaries. One typed error per boundary.
- Translate errors to Thai only in `src-tauri/commands.rs`.
- Thai UI strings live in the frontend and in user-facing command errors.
- Crate code, comments, and docs are English.
- CSS is plain CSS with tokens from `docs/DESIGN.md`. No framework.
- Prefer small, explicit functions over clever abstractions.
- No `unwrap` or `expect` in production paths without a documented invariant.
- Use `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `cargo doc`, and `cargo deny` before claiming completion.

## 14. What not to do

- Do not add a second way to reach the database.
- Do not add an ORM or a heavy query builder.
- Do not add write-capable Tauri fs or shell scopes.
- Do not add a PDF generation library.
- Do not build a separate hidden print layout.
- Do not re-sort drug rows by quantity. Keep the order from the source or the
  catalog.
- Do not print a drug that is not in the configured catalog.
- Do not infer the legal category from the drug name.
- Do not print a signature or a signature image.
- Do not persist patient data to disk in v1.
- Do not add a local print history without a privacy decision.
- Do not remove the official legal note box from the print.
- Do not use the app brand on the printed form.
