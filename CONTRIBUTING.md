# Contributing to TypoMorph

## Development setup

Requirements: Rust stable (2021 edition), Linux with `evdev`/`/dev/uinput` for the live daemon path (not required for `core-engine`, `licensing`, `prompt-cloud`, or `native-host`, which have no Linux-specific dependencies), and `libdbus-1-dev` + `pkg-config` to build the `daemon` crate (needed by its `ksni` tray dependency).

```bash
sudo apt-get install -y libdbus-1-dev pkg-config
cargo build --workspace
cargo test --workspace
```

## Before opening a PR

Run the same checks CI runs (`.github/workflows/ci.yml`):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

All three must pass with zero warnings. `cargo-audit` also runs in CI against `Cargo.lock` on every PR — if it flags a new advisory in a dependency you didn't touch, mention it in the PR rather than silently ignoring it.

## Code style

- No comments explaining *what* code does — only *why*, when the reason isn't obvious from the code itself (a workaround, a non-obvious invariant, a hidden constraint).
- Prefer small, focused functions and pure logic in `core-engine` over anything Linux- or I/O-specific; if a piece of logic needs no `evdev`/`uinput`/D-Bus/network access, it belongs in `core-engine` (or `prompt-cloud` if it's cloud-client logic), not `daemon` or `native-host`, so both the daemon and the browser-extension host can share it.
- Match the existing test style: `#[cfg(test)] mod tests` at the bottom of the file, descriptive test names as full sentences (e.g. `hindi_text_is_detected_but_not_auto_corrected_regardless_of_current_layout`), and mock external transports (see `licensing::LicenseTransport` / `prompt_cloud::CloudTransport`) instead of making real network calls in tests.
- Don't add a language-, tier-, or feature-gate without also adding a test that exercises the boundary (see `crates/core-engine/src/layout.rs`'s tests for the pattern).

## Commit messages

Short, imperative, and focused on *why* rather than *what* (the diff already shows what changed). No fixed prefix convention is enforced, but keep one logical change per commit.

## Release checklist

Tagging `vX.Y.Z` triggers `.github/workflows/release.yml`, which builds and publishes the `.deb` — but two things are **not** automated and must be done manually, in this order, before pushing the tag:

1. **Bump `version` in `crates/daemon/Cargo.toml` to match the tag** (e.g. tag `v0.2.0` → version `0.2.0`). `cargo-deb` names the package after this version, not the git tag — if they drift, the release workflow publishes a `.deb` whose filename `landing/install.sh` never looked for, and the one-line installer 404s for everyone. Run `cargo check -p daemon` afterward so `Cargo.lock` picks up the bump.
2. **Update the download link in `landing/index.html`** (`href="downloads/typomorph_<version>-1_amd64.deb"` in the Linux download panel) to match the new version. Unlike `landing/install.sh` (which resolves the real asset name from the GitHub Releases API at install time and can't drift), this is a static link and must be updated by hand every release.

If you tag without doing both, `landing/install.sh` still works (it always asks the GitHub API for the real filename) — but the landing page's direct "Download .deb" button will point at a 404 until `landing/index.html` is fixed and manually re-uploaded (see `scripts/deploy-production.sh`).

## Reporting bugs and requesting features

Open a GitHub issue. For security vulnerabilities, see [SECURITY.md](SECURITY.md) instead — do not file a public issue.

## Browser extensions

Changes under `extensions/` should still pass `extensions/build.sh` (produces the zips CI/reviewers would load) and should be manually tested via "load unpacked" (see `extensions/README.md`) before submission — there is no automated browser test suite in this repository yet.

## What not to submit

- Do not commit real API keys, license keys, or credentials anywhere, including in tests or fixtures.
- Do not add telemetry, analytics, or any network call to layout correction — it's a core project commitment that this path stays fully local and free. Network calls belong only in the already-explicit, opt-in cloud prompt-improvement path (`prompt-cloud`, gated by `--cloud`).
