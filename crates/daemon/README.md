# TypoMorph daemon

> Working-copy safety status (2026-09-25): automatic `run` is disabled because the current uinput path cannot safely serialize replacement with focus changes and new input. Read-only diagnostics and explicit controlled debug-build `--dry-run` remain available. This is not a release-ready build.

## Status

This is the existing Linux prototype, not the approved cross-platform release. It currently evaluates input at word boundaries and uses limited layout mapping. The P0 working copy removes live text/key diagnostics and prompt notifications; the installed 0.2.2 binary has not been replaced.

The owner reports no automatic correction in Chrome or Text Editor on Ubuntu GNOME Wayland. [BUG-001](../../docs/BUGS.md) remains independently unreproduced. An active tray icon is not proof of working capture or replacement.

See [SPEC.md](../../docs/SPEC.md) for required behavior and [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) for the gap between source and requirements.

## Device-free input simulation

Use deliberately synthetic input without opening input devices, uinput, or D-Bus:

```bash
printf '%s\n' 'ghbdtn' | cargo run -p daemon -- test-input
```

The command reads stdin lines, places their characters in a 32-character ring buffer, and prints the classification and proposed correction. Long lines retain only the ring buffer's tail. This does not simulate field safety, real replacement, or the whole application's memory bounds.

A line prefixed with `KEYS` accepts Linux keycodes through the current limited mapping:

```text
KEYS 16 17 18 18 24
```

Set the starting simulated layout and confidence threshold:

```bash
cargo run -p daemon -- test-input --layout ru --threshold 0.65
```

The defaults are `us` and `0.60`. The simulator can infer a layout from input text and updates its simulated layout after a switch; the initial flag is not a guarantee that every line is evaluated under that layout.

## Live input and legacy commands

The current `run` command has `--layout` and `--dry-run`, not an `--input` device-selection argument. Dry-run avoids replacement but still reads actual input devices and emits diagnostics. It is not a privacy-safe alternative to device-free simulation.

Do not capture personal typing or publish raw diagnostic output. Live investigation must use controlled synthetic input under the later approved implementation plan. Release builds must satisfy the no-input-logging policy in [PRIVACY.md](../../docs/PRIVACY.md).

Legacy prompt-improvement and Lemon Squeezy activation commands are removed.
`pause`, `resume`, and `sounds true|false` persist content-free preferences.
`status` reports these preferences and unavailable correction/account integration.
Resume does not itself start capture. Diagnostic pause closes readers and clears
buffers; overflow/device loss ends the diagnostic rather than replaying gaps.
Preference updates are polled every 100 ms while diagnostics run; live timing
and device teardown still require controlled hardware validation. Sound playback
and a full settings UI are not implemented by the saved preference alone.

## Validation

Follow [TESTING.md](../../docs/TESTING.md). Simulation success alone cannot close BUG-001, prove safe undo, establish all language/layout mappings, or demonstrate Windows/macOS support.

## P0 stage diagnostics

The working-copy `run --diagnostics` option emits fixed stage labels without text, keycodes, device identifiers, or event timestamps. It is off by default. `--dry-run` still captures live input and is suitable only for a coordinated synthetic session; it does not change application text or OS layout. A stage named `switch_returned` or `replacement_returned` is not proof that the corresponding visible operation succeeded.

See [P0 investigation](../../docs/P0_INVESTIGATION.md) for evidence and the controlled test protocol.

## Non-capturing backend check

Run `cargo run -p daemon -- check-layout-backend` to check access without opening keyboard devices, changing the layout, or replacing text. The working copy now uses the [GNOME Layout Bridge](../../integrations/gnome/README.md), with no Shell Eval fallback. It returns an unsupported-backend error until the compatible companion is enabled. Production `run` refuses before any backend/capture setup; the backend check is a separate diagnostic. Availability alone does not establish safe text replacement; no weakening of GNOME protection is required or recommended.

## Input-context metadata check

The working-copy `check-input-context` command reports whether the focused field exposes eligible AT-SPI metadata, without reading its text, capturing keys, or changing anything. Diagnostic `--dry-run` requires eligible initial metadata; production `run` is already gated before this step. See [context guard](../../docs/CONTEXT_GUARD.md) for tests, conservative rejection behavior, and remaining race/lifecycle limitations. A successful metadata check does not authorize general live correction.
