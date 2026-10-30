# Addict Order

[![CI](https://github.com/suradet-ps/addict-order/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/suradet-ps/addict-order/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust: stable](https://img.shields.io/badge/rust-stable-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Leptos 0.8](https://img.shields.io/badge/Leptos-0.8-blue.svg)](https://leptos.dev)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/suradet-ps/addict-order/issues)

---

## ◆ PULSE

A controlled prescription is already a legal document. The pharmacist should
not be re-typing the same patient identity, address, prescriber licence, and
drug details into it. Addict Order removes the transcription without removing
the pharmacist from the loop.

Operators sign in with their HOSxP **`officer`** account after the database
connection is verified. Addict Order then reads the hospital's
[HOSxP](https://hosxp.org/) record, strictly read-only, finds a patient by HN,
CID, or name, loads the selected OPD visit, and pulls only the configured
narcotic icodes from that visit. It assembles the official **Y.S. 2 / W.C. 2
prescription form** (ใบสั่งจ่าย ย.ส. ๒/ว.จ. ๒) for the prescriber to sign.
HOSxP is never written to, no patient data is cached, and the printed paper is
the legal record.

It is a prescription, not a ward requisition. The app uses the official Thai
term "ใบสั่งจ่าย" in the UI and on paper. It is a document builder, not a
prescribing system: no dose calculation, no drug interaction checks, no stock
control, and no HOSxP write. OPD first; IPD admissions are out of scope for v1.

| D0 design ▣ | D1 spec ▣ | D2 schema ◐ | M0-M4 app ▣ | M5 pilot ☐ |
|---|---|---|---|---|

*The document pipeline - login, patient, visit, catalog, order pull, form,
confirmation, print - is implemented. The schema and legal blockers (B1,
B4-B11, B12-B14) and the pharmacy pilot wait ahead.*

> Built with Tauri 2 + Leptos 0.8, decided by `addict-order-core`, read from
> HOSxP by `addict-order-hosxp-connector` - never a write, never a guessed
> legal category.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One target, two installs, one test, one launch.

```
⟫ rustup target add wasm32-unknown-unknown
⟫ cargo install trunk --locked
⟫ cargo install tauri-cli --locked
⟫ cargo test            # DB-free: core runs against a mock repository
⟫ cargo tauri dev       # desktop app; trunk serves itself
```

The release artifact: `⟫ cargo tauri build` - output lands in
`target/release/bundle`.

<details>
<summary>Notes for clinic PCs</summary>

- Windows needs the WebView2 Runtime - preinstalled on updated Windows 10/11;
  offline machines should take Microsoft's Evergreen offline installer first.
- Installers are **unsigned for v0.x**: expect the SmartScreen prompt
  (*More info -> Run anyway*) and coordinate AV whitelisting with hospital IT.
- The HOSxP connection is configured once per machine in the settings dialog;
  every field is encrypted with `encryptman` and the master key rests in the OS
  keychain. Each PC enters its own connection; credentials never travel.
- The catalog is configured once by the pharmacist: icode search, generic and
  trade name, strength, print unit, default sig, and the legal category
  (`NARCOTIC_2` or `PSYCHOTROPIC_2`). The category is never inferred from the
  drug name.
- The official form has **five drug rows**. A prescription with more than five
  lines is blocked and must be split into separate forms.

</details>

---

## ◆ ANATOMY

Four crates, one hard law: HOSxP is read-only, without exceptions.

- **Decides** - `addict-order-core` is pure Rust: validation, the form builder,
  the confirmation snapshot, and the repository trait. It decides whether a
  draft may be confirmed: patient name, CID, prescriber and licence, hospital
  name, prescription date, at least one line, at most five lines, and a complete
  drug line (generic name, strength, sig, quantity greater than zero, print
  unit, legal category). Tested against a mock repository - no live database in
  the default suite, ever.
- **Reads** - `addict-order-hosxp-connector` is the only crate allowed to talk
  to MySQL/MariaDB. It signs in against `officer` (MD5 password, never the
  plaintext column), searches `patient` by HN, CID, or name, lists `ovst`
  visits, and reads `opitemrece` lines for the selected `vn` filtered to the
  configured icodes, `qty > 0`, `sub_type = '1'`, `item_type <> 'A'`, and OPD
  lines only, joined to `drugitems`. Line identity is `hos_guid`. Every SQL
  constant carries a guard test asserting it is a single read statement - no
  writes of any kind.
- **Prints** - `app` is the Leptos frontend, document-first: the command bar
  holds the search and pull controls, and the stage shows the official A4 sheet
  itself - the same DOM and stylesheet that print. The sheet has five drug
  rows, patient and prescriber blocks, category checkboxes, notes, and the
  legal box. Editing the patient, prescriber, or any line invalidates the
  confirmation. Printing is gated: the operator confirms a snapshot of the
  patient, visit, prescriber, licence, hospital, date, lines, and category
  flags; only then does **พิมพ์ใบสั่งจ่าย** call `window.print()` on exactly
  that snapshot.
- **Seals** - read-only is enforced in layers: a dedicated user with
  `GRANT SELECT` only, `SET SESSION TRANSACTION READ ONLY` on every pooled
  connection, an application-level guard rejecting anything but a single read
  statement, parameterized queries everywhere. Credentials rest encrypted with
  `encryptman`; the key lives in the OS keychain through
  `encryptman-keyring`; the password is a `secrecy::SecretString` end to end,
  zeroized on drop, `Debug`-redacted. The plain settings file holds only the
  hospital profile, catalog, and print preferences - never credentials or
  patient data.
- **Signs in** - the login gate verifies the encrypted HOSxP connection, reads
  one `officer` row, checks active/auth type, and compares `MD5(password)` with
  `officer_login_password_md5`. The plaintext column is never selected;
  `opduser` is never queried; the session is memory-only and cleared on logout,
  restart, or connection change (see `docs/auth.md`).
- **Labels** - Thai/Buddhist-era formatting lives in `addict-order-models`,
  one implementation shared by the screen and the paper. CID is masked in
  search results and shown in full only on the selected patient card and the
  form preview.

---

## ◆ RITUALS

**The core ceremony** - one controlled prescription:

1. Open the app: verify the HOSxP connection, sign in with the `officer`
   account. The session is memory-only.
2. Search for the patient by HN, CID, or Thai name. The result list masks the
   CID; the selected patient card and the form preview show it in full.
3. Select the OPD visit that the prescription belongs to. The prescription
   date is the selected visit date; free backdating is not allowed.
4. The backend reads only the configured narcotic icodes for that visit. A
   drug that is not in the pharmacist-managed catalog is never printed.
5. Review the official form: patient identity and address, prescriber and
   licence, hospital header, prescription date, legal category checkboxes, and
   up to five drug rows. Any edit after confirmation clears the snapshot.
6. Confirm the snapshot, then print. The app never prints a signature; the
   prescriber signs the paper by hand.

**The ceremony of the legal category** - the pharmacist sets `NARCOTIC_2` or
`PSYCHOTROPIC_2` on every catalog item. The app never guesses the category from
a drug name, a therapeutic group, or an unverified HOSxP column.

**The ceremony of restraint** - the app reads a record and prints a paper. It
stores no patient data and keeps no local prescription history. The signed
paper is the record of the visit.

---

## ◆ ECHOES

**Where this artifact is heading**

```
design  ▸ official form, schema verification package ─────────── ▸ sealed
login   ▸ officer auth, encrypted connection, memory session ─── ▸ sealed
search  ▸ HN / CID / name, CID masking, visit selection ──────── ▸ sealed
catalog ▸ pharmacist-managed icodes + legal category ─────────── ▸ sealed
pull    ▸ configured icodes from one visit, read-only ────────── ▸ sealed
form    ▸ A4 Y.S. 2 / W.C. 2, five rows, confirmation gate ───── ▸ sealed
print   ▸ window.print() on the confirmed snapshot ───────────── ▸ sealed
schema  ▸ B1, B4-B11, B12-B14 sign-off ──────────────────────── ▸ ahead
pilot   ▸ pharmacy review, real prescriptions, feedback ──────── ▸ ahead
```

**The schema gate** - the first verification pass on 2026-10-29 against `hos`,
โรงพยาบาลสระโบสถ์, MariaDB 10.5.29 corrected the SQL and schema assumptions:
`tis620` charset, `opitemrece.hos_guid` as the primary key,
`doctor.licenseno` as the licence column, and the missing legal-category data
in `drugitems`. The remaining blockers are tracked in
`docs/verify-schema.md`; no column is guessed.

**Raising the artifact** - read `AGENTS.md` first, especially the hard rules:
HOSxP read-only without exceptions, no PHI or secrets in logs, parameterized
queries only, `SCHEMA-UNVERIFIED` markers until the DBA confirms, and the
pharmacist-verified legal category. `CONTRIBUTING.md` holds the contribution
checklist. The form transcription is `docs/form-spec.md`; the schema checklist
is `docs/verify-schema.md`; the login design is `docs/auth.md`; the visual
system is `docs/DESIGN.md`. `SECURITY.md` holds the vulnerability and PHI
policy; `docs/security-scanning.md` covers GitHub, GitGuardian, and local
secret scanning.

**Status** - every change runs fmt, `clippy -D warnings`, the DB-free test
suite, `cargo doc`, and a `trunk build` of the wasm frontend. CI also compiles
and tests the Tauri shell on Linux.

> Addict Order prints what HOSxP recorded. The pharmacist verifies, the
> prescriber signs, and the paper is the contract.

---

```
  ─────────────────────────────────────────
   A controlled drug on a controlled form
   is not a typing exercise. It is a
   prescription the patient will carry.
  ─────────────────────────────────────────
```

Addict Order is distributed under the [MIT License](LICENSE).
Copyright (c) 2026 Suradet Pratomsak.
