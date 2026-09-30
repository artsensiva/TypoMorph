# GNOME Layout Bridge prototype

Status: local P1 prototype for GNOME Shell 50. Installed with owner approval; ACTIVE status, session-bus connection, and read-only source reporting verified after login. Real-session RU → US → RU switching and restoration were confirmed with independent state reads. Text replacement and field safety remain unverified.

## Purpose and boundaries

This is a GNOME Shell companion, not a browser extension. It exposes only source identity and source activation over the current user's session bus. It contains no input hooks, text access, network, file storage, or arbitrary JavaScript evaluation endpoint.

The current user can access this session-bus interface; it is not a security boundary against other processes running as that same user. No root service or Shell unsafe mode is required. GNOME Shell-specific imports are version-dependent: metadata currently advertises only 50, the locally inspected version.

This component does not identify password fields or make global capture/text replacement safe. Those P2/P3 requirements remain outstanding. Do not run the live correction daemon on ordinary personal input merely because this bridge is available.

## Protocol version 1

Bus/interface: `org.typomorph.Layout1`; object: `/org/typomorph/Layout1`.

- `GetState() -> (u version, b ready, s type, s id, as installedXkbIds)`.
- `Switch(s expectedSource, s target) -> (b confirmed, s type, s id)`.

State is unavailable while locked, at the greeter, while the keyboard manager is locked, for IME/unknown sources, or after disable. A switch requires the exact expected source and an installed XKB target. Confirmation requires the manager's current source to match the requested target after activation. The Rust client rechecks source state before returning success.

This is confirmation of GNOME's input-source state, not proof that the compositor has finished processing a keymap or that subsequent text replacement is safe. Source-to-replacement races, focus identity, and actual visible output still require real-session validation and the later context transaction.

The Rust client uses a 250 ms deadline per connection/call as a failure bound, not a claim to meet the 100 ms end-to-end release target. Timeout cancels local waiting; it cannot retract a request already delivered to Shell. On timeout the daemon must not replace text. No ignored-setting or Eval fallback remains.

The current daemon initializes from the bridge's actual US/RU source rather than assuming US. Other layouts are rejected before capture until their mappings are implemented. Its explicit dry-run uses the requested simulated layout and still requires a controlled session.

## Local checks

From the repository root:

```bash
node --test integrations/gnome/tests/service.test.mjs
python3 integrations/gnome/tests/run_dbus_test.py
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
bash scripts/build-gnome-bridge.sh
```

The Python runner creates a private D-Bus session, exports the actual GJS service wrapper with synthetic state, and runs the ignored Rust protocol test there. It does not load the companion into GNOME Shell or access input devices. The ignored test is not a pass unless this runner has been run successfully.

## Package and controlled installation

The build script produces `target/gnome/layout-bridge@typomorph.com.shell-extension.zip` containing metadata, the Shell adapter, and the service module.

Before installing, inspect any existing extension with the same UUID and preserve it rather than blindly replacing it. For a new installation, the intended user-session steps are:

```bash
gnome-extensions install target/gnome/layout-bridge@typomorph.com.shell-extension.zip
gnome-extensions enable layout-bridge@typomorph.com
cargo run -p daemon -- check-layout-backend
```

These commands are documentation, not a record of installation. A fresh extension may require a new graphical login before GNOME discovers it. Do not restart/log out the user's session automatically. Start with the non-capturing backend check. Coordinate any layout-changing test separately; do not start global capture for this check.

To disable the companion:

```bash
gnome-extensions disable layout-bridge@typomorph.com
```

Disabling it removes the exported service and makes the Rust client refuse correction. The existing installed TypoMorph binary is not updated by installing this package.

See [P0/P1 investigation](../../docs/P0_INVESTIGATION.md) and the [approved plan](../../docs/IMPLEMENTATION_PLAN.md).
