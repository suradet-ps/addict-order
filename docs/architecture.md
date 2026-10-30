# Architecture - Addict Order

This document describes the system design for Addict Order. It is a design
baseline. Schema-dependent details remain provisional until the checklist in
[`verify-schema.md`](verify-schema.md) is signed off.

## 1. Problem

The hospital pharmacy must issue the official Thai prescription form for
Category 2 narcotics and Category 2 psychotropic substances. The form is a
controlled document. It requires data from several sources:

- Patient identity and address from HOSxP.
- The prescriber and professional licence number from HOSxP or a verified
  manual entry.
- The hospital name, address, phone, and fax from the hospital profile.
- The drug generic name, trade name, strength, directions, quantity, and unit
  from the order and the pharmacy catalog.
- The legal category of each drug, which is a pharmacist-verified catalog
  attribute.

The current process requires re-typing data from HOSxP onto paper. That is slow
and error-prone. Addict Order reduces the transcription while keeping the
pharmacist and prescriber in control.

## 2. Product boundary

Addict Order is a read-only document builder.

It does:

- Read HOSxP patient, visit, order, drug, and doctor data.
- Use a pharmacist-managed catalog to identify narcotic icodes and print
  metadata.
- Build and print the official form.
- Require a confirmation snapshot before printing.

It does not:

- Prescribe, calculate doses, or make clinical decisions.
- Write to HOSxP.
- Print signatures.
- Manage stock or ward requisitions.
- Store patient history in v1.
- Infer legal categories.

## 3. Users and roles

### Operator

A pharmacy staff member who signs in with a HOSxP `officer` account. The
operator searches for the patient, selects the visit, reviews the form, and
prints it.

### Pharmacist reviewer

The pharmacist who verifies the patient, prescriber, licence, legal category,
and drug details. In v1 this is the same signed-in operator. The app does not
enforce a separate role yet.

### Prescriber

The licensed physician, dentist, or veterinarian who signs the printed form.
The app never signs for the prescriber. The prescriber identity and licence
number are printed, but the signature line remains blank.

### DBA and schema verifier

The person who runs the read-only verification queries and signs off the
schema. This role is external to the app.

## 4. Data flow

```text
HOSxP MySQL/MariaDB
        |
        | SELECT only
        v
hosxp-connector
        |
        | Repository trait
        v
addict-order-core
        |
        | Validated draft and snapshot
        v
Tauri commands
        |
        | IPC
        v
Leptos frontend
        |
        | Same DOM and CSS
        v
A4 print preview and window.print()
```

### Step by step

1. **Startup**
   - Load the plain settings file.
   - Load the encrypted connection configuration.
   - Probe the connection with `SELECT 1`.
   - Show the login gate.

2. **Login**
   - Read one `officer` row by login name.
   - Check active flag, auth type, and MD5 password.
   - Store a sanitized `SessionUser` in the Tauri process.
   - Clear the session on logout, restart, or connection change.

3. **Patient search**
   - The frontend sends a search term.
   - The backend classifies the term:
     - 13 digits: CID.
     - Digits within the HN length policy: HN.
     - Otherwise: name.
   - The connector runs the appropriate SELECT.
   - Results are limited to 20 and CID is masked in the list.

4. **Patient selection**
   - Fetch the full patient identity and address.
   - Resolve address names using the verified join or fallback.
   - Clear any previous visit, lines, and print snapshot.

5. **Visit selection**
   - Fetch OPD visits for the patient in the selected date range.
   - Default range: today. Lookback option: 30 days.
   - The selected visit date becomes the prescription date.
   - Clear any previous lines and print snapshot.

6. **Order pull**
   - Read configured narcotic icodes from settings.
   - Query `opitemrece` for the selected `vn` and those icodes.
   - Join `drugitems` and the sig lookup tables.
   - Return raw order lines to the core.
   - The core maps each line to a `PrescriptionLine` using the catalog.
   - Identical `(icode, sig, doctor)` lines are merged and quantities summed.
   - Different sigs remain separate.
   - Lines with missing required fields are flagged.

7. **Review**
   - The operator reviews the official form preview.
   - The preview shows exactly what will print.
   - Required fields are highlighted.
   - The operator can edit the prescription date only by selecting a different
     visit. There is no free date picker.
   - The operator cannot add a drug outside the configured catalog.
   - Manual additions are out of scope for v1. If added later, they require a
     reason and a separate audit decision.

8. **Confirmation**
   - The operator opens the confirmation dialog.
   - The dialog summarizes patient, visit, prescriber, licence, hospital, date,
     row count, and category flags.
   - Confirming creates an immutable snapshot.
   - Any edit clears the snapshot and disables printing.

