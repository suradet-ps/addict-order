# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project follows Semantic
Versioning once a release exists.

## [Unreleased]

### Added

- Project README, architecture, visual design, and contribution documents.
- Official form specification for the Y.S. 2 / W.C. 2 prescription form.
- HOSxP `officer` authentication design.
- Schema verification checklist and read-only SQL verification queries.
- Static A4 print reference with fictional sample data.
- Brand master SVG.
- Security scanning guide, `.gitguardian.yaml`, and an optional `ggshield`
  pre-commit hook.
- Dependabot configuration for both Rust workspaces and GitHub Actions.

### Changed

- Corrected the schema verification package after a live MariaDB 10.5.29 check
  against `hos`, โรงพยาบาลสระโบสถ์: `tis620` charset, `@@tx_read_only`,
  `thaiaddress` subdistrict join, `drug_control_type_id` blocker, actual
  patient phone columns, `hos_guid` primary key, `doctor.licenseno`, and
  `hospital_profile` address warning.
- CI now checks the Tauri shell and documentation, uses read-only token
  permissions, and cancels superseded runs.
- The Tauri global API is disabled (`withGlobalTauri: false`); the frontend
  calls `__TAURI_INTERNALS__` directly.

### Fixed

- None yet.

### Security

- Documented the read-only HOSxP boundary, PHI rules, and the no-signature
  print rule.
- Enabled GitHub secret scanning, push protection, Dependabot alerts, and
  private vulnerability reporting for the public repository.

## [0.1.0] - 2026-10-29

First working scaffold.

### Added

- Tauri 2 + Leptos 0.8 workspace with `models`, `core`, `hosxp-connector`, and
  `src-tauri` crates.
- HOSxP `officer` login, encrypted connection settings, read-only pool, and
  in-memory session.
- Patient search by HN, CID, or name; visit selection; configured narcotic
  order pull.
- Pharmacist catalog settings with icode search, legal category, print unit,
  default sig, and active flag.
- Official A4 form preview, patient and prescriber edit dialogs, confirmation
  snapshot, and print.
- Unit tests for models, core, connector, and Tauri settings.

### Changed

- The connector uses a `utf8mb4` client charset so MySQL converts TIS-620
  column data to UTF-8 for sqlx. The CLI verification still uses `tis620`.
- The print stylesheet was tightened to keep a two-line sig on one A4 page.

### Security

- Read-only guard, parameterized SQL, encrypted credentials, no PHI logging,
  and no patient data on disk.
