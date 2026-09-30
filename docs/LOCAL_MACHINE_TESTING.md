# Local machine testing

Updated: 2026-09-30. This is a development test guide, not a release installer.

## Available machines and limits

- Owner-confirmed Ubuntu 26.04.1 LTS, GNOME 50, Wayland, x86-64 on a
  Lenovo ThinkCentre M720q. The earlier `24.06` report is superseded.
- Owner offers Windows 10 Home. Build number, architecture and development
  tools are not yet recorded. Treat this as experimental testing; Windows 10
  remains deferred from the approved release matrix.

The Linux working copy can run device-free tests and input simulation. Normal
automatic correction deliberately refuses to start because safe application
replacement is unresolved. There is no native Windows application or installer
yet. Core tests on Windows do not establish Windows application compatibility.

Use the current working copy: substantial changes are uncommitted, so a fresh
clone of the remote repository is not an equivalent test build. Do not install
the old published package for this test. These commands do not require starting
the daemon, enabling autostart, granting input-device access or installing a
browser extension.

## Ubuntu: safe checks now

From the current repository, with the existing Rust and Linux build dependencies:

```bash
cargo test --locked -p core-engine
printf '%s\n' 'ghbdtn' | cargo run --locked -p daemon -- test-input
cargo run --locked -p daemon -- status
```

The fixed simulator input should propose `corrected="привет"`, `switch=yes`
and target `ru`. It does not edit Text Editor, switch the actual system layout,
or read keyboard devices. It prints its simulated result, so use only this
synthetic input. `status` should report automatic correction and account
activation as unavailable; that is expected, not a setup error.

Cargo may download build dependencies when they are not cached. The simulator
does not transmit input. Add `--offline` to Cargo commands when dependencies
are already cached to disallow Cargo downloads.

Do not start a live `--dry-run` for this protocol: that diagnostic captures
keyboard events and needs a separately coordinated synthetic session.

## Windows 10 Home: core tests only

Prerequisites: a copy of this current source tree and a working native Windows
Rust toolchain/linker. Do not copy Linux `target/` build artifacts. If development
tools are missing, report that before attempting a full workspace build.

In PowerShell, from the copied repository root:

```powershell
rustc --version
cargo --version
cargo test --locked -p core-engine
```

Run this on Windows directly, not in WSL. Do not run `cargo build --workspace`
or the daemon: those include Linux-specific components. This core test command
has been verified on Linux only; its Windows result is still pending. Tests
include a timing assertion, so report the failing test name if one fails rather
than treating every failure as a layout-conversion failure.

Record Windows edition, version/build (from `winver`), and system type
(Settings → System → About). Share only these fields; product/device IDs and
account details are unnecessary.

## Evidence and next application test

For each run, record date, source snapshot, OS/session, toolchain version,
test counts and any failing test names. Linux core tests currently pass 45/45;
the synthetic `ghbdtn` simulation succeeds. No Windows result is recorded yet.

Owner-reported confirmation (2026-09-30): on the Ubuntu machine above,
`printf '%s\n' 'ghbdtn' | cargo run --locked -p daemon -- test-input` completed
with `detected=Russian confidence=0.98 switch=yes target=Some("ru")`
and `corrected="привет"`. This confirms the local simulator only; no application
replacement, actual layout switch or keyboard capture was tested. The command
used the working tree; an immutable source snapshot was not recorded.

Real application testing remains a separate gate. The isolated patched GNOME
Text Editor experiment succeeds for the simple conversion and ordinary undo/redo,
but reproduces unsafe partial edits under competing callbacks. See
[native results](NATIVE_TEXT_EDITOR_RESULTS.md). Do not use that experimental
editor with personal documents or interpret its happy path as a working beta.

Next: resolve safe replacement on Linux and implement a Windows adapter before
providing ordinary application autocorrection tests or native installable builds.