9. **Print**
   - The print button calls `window.print()`.
   - The print stylesheet hides all app chrome.
   - The A4 form is the only visible content.
   - The signature line is blank.
   - The app does not save a PDF automatically. The OS print dialog may save
     one, and that is outside the app's control.

## 5. Workspace and crates

### `crates/models`

Pure domain types and formatting. No I/O, no sqlx, no Tauri.

Contents:

- Patient, address, visit, prescriber, hospital, catalog, line, draft,
  snapshot, and print page types.
- `LawCategory` enum.
- `SourceKind` enum.
- Thai month names and Buddhist-era conversion.
- Thai digit conversion for row numbers.
- Quantity formatting.
- Address line assembly.
- Validation error types that do not depend on the database.

### `crates/core`

Business logic and the repository trait.

Contents:

- `HosxRepository` trait.
- `RepositoryError`.
- Patient search term classification.
- Catalog validation.
- Order line to form line mapping.
- Line merging rules.
- Required field validation.
- Category flag derivation.
- Five-row pagination.
- Confirmation snapshot equality.
- Mock repository for tests.

### `crates/hosxp-connector`

The only crate allowed to use sqlx or MySQL.

Contents:

- Encrypted connection configuration.
- Connection pool with read-only session settings.
- SQL constants and query builders.
- Read-only guard.
- Repository implementation.
- Integration tests behind a feature flag.

### `src-tauri`

Thin Tauri shell.

Contents:

- Application state.
- Command adapters.
- Thai error mapping.
- Settings persistence.
- Health monitor.
- Dialog plugin wiring.

No SQL and no business logic belong here.

### `app`

Leptos 0.8 CSR frontend.

Contents:

- Login gate.
- Top bar.
- Command bar.
- Patient search and patient card.
- Visit selector.
- Form preview.
- Confirmation dialog.
- Settings modal.
- Print stylesheet.
- IPC API layer with a mock for tests.

### Connection charset

The Sra Bos database and server connection use `tis620` / `tis620_thai_ci`.
The CLI verification uses `--default-character-set=tis620`. The connector sets
`charset=utf8mb4` on its client connection so MySQL converts TIS-620 column
data to UTF-8 before sqlx decodes it. A `tis620` client charset makes sqlx fail
with `invalid utf-8 sequence`.

## 6. Repository contract

The exact methods will follow the verified schema. The intended contract is:

```rust
#[async_trait]
pub trait HosxRepository: Send + Sync {
    async fn ping(&self) -> Result<(), RepositoryError>;
    async fn fetch_officer(&self, login_name: &str) -> Result<Option<OfficerAccount>, RepositoryError>;
    async fn search_patients(&self, query: &PatientQuery) -> Result<Vec<PatientSummary>, RepositoryError>;
    async fn fetch_patient(&self, hn: &str) -> Result<Option<PatientDetail>, RepositoryError>;
    async fn fetch_visits(&self, hn: &str, range: DateRange) -> Result<Vec<VisitSummary>, RepositoryError>;
    async fn fetch_narcotic_orders(&self, vn: &str, icodes: &[String]) -> Result<Vec<OrderLine>, RepositoryError>;
    async fn search_drugs(&self, term: &str) -> Result<Vec<DrugItem>, RepositoryError>;
    async fn fetch_doctor(&self, code: &str) -> Result<Option<DoctorProfile>, RepositoryError>;
    async fn search_doctors(&self, term: &str) -> Result<Vec<DoctorProfile>, RepositoryError>;
    async fn fetch_hospital_profile(&self) -> Result<Option<HospitalProfile>, RepositoryError>;
}
```

`fetch_hospital_profile` is the primary source. It has 1 row on Sra Bos. The
settings profile is an override for the printed header when the database address
is incomplete or needs correction.

### Patient query type

```rust
pub enum PatientQuery {
    Hn(String),
    Cid(String),
    NamePrefix(String),
    NameContains(String),
}
```

The frontend sends a raw term. The core classifies it. The connector executes
the matching query.

### Order line type

```rust
pub struct OrderLine {
    pub vn: String,
    pub hn: String,
    pub icode: String,
    pub item_no: Option<i64>,
    pub hos_guid: String,
    pub quantity: i64,
    pub drugusage_code: Option<String>,
    pub sp_use_code: Option<String>,
    pub sig_text: Option<String>,
    pub doctor_code: Option<String>,
    pub doctor_name: Option<String>,
    pub doctor_license_no: Option<String>,
    pub generic_name: Option<String>,
    pub trade_name: Option<String>,
    pub strength: Option<String>,
    pub source_units: Option<String>,
    pub order_date: Option<NaiveDate>,
    pub order_time: Option<NaiveTime>,
}
```

`hos_guid` is the PRIMARY KEY on Sra Bos. `quantity` is int(11); v1 does not
support decimal quantities.

