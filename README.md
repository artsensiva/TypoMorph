# TypoMorph

TypoMorph is being developed as a local keyboard-layout correction utility for everyday multilingual writing in browsers, messages, and documents.

**Status: existing Linux prototype; approved product requirements; documentation under review. The first public release described here is not implemented or verified yet.** Product discovery was approved on 2026-09-24. Documentation approval and subsequent implementation-plan approval are separate gates.

## Approved first-release product

- Correct wrong-layout text during word entry as soon as confidence is sufficient; preserve correct text and abstain when uncertain.
- Automatic correction, manual correction of the last word or a selection, and safe undo.
- Six input and interface languages: English, Russian, Ukrainian, German, French, and Spanish, using the explicitly selected layouts in the compatibility matrix.
- Ubuntu LTS with GNOME on Wayland and X11, Windows 11, and macOS; exact tested versions remain to be approved.
- Tray/menu-bar controls, a settings window, application exclusions, and optional sounds, off by default.
- Release input processing only in transient memory: no typed-text transmission, input logging, automatic telemetry, or automatic crash reporting.
- Seven-day account-wide trial without a card, then **USD 7/year with automatic renewal or USD 19 once for a perpetual license**. Both paid options cover three computers and the same features. The perpetual license includes all future released updates.
- No AI prompt improvement, spelling correction, permanent free tier, or Enterprise tier in this release.

Prices include applicable taxes. Payment is in USD; the website will also show a dated approximate EUR equivalent when its reference rate is sufficiently current. These are approved product requirements, not an announcement that checkout or licensing already exists.

## Current implementation and known limitations

The current workspace contains six crates:

| Crate | Current responsibility |
| --- | --- |
| `core-engine` | Language scoring, a 32-character ring buffer, limited layout conversion, and legacy prompt heuristics |
| `platform-linux` | evdev input, uinput emission, GNOME-oriented layout switching |
| `daemon` | Linux CLI, live orchestration, basic tray, systemd and Debian packaging |
| `licensing` | Legacy Lemon Squeezy activation and local status storage |
| `prompt-cloud` | Legacy optional network prompt-improvement client; excluded from the approved release scope |
| `native-host` | Existing browser Native Messaging process, independent of the running daemon |

The classifier has English, Spanish, German, French, Russian, Ukrainian, and Hindi profiles. Physical layout conversion is based on incomplete US/RU tables; recognizing a language or returning a layout identifier does not establish correction support.

The live daemon evaluates words at a boundary rather than implementing the approved within-word behavior. Windows/macOS adapters, the full settings UI, Stripe accounts/billing, and the approved entitlement system are not implemented.

**The diagnostic branch prints input characters and buffers to stderr.** A service manager can retain that output. The prototype therefore does not satisfy the approved release privacy requirements. Legacy cloud-prompt commands also still exist; their presence is not authorization to include them in the new product.

[BUG-001](docs/BUGS.md) records the owner's report that correction never occurs on Ubuntu 26.04.1 / GNOME 50 / Wayland, including after typing a space. An active tray icon does not prove that capture or replacement works. The report has not yet been independently reproduced.

## Documentation map

| Document | Purpose |
| --- | --- |
| [Product specification](docs/SPEC.md) | Approved behavior, scope, and release criteria |
| [Decision log](docs/DECISIONS.md) | Final decisions and superseded historical requirements |
| [Compatibility](docs/COMPATIBILITY.md) | Language/layout/platform/browser targets and evidence status |
| [Licensing and commerce](docs/LICENSING.md) | Trial, subscription, perpetual access, devices, payments, and refunds |
| [Privacy](docs/PRIVACY.md) | Input boundaries, operational data, and diagnostics |
| [Architecture](docs/ARCHITECTURE.md) | Observed implementation and target constraints, without a final technology selection |
| [Testing](docs/TESTING.md) | Required automated and real-application evidence |
| [Known bugs](docs/BUGS.md) | User-reported reproduction record and investigation status |
| [Roadmap](docs/ROADMAP.md) | Approval gates, proposed delivery stages, and deferred work |
| [Open questions](docs/OPEN_QUESTIONS.md) | Unresolved decisions, feasibility risks, and release blockers |
| [Security](SECURITY.md) | Current caveats, release security boundaries, and reporting |

The local workspace and approved owner decisions take precedence over older GitHub descriptions, source comments, historical prompts, and landing-page copy. The documents in this map are a reviewable transcription of the approved discovery; they do not authorize application changes.

## Development and simulation

The current full workspace is Linux-oriented. Install Rust with Rust 2021 support, `libdbus-1-dev`, and `pkg-config` before building.

```bash
cargo build --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

A device-free simulation accepts deliberately synthetic input:

```bash
printf '%s\n' 'ghbdtn' | cargo run -p daemon -- test-input
```

This command prints the test input/result. It does not prove safe live correction in an application. See the [daemon notes](crates/daemon/README.md) before testing live input; `--dry-run` still reads real input devices.

See [CONTRIBUTING.md](CONTRIBUTING.md) for workflow and validation. No tests or live input experiments were run as part of the discovery/documentation update.

## Distribution and verification status

The existing [release workflow](.github/workflows/release.yml) builds a Debian package and is configured to sign its checksum file using cosign. This describes workflow configuration, not verification of any downloaded artifact. The existing installer checks checksums; it is not the approved fail-closed signed updater.

The target release uses direct website downloads: `.deb`, a signed Windows installer, and a signed/notarized macOS application in `.dmg`. Stores and additional repositories are deferred. Browser extensions are conditional on demonstrated need and owner approval; Safari's extension is deferred.

The [landing page](landing/index.html) still contains obsolete free/Pro pricing and a Lemon Squeezy link. It has not been updated or published during this documentation-only phase. Existing Windows/macOS download placeholders are not evidence of native application support.

## Source license

The repository retains its existing [proprietary license notice](LICENSE). It was not replaced with an open-source license or new customer agreement during discovery. [LICENSING.md](docs/LICENSING.md) records approved commercial product requirements, not a finalized EULA or a grant of source-code rights.
