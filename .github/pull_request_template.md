# Pull Request

## Summary

Describe the change in one or two sentences.

## Scope

- [ ] Documentation
- [ ] Schema verification
- [ ] Backend
- [ ] Frontend
- [ ] Print
- [ ] Build or tooling

## Related issues

Closes #

## Schema impact

List any table or column touched, and the verification ID from
`docs/verify-schema.md`.

- Verification IDs:
- `SCHEMA-UNVERIFIED` markers added or removed:

## Verification

- [ ] `cargo fmt --check`
- [ ] `cargo check --all-targets`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo doc --no-deps`
- [ ] `cargo deny check`
- [ ] Frontend build, if applicable
- [ ] Print preview checked against `docs/print-template.html`, if applicable
- [ ] Printed on a real A4 printer, if applicable

## Safety checklist

- [ ] No HOSxP write statement was added.
- [ ] No PHI or secret is logged.
- [ ] Every SQL value is bound.
- [ ] New SQL constants have a read-only guard test.
- [ ] New schema assumptions are documented in `docs/verify-schema.md`.
- [ ] The official form is not branded.
- [ ] The signature line is still blank.
- [ ] The five-row limit is enforced.
- [ ] The confirmation snapshot is invalidated on edit.
- [ ] No patient data is written to disk.

## Screenshots or print samples

Use fictional data only. Do not attach real patient data.