## 7. Settings model

### Plain settings file

Suggested file name: `addict-order.settings.json`.

Contents:

- `version`
- `hospital`: hospital profile override. `hospital_profile` is the primary
  source, but its address needs pharmacist correction on Sra Bos.
- `catalog`: configured narcotic drugs.
- `print`: print options.
- `ui`: optional last selected lookback range.

The plain settings file must never contain:

- database host, user, password, or port
- patient data
- visit data
- prescription lines
- doctor licence overrides entered for a session

### Encrypted connection file

Suggested file name: `addict-order.connection.json`.

Contents, encrypted with encryptman:

- host
- port
- database
- user
- password

The master key lives in the OS keychain through encryptman-keyring. The service
name should be `addict-order`.

### Catalog entry

```json
{
  "icode": "1001234",
  "genericName": "Morphine sulfate",
  "tradeName": "MST Continus",
  "strengthText": "30 mg",
  "printUnit": "Tab",
  "lawCategory": "NARCOTIC_2",
  "defaultSig": "รับประทานครั้งละ 1 เม็ด วันละ 2 ครั้ง",
  "sortOrder": 10,
  "active": true
}
```

The catalog is the source of truth for:

- which icodes are in scope
- the printed generic name
- the printed trade name
- the printed strength
- the printed unit
- the legal category
- the default sig when the order has none

The catalog must be reviewed and signed off by a pharmacist. The app must not
derive these values automatically from an unverified source.

## 8. Address strategy

HOSxP address storage varies. The app must support a verified primary strategy
and a manual fallback.

### Primary strategy

Use the verified patient address columns and the `thaiaddress` join:

```sql
LEFT JOIN thaiaddress t
  ON t.addressid = CONCAT(
       COALESCE(p.chwpart, ''),
       COALESCE(p.amppart, ''),
       COALESCE(p.tmbpart, '')
     )
```

Verified facts:

- `thaiaddress.addressid` is varchar(6) = `chwpart + amppart + tmbpart`.
- The join covers 49,550 / 49,557 patients (99.99%).
- `name` is the subdistrict name.
- `full_name` needs a format check before it is split into district and
  province.
- `pocode` is the postal code and is missing on 7.7% of subdistricts.
- `patient_address` exists but has 0 rows. Do not use it.
- `c_province`, `c_district`, and `c_subdistrict` are not needed.

### Fallback strategy

If the address cannot be resolved:

- Show the raw values that are available.
- Leave unresolved fields blank on the form.
- Allow the operator to edit the address fields in the preview.
- Warn in the confirmation dialog that the address is incomplete.
- Do not block printing solely because the address is incomplete. The paper
  form can be completed by hand.

The app must never print a wrong address silently. If the join is ambiguous,
the verification entry fails and the fallback is used.

## 9. Prescriber strategy

### Source order

1. `opitemrece.doctor` if present.
2. `ovst.doctor` if present.
3. The signed-in officer's `officer_doctor_code` if the officer is a doctor.
4. Manual selection from the verified `doctor` table.

### Licence number

1. `doctor.licenseno` varchar(50), the verified licence column.
2. Session-only manual entry if the column is empty. 16 / 293 doctors have no
   licence on Sra Bos.
3. Block printing if neither exists.

The manual entry is not persisted. It is cleared on logout and restart.

### Signature

The signature line is always blank. The app does not use `doctor.signature`,
`doctor.signature_image`, or any signature font.

## 10. Legal category strategy

The official form has two checkboxes:

- ยาเสพติดให้โทษในประเภท ๒
- วัตถุออกฤทธิ์ในประเภท ๒

Each catalog entry has one `law_category`. The form derives the checkboxes from
the lines:

- All lines `NARCOTIC_2`: check the first box only.
- All lines `PSYCHOTROPIC_2`: check the second box only.
- Mixed: both boxes checked, pending pharmacist confirmation that a mixed form
  is legally acceptable. If not, the app must split the prescription into two
  forms by category.

The app must not infer the category from the drug name, the HOSxP therapeutic
group, or an unverified column. A pharmacist verifies every icode in the
catalog.

**Verified on Sra Bos:** `drugitems.drug_control_type_id` exists but is empty
on all 659 drugitems. `opitemrece_narcotic` has 0 rows. The catalog manual
`law_category` is the only source and is a pilot blocker (B4).

## 11. Print model

The print model has one source of truth: the confirmed snapshot.

### Page rules

- A4 portrait.
- Five drug rows per form in v1.
- One form per prescription.
- More than five lines: block printing and ask the operator to split the
  prescription.
- A continuation policy is pending pharmacist and Narcotics Control Division
  confirmation.

### Fidelity rules

