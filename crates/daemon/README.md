# TypoMorph daemon

## Status

This is the existing Linux prototype, not the approved cross-platform release. It currently evaluates input at word boundaries, uses limited layout mapping, and prints live input diagnostics to stderr. Service logs can retain that output.

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

Legacy prompt-improvement and Lemon Squeezy license commands remain in the source. They are outside the approved new product model. Documentation changes do not remove those commands or establish Stripe/trial support.

## Validation

Follow [TESTING.md](../../docs/TESTING.md). Simulation success alone cannot close BUG-001, prove safe undo, establish all language/layout mappings, or demonstrate Windows/macOS support.
