# Replacement safety boundary

Status: unsafe automatic emission disabled in the working copy, 2026-09-25. Safe live replacement is not implemented. Installed binaries are unchanged.

## Finding

The previous daemon checked field metadata, switched layout, checked metadata again, then emitted independent backspace and character key events. Physical input and focus changes could occur after the final check or between emitted events. Stopping after a partial deletion would still corrupt text. Suppressing delivery to the analysis channel did not suppress physical input to the application; it only hid those events from the daemon.

The [kernel uinput interface](https://www.kernel.org/doc/html/latest/input/uinput.html) creates a virtual input device and emits events. The current adapter has no operation that binds a replacement to a field or serializes it with the application's input processing. More polling, shorter sleeps, one larger event batch, or an evdev grab cannot by themselves supply a target-bound text transaction (nor account for every pointer, accessibility or other input source).

## Current behavior

- Ordinary `run` refuses with `automatic replacement unavailable` before license I/O, accessibility discovery, device capture, tray startup, layout mutation or virtual-keyboard creation.
- `run --dry-run` remains explicitly controlled analysis only. Its candidate path cannot switch layouts, suppress events, emit text, or pretend the source layout changed after a hypothetical correction.
- The library's legacy `UinputKeyboard::replace_text` also refuses. Low-level emission helpers remain legacy primitives; they are not approved automatic replacement mechanisms.
- Read-only metadata/backend checks and device-free simulations remain available.
- No unsafe override flag was introduced.

This does not complete input lifecycle/privacy work: raw capture queues, pause, event-driven idle invalidation and source tracking in diagnostic mode remain unresolved. The refusal is a safety boundary, not a successful correction implementation or closed BUG-001.

## Required replacement contract

A viable adapter must bind the operation to a specific editable context and serialize validation/application with new input. The request must carry expected context identity, context/input revisions, caret/selection state and a complete replacement plan. A focus transition away and back must invalidate the old revision. New input must invalidate or be ordered after the replacement without loss. Rejection must occur before any deletion or source change. Partial failure must be explicit; do not perform blind rollback into a newly focused field.

Focus/selection/input event notifications can invalidate pending analysis, but notifications alone cannot make global uinput editing transactional. Do not subscribe indiscriminately to accessibility text-change signals: their payloads can contain text. Any event implementation must establish content-free subscription boundaries first.

Next P1/P2 work: evaluate a native input-method integration and any application-bound editing API against this contract on GNOME Wayland, including Chrome and Text Editor. Do not assume either approach meets the requirements. A required-target capability conflict needs an owner decision per IMPLEMENTATION_PLAN.md; no product-scope change has been made.

## Validation

The CLI regression test runs with an unavailable session bus and configuration path and expects the replacement-safety refusal, proving that the ordinary startup path reaches the gate first. Full workspace tests, Clippy and the debug build passed. This step ran no live capture or synthetic key emission. Existing private-bus protocol suites were not rerun because their metadata/layout implementation was unchanged.

## Input-method review

The [IBus feasibility investigation](IBUS_FEASIBILITY.md) recommends an isolated preedit experiment. Available APIs do not yet establish a safe live adapter; the automatic-run gate remains in place.
