# Addict Order

**A controlled-document desktop app for Thai Category 2 narcotic and Category 2 psychotropic prescriptions.**

> The target document is the official **Y.S. 2 / W.C. 2 prescription form**
> (ใบสั่งจ่าย ย.ส. ๒/ว.จ. ๒) issued by the Thai Narcotics Control Division.
> It is a prescription, not a ward drug requisition. The app uses the official
> Thai term "ใบสั่งจ่าย" in the UI and on paper.

Status: **M0 to M4 scaffold implemented; schema blockers still open**. The
first verification pass against `hos`, โรงพยาบาลสระโบสถ์, MariaDB 10.5.29
corrected the SQL and schema assumptions. The working desktop app now has the
login gate, patient search, visit selection, pharmacist catalog, order pull,
official A4 form preview, confirmation gate, and print. The remaining blockers
are B1, B4, B5 to B11, and B12 to B14. See
[`docs/verify-schema.md`](docs/verify-schema.md) and
[`docs/verify-queries.sql`](docs/verify-queries.sql).

## Overview

Addict Order is a Tauri 2 desktop application for hospital pharmacy staff. It
reads a HOSxP MySQL/MariaDB database in strict read-only mode, finds a patient
by HN, CID, or name, loads a selected visit, pulls the configured narcotic
icodes from that visit, and assembles the official Thai prescription form for
the prescriber to sign.

The app is a document builder, not a prescribing system. It never writes to
HOSxP, never prints a signature, and never invents clinical or legal data.

## Why this exists

The official form requires patient identity, address, prescriber identity,
professional licence number, hospital details, drug generic name, trade name,
strength, directions for use, quantity, and unit. Much of that data already
exists in HOSxP, but it is spread across several tables and varies between
HOSxP installations. Re-typing it creates transcription errors on a controlled
document.

Addict Order reduces transcription while keeping the pharmacist and prescriber
in control:

1. Sign in with the HOSxP `officer` account.
2. Search for the patient by HN, CID, or name.
3. Select the visit that the prescription belongs to.
4. The app reads only the configured narcotic icodes for that visit.
5. The pharmacist reviews the official form preview.
6. The form is confirmed and printed. The prescriber signs it by hand.

## Scope

### In scope for v1

- HOSxP `officer` login with an in-memory session.
- Patient search by HN, CID, or Thai name.
- OPD visit selection for the prescription date.
- A pharmacist-managed catalog of narcotic icodes with print metadata.
- Pulling configured narcotic order lines from the selected visit.
- Official A4 form preview and print, five drug rows per form.
- A confirmation gate that freezes the printed snapshot.
- Local settings for hospital header, catalog, and print options.

### Out of scope for v1

- Writing, updating, or deleting anything in HOSxP.
- Prescribing, dose calculation, drug interaction checks, or clinical decisions.
- Printing a signature or a signature image.
- Stock control, ward requisitions, or inventory.
- IPD admissions. OPD visits first, pending pharmacist confirmation.
- Persistent patient history or a local prescription archive.
- Automatic inference of the legal drug category.
- A second print layout. The screen preview is the print output.

## Implemented in the current scaffold

- Tauri 2 + Leptos 0.8 workspace with four Rust crates.
- HOSxP `officer` login with encrypted connection settings and a memory-only session.
- Patient search by HN, CID, or name, with CID masking.
- Visit selection for the last 30 days.
- Pharmacist-managed narcotic catalog with icode search, legal category, print unit, default sig, and active flag.
- Order pull from the selected visit using the configured icodes.
- Official A4 form preview with five rows, category checkboxes, patient and prescriber blocks, notes, and the legal box.
- Patient edit and prescriber selection dialogs.
- Confirmation snapshot gate and `window.print()`.
- Read-only guard, parameterized SQL, and encrypted connection config.
- Unit tests for models, core, connector, and Tauri settings.

The remaining work is closing the schema and legal blockers before the pilot.

## Architecture

The workspace follows the proven er-refill structure:

```text
addict-order/
├── src-tauri/              # Tauri 2 shell: thin command adapters only
├── crates/
│   ├── models/             # Domain types and Thai/Buddhist-era formatting
│   ├── core/               # Validation, form builder, snapshot, repository trait
│   └── hosxp-connector/    # The only crate allowed to talk to MySQL
├── app/                    # Leptos 0.8 CSR/WASM frontend
│   ├── src/
│   └── style/
├── docs/
└── icon-master.svg
```

Dependency direction:

```text
app -> addict-order-core -> addict-order-models
hosxp-connector implements a repository trait defined in addict-order-core
```

`addict-order-core` is testable with a mock repository. The default test suite
must never require a live database.

