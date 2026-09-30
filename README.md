# TypoMorph

**Version 1 is in development and is not release-ready.** Automatic application
correction is disabled in the current build because safe text replacement across
focus changes and concurrent input is not yet established. Do not use older
prototype packages as a substitute for the planned release.

TypoMorph is a local keyboard-layout correction utility in development for
multilingual writing in browsers, messages and documents. Input must stay in
bounded transient memory, with no typed-text uploads or logs in the release.

Current work is on [`diagnostic/device-echo-investigation`](https://github.com/artsensiva/TypoMorph/tree/diagnostic/device-echo-investigation).
See the [project checkpoint](docs/PROJECT_STATE.md) for verified progress and the
[resume instructions](docs/RESUME_DEVELOPMENT.md) for continuing after interruption.

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
| `core-engine` | Local language scoring, bounded analysis buffer, limited conversion, context and replacement guards |
| `platform-linux` | evdev input, uinput emission, GNOME-oriented layout switching |
| `daemon` | Linux CLI, live orchestration, basic tray, systemd and Debian packaging |
| `licensing` | Ed25519 verification of bounded offline trial/annual/perpetual entitlements; service integration pending |
| `settings` | Atomic, locked, content-free local preferences, including persistent pause and sounds |
| `native-host` | Bounded readiness-only Native Messaging endpoint; correction unavailable |

The classifier has English, Spanish, German, French, Russian, Ukrainian, and Hindi profiles. Physical layout conversion is based on incomplete US/RU tables; recognizing a language or returning a layout identifier does not establish correction support.

The live daemon evaluates words at a boundary rather than implementing the approved within-word behavior. Windows/macOS adapters, the full settings UI, Stripe accounts/billing, and end-to-end entitlement activation are not implemented. Offline signature and period verification is implemented and tested independently.

**The installed 0.2.2 prototype can print input characters and buffers to stderr.** The P0 working-copy change removes these live-path outputs and prompt notifications, with optional fixed stage labels instead. It has not replaced the installed application. Full field-safety and release-privacy requirements are still outstanding. AI/prompt modules are no longer exported or built, the cloud crate is excluded from the workspace, and legacy AI/license commands are removed. Historical source files remain outside the working build.

[BUG-001](docs/BUGS.md) records the owner's report that correction never occurs on Ubuntu 26.04.1 / GNOME 50 / Wayland, including after typing a space. An active tray icon does not prove that capture or replacement works. Isolated native-editor experiments now reproduce unsafe partial replacements under competing callbacks. The core simulator works, but safe ordinary application correction remains unresolved.

## Latest development evidence

As of 2026-10-01:

- The isolated GTK 4.22.4 prototype passes 29 buffer/history tests with ASan,
  UBSan and leak detection: bounded replacement, ordinary undo/redo, and targeted
  reversal preserving later disjoint typing.
- GtkTextView and GtkTextLayout integration remain blocked. Existing legacy
  callbacks need a compatible transaction contract; cache invalidation alone
  is insufficient. Refusal checks remain enabled.
- Tests use fixed synthetic text and disposable toolkit builds. They do not
  establish working Chrome, Text Editor, Windows or macOS autocorrection.

See the [prototype evidence and limitations](integrations/gnome/gtk-transaction/README.md)
and [native editor results](docs/NATIVE_TEXT_EDITOR_RESULTS.md).

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
| [Implementation plan](docs/IMPLEMENTATION_PLAN.md) | Approved work packages, dependencies, validation, and authorization boundaries |
| [Roadmap](docs/ROADMAP.md) | Approval gates, proposed delivery stages, and deferred work |
| [Open questions](docs/OPEN_QUESTIONS.md) | Unresolved decisions, feasibility risks, and release blockers |
| [Security](SECURITY.md) | Current caveats, release security boundaries, and reporting |

Approved owner decisions and the current workspace take precedence over historical descriptions. Development continues under [AGENTS.md](AGENTS.md) and the approved implementation plan.

## Development and simulation

The current full workspace is Linux-oriented. Install Rust with Rust 2021 support, `libdbus-1-dev`, and `pkg-config` before building.

```bash
cargo build --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

A device-free simulation accepts deliberately synthetic input:

```bash
printf '%s\n' 'ghbdtn' | cargo run --locked -p daemon -- test-input
```

This command prints the test input/result. It does not prove safe live correction in an application. See the [daemon notes](crates/daemon/README.md) before testing live input; `--dry-run` still reads real input devices.

See [CONTRIBUTING.md](CONTRIBUTING.md) for workflow and validation and [local testing](docs/LOCAL_MACHINE_TESTING.md) for machine-specific limits. Some integration tests require a private D-Bus session; follow their isolation instructions rather than capturing live typing.

## Distribution and verification status

The [release workflow](.github/workflows/release.yml) prepares draft Debian releases with a Sigstore bundle for their checksums. The installer requires an exact workflow/tag signing identity, a valid package hash, and explicit `--install`; it refuses legacy v0 packages, does not start the service, and does not enable autostart. New packaging omits broad input-device rules. This is source-level hardening with mocked installer tests, not a certified published artifact or a completed in-app updater. See [distribution evidence](docs/DISTRIBUTION.md).

The target release uses direct website downloads: `.deb`, a signed Windows installer, and a signed/notarized macOS application in `.dmg`. Stores and additional repositories are deferred. Browser extensions are conditional on demonstrated need and owner approval; Safari's extension is deferred.

The [landing page](landing/index.html) now describes development status and approved planned pricing. It contains no purchase links, installation commands or placeholder downloads. Updating this source does not deploy the website or create a release.

## Source license

The repository retains its existing [proprietary license notice](LICENSE). It was not replaced with an open-source license or new customer agreement during discovery. [LICENSING.md](docs/LICENSING.md) records approved commercial product requirements, not a finalized EULA or a grant of source-code rights.
