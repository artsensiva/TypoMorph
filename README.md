# TypoMorph

[![Rust 2021](https://img.shields.io/badge/rust-2021-orange?logo=rust)](https://www.rust-lang.org/)
[![Linux](https://img.shields.io/badge/platform-Linux-ffc107?logo=linux&logoColor=black)](https://www.kernel.org/)
[![Wayland](https://img.shields.io/badge/Wayland-supported-78e5dc)](https://wayland.freedesktop.org/)
[![X11](https://img.shields.io/badge/X11-supported-78e5dc)](https://www.x.org/)
[![License](https://img.shields.io/badge/license-proprietary-lightgrey)](LICENSE)

> Intelligent cross-platform input daemon for multilingual typing, layout recovery, and privacy-first desktop input.

TypoMorph detects when text was entered with the wrong keyboard layout and corrects it locally. The runtime is built around a bounded Rust engine, short-lived in-memory context, and native Linux input integration. It is designed for low-latency desktop typing without telemetry, cloud classification, or raw keystroke persistence.

**Zero telemetry. Local in-memory processing. Sub-millisecond classification target.**

## Feature Highlights

- **Real-time intelligent typo detection**: compares the active layout with an alternate-layout projection and uses language signals before proposing a switch.
- **Sub-millisecond keystroke replacement**: captures Linux key events through `evdev` and emits guarded Backspace/replacement sequences through `uinput`.
- **Systemd user-daemon integration**: runs as a per-user service and supports udev access rules for `/dev/uinput` and `/dev/input/event*`.
- **Zero telemetry and total local privacy**: classification uses volatile bounded buffers; raw keystrokes and user text are not sent to a remote service.
- **Wayland and X11 model**: Linux device access is handled at the input layer, with GNOME Shell D-Bus integration for layout switching.
- **Offline-first licensing**: layout correction is unlimited and free for every supported language with no account at all; optional licensing state (stored locally as hashed metadata) only ever gates the Pro cloud prompt-improvement backend.
- **Interactive simulation**: `typomorph test-input` exercises the classifier entirely in user space without root access or Linux input devices.

## How It Works

TypoMorph keeps the hot path small and conservative:

1. The Linux adapter receives a raw key event from an `evdev` device.
2. The daemon appends the corresponding character to a bounded 32-token ring buffer.
3. The core classifier scores the observed text and its alternate keyboard-layout projection.
4. A switch is considered only when the alternate candidate is coherent enough and clears the configured confidence margin.
5. The platform adapter requests a layout change through GNOME Shell D-Bus and can emit a minimal replacement sequence through `uinput`.
6. Native composition and developer-context guards can suppress mutation when the input context is unsafe.

The system prefers a safe no-op over an ambiguous rewrite.

## Quick Install

### One-line installer

On a supported Debian or Ubuntu system:

```bash
curl -sSL https://raw.githubusercontent.com/artsensiva/TypoMorph/main/landing/install.sh | bash
```

The installer fetches the latest signed `.deb` from [GitHub Releases](https://github.com/artsensiva/TypoMorph/releases), verifies its SHA-256 checksum, installs the user-service and udev integration, reloads the user systemd manager, and enables `typomorph.service`. If no matching release asset is found, it prints instructions to build from source instead of failing silently. Review remote install scripts before executing them in production environments. See [Verifying releases](#verifying-releases) below to check a downloaded package manually.

### Install a downloaded Debian package

```bash
sudo apt install ./typomorph_0.1.0_amd64.deb
# or:
sudo dpkg -i ./typomorph_0.1.0_amd64.deb
sudo apt-get -f install
```

Enable the user daemon after installation:

```bash
systemctl --user daemon-reload
systemctl --user enable --now typomorph.service
systemctl --user status typomorph.service
```

The Debian package includes `99-typomorph.rules`, which grants the active desktop session access to uinput and input event devices through `uaccess`. A new login session may be required after installing or changing udev permissions.

### Build from source

Requirements:

- Rust stable with Rust 2021 support
- Linux with `evdev`, `/dev/uinput`, and a supported desktop session
- Permission to read the relevant `/dev/input/event*` device
- A user systemd session for service integration

Build the workspace:

```bash
cargo build --release
```

The daemon binary is produced at:

```text
target/release/typomorph
```

Build the Debian package with the repository helper:

```bash
./scripts/build-deb.sh
```

### Verifying releases

Every tagged release publishes `typomorph_<version>_amd64.deb`, a `SHA256SUMS` file, and a keyless [cosign](https://docs.sigstore.dev/) signature over that checksum file (`SHA256SUMS.sig` / `SHA256SUMS.pem`), built and signed by the `Release` GitHub Actions workflow.

```bash
# 1. Verify the checksum file's signature against the GitHub Actions OIDC identity
cosign verify-blob \
  --certificate SHA256SUMS.pem \
  --signature SHA256SUMS.sig \
  --certificate-identity-regexp "https://github.com/artsensiva/TypoMorph/.github/workflows/release.yml@.*" \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  SHA256SUMS

# 2. Verify the .deb matches the signed checksum
sha256sum -c SHA256SUMS
```

`landing/install.sh` performs step 2 automatically; step 1 is worth doing yourself if you download artifacts manually.

## Interactive Simulation

Test the classifier without `/dev/input`, `/dev/uinput`, D-Bus, or root permissions:

```bash
printf '%s\n' \
  'ghbdtn' \
  'FHNTV' \
  'руддщ цщкдв' \
  | cargo run -p daemon -- test-input
```

Typical output includes the detected language, confidence, switch decision, target layout, and corrected buffer:

```text
detected=Russian confidence=0.98 switch=yes target=Some("ru") corrected="привет"
detected=Russian confidence=0.98 switch=yes target=Some("ru") corrected="АРТЕМ"
detected=English confidence=0.94 switch=yes target=Some("us") corrected="hello world"
```

Raw keycode input is also supported:

```text
KEYS 16 17 18 18 24
```

Adjust the simulation threshold when experimenting with ambiguous input:

```bash
cargo run -p daemon -- test-input --threshold 0.60
```

## CLI

```text
typomorph run [--input /dev/input/event0] [--layout us] [--dry-run]
typomorph test-input [--layout us] [--threshold 0.60]
typomorph improve-prompt --stdin [--cloud] [--api-key KEY]
typomorph license activate <KEY>
typomorph status
```

`--dry-run` keeps the daemon from opening uinput or connecting to GNOME Shell D-Bus, while still requiring an evdev input source for the live path. Use `test-input` for a completely device-free simulation.

`improve-prompt` reads a prompt from stdin, detects whether it looks like an AI prompt, and cleans it up. Without `--cloud` it never touches the network — see [Free vs Pro: prompt improvement](#free-vs-pro-prompt-improvement) below for what `--cloud` does and requires. While `run` is active and about to auto-correct a buffer that looks prompt-like, TypoMorph also shows a desktop notification suggesting `Ctrl+Alt+I` as a shortcut to run the same local improvement instead of switching layout.

## Workspace Architecture

The workspace separates portable classification from Linux-specific input operations:

### `core-engine`

Portable language and layout decision engine. It provides:

- a bounded `RingBuffer<32>` for recent input;
- script-aware normalization (Latin, Cyrillic, Devanagari);
- Laplace-smoothed n-gram scoring;
- illegal-sequence hard filtering;
- confidence and fallback logic for conservative switching;
- `prompt_detector`: offline heuristic scoring of whether text looks like an AI prompt;
- `prompt_improver`: offline, deterministic, free-forever prompt cleanup (no AI, no network).

The core crate does not depend on Linux, D-Bus, uinput, network access, or desktop services.

### `daemon`

The executable orchestration layer and CLI. It provides:

- the `typomorph` binary;
- the `run`, `test-input`, `improve-prompt`, `license`, and `status` commands;
- unlimited layout routing for every supported language, free for all users;
- alternate-layout candidate evaluation;
- developer-window filtering;
- a local prompt-improvement hotkey (`Ctrl+Alt+I`) and pre-switch notification hint;
- systemd user-service integration.

### `platform-linux`

Linux-native input and desktop integration. It provides:

- raw keyboard monitoring with `evdev`;
- synthetic key emission with `/dev/uinput`;
- GNOME Shell D-Bus layout switching;
- platform error boundaries and input event types.

### `licensing`

Offline and online license-state handling. It provides Lemon Squeezy activation transport, local integrity-checked status storage, hashed license metadata, and Pro entitlement checks — used only to gate the managed cloud prompt-improvement backend, never language or layout correction.

### `prompt-cloud`

Client for the paid, network-based prompt-improvement path. Two backends: bring-your-own-key (calls the Anthropic API directly with a user-supplied key, free) and managed (calls TypoMorph's hosted proxy, requires an active Pro license). Never invoked unless `--cloud` is passed explicitly.

### `native-host`

Stateless Chrome/Firefox Native Messaging host (`typomorph-native-host`) for the browser extensions in [`extensions/`](extensions/README.md). Reuses `core-engine` and `prompt-cloud` directly rather than talking to the running `typomorph` daemon over IPC — Chrome spawns a fresh subprocess per connection, so there is nothing long-lived to connect to.

## Browser extensions

`extensions/` holds thin-client Chrome and Firefox extensions (Manifest V3) that relay selected page text to the local native host for free layout correction and prompt improvement — no network access unless you opt into cloud improvement in the popup. See [`extensions/README.md`](extensions/README.md) for loading, native-messaging-host registration, and packaging. Not published to any store; that remains a manual step.

## Privacy and Security Model

TypoMorph is designed around the following boundaries:

- raw input is processed in memory and kept inside a bounded active context;
- no raw keystroke logging is written to disk by the engine;
- no cloud language API is required for classification;
- the local license file stores hashes and status metadata rather than the raw license key;
- developer-mode and active-composition safeguards can suppress automatic mutation;
- Linux device access is granted through udev session access rather than running the daemon as root.

The Linux systemd unit should run as the logged-in user. Do not run the daemon as root unless you have a specific deployment policy that requires it.

### Free vs Pro: prompt improvement

Layout correction (this project's core feature) is entirely free, offline, and unlimited for every supported language — it never touches the network, licensed or not. Prompt improvement is the one feature with a network-connected path:

| | Local (always available) | Bring-your-own-key (`--cloud --api-key`) | Managed (`--cloud`, Pro) |
|---|---|---|---|
| Cost | Free | Free (you pay Anthropic directly) | Paid TypoMorph subscription |
| Network | None | Your prompt is sent to `api.anthropic.com` | Your prompt is sent to TypoMorph's proxy, which forwards it to Anthropic |
| Requires | Nothing | Your own Anthropic API key | An active Pro license |
| Quality | Deterministic rule-based cleanup (whitespace, punctuation, abbreviations) | Full AI rewrite | Full AI rewrite |

Nothing is ever sent to any network unless you pass `--cloud` explicitly. `typomorph run`'s live hotkey (`Ctrl+Alt+I`) and pre-switch notification only ever use the local, offline path.

## Development

Run formatting, tests, and workspace checks:

```bash
cargo fmt --all
cargo test --workspace
cargo check --workspace
```

Run only the daemon tests:

```bash
cargo test -p daemon
```

Build and inspect the Debian package:

```bash
./scripts/build-deb.sh
dpkg-deb -c target/debian/typomorph_*.deb
```

## Repository Layout

```text
TypoMorph/
├── .github/workflows/     # CI (fmt/clippy/test/audit) and tagged-release automation
├── crates/
│   ├── core-engine/       # Portable classifier, layout correction, prompt detector/improver
│   ├── daemon/            # typomorph CLI, runtime orchestration, packaging
│   ├── licensing/         # Offline/online Pro entitlement state
│   ├── platform-linux/    # evdev, uinput, and GNOME Shell integration
│   ├── prompt-cloud/      # BYOK / managed cloud prompt-improvement client
│   └── native-host/       # Native Messaging host for the browser extensions
├── extensions/            # Chrome/Edge and Firefox browser extensions (unpublished)
├── docs/                  # Product specification, architecture, and roadmap
├── landing/               # Static download and product landing page
├── scripts/               # Release and Debian packaging helpers
├── SECURITY.md
├── CONTRIBUTING.md
├── Cargo.toml
└── LICENSE
```

## License

The repository currently contains a proprietary license notice in [LICENSE](LICENSE). The README intentionally reflects that legal status rather than claiming MIT or Apache-2.0 licensing. If the project is formally relicensed under MIT, Apache-2.0, or both, update the license file and badges together.

## Status

TypoMorph is under active development. The Linux Phase 2 path, daemon CLI, interactive simulation, Debian packaging, udev rules, systemd user service, and static release landing page are implemented. macOS and Windows download artifacts remain release placeholders until their native adapters are available.
