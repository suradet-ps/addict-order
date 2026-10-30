# Security scanning

Addict Order handles credentials and patient data. A secret that reaches a
public GitHub repository must be treated as compromised even if it is deleted
in a later commit, because it remains in the git history and in forks.

This document covers the secret-scanning layers around the repository. It does
not replace [`SECURITY.md`](../SECURITY.md) or the hard rules in
[`AGENTS.md`](../AGENTS.md).

## Layers

| Layer | Where it runs | Purpose |
|---|---|---|
| `.gitignore` and review | Local, before commit | Keep credentials, patient data, and build output out of git |
| `ggshield` pre-commit hook | Local, optional | Block a secret before it enters a commit |
| GitHub secret scanning | GitHub, after push | Detect supported provider tokens in the repository |
| GitHub push protection | GitHub, before accepting a push | Block supported provider tokens before they land |
| GitGuardian | GitGuardian dashboard / GitHub App | Deep detection, historical scan, pull request checks, incident workflow |

No single layer is sufficient. GitHub push protection only covers supported
provider patterns; it does not replace a review of configuration files, test
fixtures, and documentation.

## Current repository settings

Checked on 2026-10-30 for `suradet-ps/addict-order`:

- The repository is public. Treat every push as public disclosure.
- GitHub secret scanning is enabled.
- GitHub push protection is enabled.
- Dependabot alerts are enabled.
- Dependabot security update pull requests are disabled; Renovate handles
  dependency and security pull requests.
- GitHub private vulnerability reporting is enabled.
- Non-provider patterns and validity checks could not be enabled through the
  REST API on this user-owned public repository. Check **Settings > Advanced
  Security** in the GitHub UI and enable them if the account's plan offers
  them.
- The GitGuardian GitHub App is installed. The `GitGuardian Security Checks`
  check run passed on pull request #1 on 2026-10-30; server-side monitoring and
  pull request checks are active.

The `.gitguardian.yaml` file in the repository root configures `ggshield` only.
It does not change the GitGuardian dashboard, the GitGuardian GitHub App, or
GitHub native secret scanning.

## Renovate

Dependency and security pull requests are handled by Renovate, configured in
[`renovate.json5`](../renovate.json5).

- The update window is Monday before 09:00 Asia/Bangkok. The Dependency
  Dashboard issue tracks pending updates.
- Cargo minor and patch updates are separate; low-risk patch updates are
  grouped. Tauri, `tauri-build`, `encryptman`, and `encryptman-keyring` move
  together.
- GitHub Actions are pinned to commit digests; non-major updates are grouped.
- The Ubuntu runner stays on 24.04. Tauri system packages are validated against
  that image, so Renovate must not move it to 26.04 automatically.
- `md-5` is held below 0.11.0 until `crates/core/src/auth.rs` handles the new
  digest type.
- `sqlx` is held below 0.9.0 until the workspace MSRV moves to Rust 1.94.
- `glib` is held below 0.19.0 because `gtk 0.18` (via Tauri 2) requires
  `glib ^0.18`; the patched 0.20.0 cannot be selected yet.
- Dependabot remains the vulnerability-alert source, but its security update
  pull requests are disabled so Renovate is the only PR author.

The hold rules are deliberate. Remove a rule only after the code or toolchain
change that makes the newer version buildable and testable.

## GitGuardian setup

### Server-side monitoring

1. Sign in at <https://dashboard.gitguardian.com>.
2. Open **Settings > Integrations > Sources** and install the GitGuardian
   GitHub App for the `suradet-ps` account.
3. Select **All repositories** or explicitly select `addict-order`. Selecting
   only some repositories means a new repository is not monitored until the
   installation is updated.
4. Confirm the repository is in the monitored perimeter.
5. Enable pull request check runs. Use the `Failed` conclusion if the `main`
   branch is protected and secret findings must block a merge, or `Neutral`
   while the project is still in its scaffold phase.
6. Run a historical scan after the first push.
7. In the dashboard, add file-path exclusions only for generated or binary
   paths. Do not exclude source, configuration, or documentation paths.

### Local scanning with ggshield

Install once:

```bash
pipx install ggshield
ggshield auth login
```

Scan the working tree and the git history:

```bash
ggshield secret scan path -r .
ggshield secret scan repo .
```

The pre-commit hook is optional:

```bash
pipx install pre-commit
pre-commit install
```

The hook is defined in [`.pre-commit-config.yaml`](../.pre-commit-config.yaml)
and calls the local `ggshield` binary. It does not run unless pre-commit is
installed.

### False positives

Prefer one of these, in order:

1. Change the fixture or example so it is obviously synthetic and is not a
   valid credential shape.
2. Add a `# ggignore` comment on the exact line when the value must stay.
3. Add a narrow `ignored_matches` entry in `.gitguardian.yaml` with a name that
   explains why the value is safe.
4. Ignore the incident in the GitGuardian dashboard after review.

Never add a broad `ignored_paths` entry for `src/`, `docs/`, or `*.rs` to make a
finding disappear. That hides future real secrets in the same area.

## GitHub repository setup

After the first push, confirm in **Settings > Code security and analysis**:

- Secret scanning: enabled.
- Push protection: enabled.
- Non-provider patterns: check and enable in the UI.
- Validity checks: check and enable in the UI.
- Dependabot alerts: enabled.
- Dependabot security updates: disabled; Renovate handles dependency PRs.
- Private vulnerability reporting: enabled.
- Renovate: configured in `renovate.json5`.

Then protect `main` with a ruleset or branch protection rule:

- Require a pull request before merging.
- Require the `CI` status check.
- Require the branch to be up to date before merging.
- Block force pushes and deletion.

The GitGuardian check run can be added as a required status check after the
GitHub App is installed and the first pull request has produced a check run.

## What must never be committed

- HOSxP database hosts, usernames, passwords, or connection strings.
- `.env` files, key files, certificates, settings exports, and connection
  profiles.
- Patient names, HN, CID, addresses, phone numbers, visit data, or drug lines.
- Real hospital screenshots, logs, or database dumps.
- A `settings.json` or `connection.json` copied from a workstation.

If a secret is committed:

1. Revoke or rotate it immediately. Deleting the file is not enough.
2. Remove it from the current tree.
3. Decide whether history rewriting is required. For a public repository,
   assume the value is already exposed.
4. Record the incident and the rotation in the private security channel, not in
   a public issue.
