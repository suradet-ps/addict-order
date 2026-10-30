# Contributing to Addict Order

Thank you for contributing. Addict Order handles controlled-drug documents and
patient data. Correctness, privacy, and print fidelity matter more than speed.

## Before you start

1. Read [`README.md`](README.md).
2. Read [`AGENTS.md`](AGENTS.md) for the hard rules.
3. Read [`docs/architecture.md`](docs/architecture.md).
4. Read [`docs/DESIGN.md`](docs/DESIGN.md) for any UI work.
5. Read [`docs/form-spec.md`](docs/form-spec.md) for any print work.
6. Read [`docs/verify-schema.md`](docs/verify-schema.md) before writing a
   database query.

## Contribution types

### Documentation

Documentation improvements are welcome before the first code milestone. Keep
the documents in English. Use the official Thai text only when quoting the
form.

### Schema verification

Schema verification results are valuable. Use the format in
[`docs/verify-schema.md`](docs/verify-schema.md) and do not include real patient
identifiers. Aggregate counts and masked samples only.

### Code

Wait for the schema gate to pass for the area you want to change. A pull request
that touches an unverified table or column must carry a `SCHEMA-UNVERIFIED`
comment and must link to the verification item.

## Development workflow

1. Fork the repository.
2. Create a topic branch from `main`.
3. Make a small, focused change.
4. Run the checks.
5. Open a pull request with a clear description.

Before the first push to a public repository, read
[`docs/security-scanning.md`](docs/security-scanning.md) and run a local secret
scan.

Planned checks after the scaffold exists:

```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
cargo deny check
```

Frontend:

```bash
cd app
trunk build
```

Live database tests:

```bash
cd crates/hosxp-connector
cargo test --features integration-tests
```

Live tests must use a test or staging instance and must never write.

## Commit messages

Use Conventional Commits.

```text
feat: add patient search
fix: correct CID mask
docs: add schema verification checklist
refactor: split form validation
test: cover five-row limit
chore: update dependencies
```

Keep the subject in the imperative mood and under 72 characters. Use the body
to explain why, not what.

## Pull request checklist

- [ ] The change is scoped to one concern.
- [ ] Documentation is updated in the same pull request.
- [ ] No HOSxP write statement was added.
- [ ] No PHI or secret is logged.
- [ ] No credential, connection string, or patient data is committed.
- [ ] Every SQL value is bound.
- [ ] New SQL constants have a read-only guard test.
- [ ] New schema assumptions are added to `docs/verify-schema.md`.
- [ ] The print preview and the print output still share one DOM and one
      stylesheet.
- [ ] The official form is not branded.
- [ ] The signature line is still blank.
- [ ] The five-row limit is enforced.
- [ ] The confirmation snapshot is invalidated on edit.
- [ ] `cargo fmt`, `clippy`, `test`, and `doc` pass.
- [ ] No new dependency is added without a reason.

## Code style

- Rust 2024 edition.
- Follow Rust 2024, rustfmt, and clippy with warnings denied. Use the workspace
  `AGENTS-RUST.md` when it is available.
- One typed error per crate boundary.
- Thai user-facing text only at the UI edge and in `src-tauri/commands.rs`.
- Crate code, comments, and docs are English.
- Plain CSS with tokens from `docs/DESIGN.md`.
- No ORM, no heavy query builder, no PDF library.
- No `unwrap` or `expect` in production paths without a documented invariant.

## Security and privacy

Do not open a public issue with patient data, credentials, or a vulnerability
that exposes either. Follow [`SECURITY.md`](SECURITY.md).

## License

By contributing, you agree that your contribution is licensed under the MIT
License in [`LICENSE`](LICENSE).