- The form code, title, date line, prescriber block, category checkboxes,
  table, patient block, signature line, notes, and legal box are all part of
  the official form.
- No app logo, brand color, gradient, or shadow.
- Black text on white paper.
- The signature line stays blank.
- The printed form must be reviewed against the official PDF and a real
  printer before the pilot.

### Screen and print

- One DOM.
- One stylesheet.
- The screen preview is the print output.
- Print CSS hides the top bar, command bar, dialogs, and banners.
- `@page` controls A4 size and margins.
- The app does not generate a PDF file.

## 12. Security model

### Read-only layers

1. Dedicated MySQL user with `GRANT SELECT` only.
2. `SET SESSION TRANSACTION READ ONLY` on every pooled connection.
3. Read-only guard rejecting non-SELECT statements.
4. Compile-time SQL constants and bound parameters.
5. Minimal Tauri capabilities.

### Secrets

- Connection credentials encrypted at rest.
- Master key in the OS keychain.
- Password as `SecretString` in Rust.
- No secrets in logs, errors, settings export, or the frontend.

### PHI

- No PHI in logs.
- No PHI in settings.
- No PHI in crash reports.
- CID masked in search results.
- Full CID only on the form preview and the printed form.
- Draft and snapshot cleared on logout, visit change, and patient change.
- Optional idle lock clears the draft and snapshot.

### Print and PHI

The OS print dialog can save a PDF. The app cannot prevent that. The operator
must follow hospital policy for printed and electronic copies. This is a
documented residual risk.

## 13. Error handling

One typed error per boundary:

- `models::FormError`
- `core::RepositoryError`
- `core::ValidationError`
- `hosxp-connector::Error`
- `src-tauri::CommandError` with a machine-readable kind and a Thai message

Thai user-facing messages are created only in `src-tauri/commands.rs`.
Crate errors are English and developer-facing.

Error kinds:

- `NotConfigured`
- `Connection`
- `Guard`
- `Query`
- `Validation`
- `Auth`
- `Storage`
- `NotFound`

The frontend switches on the kind, never on message text.

## 14. Performance

- Single hospital, desktop app.
- Pool size around 5.
- One patient search query at a time, debounced by 250 ms.
- One visit query per patient selection.
- One order query per visit selection.
- No N+1 queries per drug.
- Patient search under 300 ms with indexes.
- Order pull under 500 ms with indexes.
- No caching of patient data.

## 15. Testing strategy

### Default suite

No database. Uses a mock repository.

- Input classification.
- CID masking.
- Thai date and number formatting.
- Address assembly.
- Catalog validation.
- Order line mapping and merging.
- Required field validation.
- Category flag derivation.
- Five-row pagination.
- Snapshot equality.
- Read-only guard.
- MD5 verification.

### Integration suite

Gated behind `--features integration-tests`.

- Uses a test or staging HOSxP instance.
- Reads the connection URL from `ADDICT_ORDER_TEST_DATABASE_URL`.
- Never runs against production.
- Never writes.

### Frontend suite

- Host tests for pure logic and state.
- WASM IPC contract tests with a mocked `invoke`.

## 16. Milestones

| Milestone | Deliverable |
|---|---|
| D0 | Form transcription, er-refill study |
| D1 | Architecture, design, schema verification package |
| D2 | Verified schema and signed checklist |
| M0 | Workspace scaffold, login gate, connection smoke test |
| M1 | Patient search, patient card, visit selection |
| M2 | Narcotic catalog settings and drug search |
| M3 | Order pull, form builder, validation, snapshot |
| M4 | Official print sheet and print gate |
| M5 | Packaging, pilot, feedback |

## 17. Open decisions

These must be resolved before or during implementation:

1. Is the current official form the version provided in the repository?
2. Can one form contain both Category 2 narcotics and Category 2 psychotropics?
3. Is a five-row limit acceptable, or is a continuation form required?
4. How many copies must be printed: original, prescribing hospital copy, or
   both?
5. Is a prescription number, book number, or hospital stamp required?
6. Is the patient address mandatory in print, or can it be handwritten?
7. Is the patient phone mandatory?
8. Prescriber licence column: resolved to `doctor.licenseno`.
9. Address lookup strategy: resolved to the `thaiaddress` subdistrict join
   `CONCAT(chwpart, amppart, tmbpart)`.
10. Is the `hospital_profile` address correct? It conflicts with the hospital
    name on Sra Bos and must be corrected by the pharmacist before printing.
11. Is IPD in scope for v1?
12. Is a local print audit log required for controlled substances?
13. Which font is acceptable for the official print: TH SarabunPSK, Sarabun, or
    another?
14. Who is allowed to print: all `STAFF`, pharmacists only, or a new role?
15. What is the retention policy for printed copies and PDFs saved by the OS?