### Stack

- Tauri 2 desktop shell.
- Leptos 0.8 client-side rendering compiled to WASM with Trunk.
- Rust 2024 edition.
- sqlx with MySQL/MariaDB, SELECT-only.
- encryptman plus encryptman-keyring for connection credentials.
- Plain CSS, no UI framework.
- HTML/CSS A4 print plus `window.print()`, no PDF library.

## Documentation

| Document | Purpose |
|---|---|
| [`docs/form-spec.md`](docs/form-spec.md) | Full transcription of the official form, measurements, field mapping, print rules, and legal questions |
| [`docs/verify-schema.md`](docs/verify-schema.md) | Every HOSxP table and column to verify, with SQL and pass criteria |
| [`docs/verify-queries.sql`](docs/verify-queries.sql) | Runnable read-only verification queries |
| [`docs/architecture.md`](docs/architecture.md) | System design, domain model, data flow, security, and milestones |
| [`docs/auth.md`](docs/auth.md) | HOSxP `officer` login, session, MD5 caveat, and prescriber identity |
| [`docs/DESIGN.md`](docs/DESIGN.md) | Visual system derived from `icon-master.svg` |
| [`docs/print-template.html`](docs/print-template.html) | Static A4 print reference for layout review |
| [`AGENTS.md`](AGENTS.md) | Rules for coding agents and future contributors |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribution workflow and review checklist |
| [`SECURITY.md`](SECURITY.md) | Security policy and PHI handling |
| [`docs/security-scanning.md`](docs/security-scanning.md) | GitHub, GitGuardian, and local secret-scanning setup |
| [`CHANGELOG.md`](CHANGELOG.md) | Release history |

## Schema verification first

The app will not guess HOSxP column names. The following areas must be verified
against the hospital instance before connector code is written:

- `officer` login columns and the MD5 assumption.
- `patient` identity, CID, name, address parts, and phone columns.
- Address lookup tables and the correct join for subdistrict, district,
  province, and postal code.
- `ovst` visit columns and the visit date source.
- `opitemrece` order columns, filters, and line identity.
- `drugitems` generic name, trade name, strength, units, and any legal category
  source.
- `drugusage` and `sp_use` text columns.
- `doctor` name and professional licence number.
- Hospital profile source or a settings-only fallback.
- Indexes, character encoding, and the MariaDB read-only variable.

**First verification pass:** 2026-10-29 against `hos`, โรงพยาบาลสระโบสถ์,
MariaDB 10.5.29. The pass found 6 critical SQL issues, 8 high-impact schema
mismatches, and 3 operational warnings. They are corrected in
[`docs/verify-schema.md`](docs/verify-schema.md). Key facts: the database uses
`tis620` (the app client uses `utf8mb4` for sqlx decoding); `opitemrece.hos_guid`
is the primary key; `drugitems` has no legal
category data; `doctor.licenseno` is the licence column; and the hospital
profile address needs pharmacist correction.

Open [`docs/verify-schema.md`](docs/verify-schema.md) for the checklist and
[`docs/verify-queries.sql`](docs/verify-queries.sql) for the queries.

Do not paste real patient names, HN, CID, or addresses into the verification
documents. Use aggregate results or a fictional test patient.

## Security and privacy

- HOSxP is read-only at every layer: database grant, session mode, SQL guard,
  and parameterized queries.
- No patient data is written to disk in v1.
- No patient data, credentials, or secrets are logged.
- The login session is memory-only and cleared on logout or restart.
- CID is masked in search results and shown in full only on the form preview
  and the printed form.
- The app never prints a signature. The prescriber signs the paper.
- The app never writes an audit row to HOSxP.
- Secret scanning is layered: GitHub secret scanning and push protection,
  GitGuardian, and an optional local `ggshield` hook. See
  [`docs/security-scanning.md`](docs/security-scanning.md).

## Development

The workspace is scaffolded. Prerequisites: Rust stable, the
`wasm32-unknown-unknown` target, Trunk, and the Tauri CLI.

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cargo install tauri-cli --locked
cargo test
cargo tauri dev
cargo tauri build
```

The default test suite runs without a database. The app connects to HOSxP
through the encrypted settings dialog. The client connection uses `utf8mb4`
while the database remains `tis620`; MySQL converts the TIS-620 column data to
UTF-8 for sqlx. Live database tests belong behind an `integration-tests`
feature and a test/staging instance.

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`AGENTS.md`](AGENTS.md) before
opening a pull request. Documentation changes and schema verification results
are welcome before the first code milestone.

## License

MIT. See [`LICENSE`](LICENSE).
