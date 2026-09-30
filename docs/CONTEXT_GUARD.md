# Metadata context guard prototype

Status: implemented and tested with synthetic objects; real-session probing exposed a null-reference traversal bug, now covered by a regression test. After the GTK 4 compatibility fix, an owner-coordinated blank Text Editor metadata check succeeded on 2026-09-25. This verifies one real-session eligibility snapshot, not live correction or complete context safety.

## What is checked

The AT-SPI reader uses only object references, roles, state bitsets, interface availability, toolkit name/version, caret position, character count, and selection count. It never requests text, accessible names/descriptions, window titles, clipboard data, or the bulk accessibility cache.

A candidate must be an editable focused Text/Entry object under an active window, with sensitive/showing/visible state and enabled state (except a verified GTK 4 provider), a valid caret, and no selection. Password, terminal, unknown, defunct, ambiguous, and inaccessible objects are refused. State/role/position are re-read, and recognition of a protected role prevents subsequent Text-interface queries.

Discovery is bounded by 256 objects, depth 32, and 500 ms per operation. A timeout or incomplete traversal denies access. These conservative limits may reject legitimate complex applications and are not a performance/support claim.

The engine guard tracks opaque app/window/field identity and expects exactly one-character caret/count advancement for each observed insertion. Mismatches reset the word. Non-text edits, observed modifier activity, repeats, missing metadata, and unavailable/changed GNOME source invalidate processing. Metadata is checked again before switching and before emission.

Both normal run and explicit dry-run require initial eligible field metadata before opening keyboard capture. The installed binary was not replaced. Device-free stdin simulation remains separate.

## Read-only diagnostic

`target/debug/typomorph check-input-context` discovers eligibility and prints only a fixed outcome or rejection code (for example `object_role_unavailable` or `metadata_timeout`), without object identifiers or field values. It does not start evdev capture, read field contents, move the caret, change layouts, or edit text. A successful result is metadata eligibility, not an end-to-end safety certification.

For the first real check, use a blank Text Editor with the caret inside the document and no selection. Coordinate focus timing with the owner; do not assume the chat/editor window is still focused after a reply.

## Tests

- Four core tests: expected insertion, pre-edit comparison, app/window/field changes, movement/paste/deletion, and unavailable context.
- Two native-reader tests: permitted roles/states and bounded wait.
- An explicit private-bus protocol test against a synthetic AT-SPI tree covers valid metadata, password fields, missing focus, selections, inactive windows, invalid caret, and a field becoming protected during the snapshot. It checks that no Text-interface queries follow recognition of protection. Repeated AT-SPI null references are ignored, while an unavailable non-null object still rejects the snapshot.
- Run the protocol test with `python3 integrations/gnome/tests/run_context_test.py`. It is intentionally ignored by the ordinary workspace suite.

## Remaining limits before general live correction

These snapshots do not form an atomic transaction with keyboard delivery. In particular:
- focus/cursor can change and return between observations;
- equal-length content changes are not detectable from counts;
- application accessibility updates can lag physical events;
- another key/focus change can arrive after the final check or during backspaces;
- event-driven invalidation and immediate clearing on idle focus transitions are not implemented;
- raw capture queue/pause lifecycle, modifier state at startup/Caps Lock, and complete application exclusions still need work;
- a manual source change currently causes conservative suspension until the tracked source matches again; the approved resume-at-boundary behavior remains to implement;
- if context is lost after source switching, replacement is canceled but the already-changed source is not blindly restored into a different context.

Do not treat this component or successful tests as completed release privacy, undo, IME, input-integrity, or latency requirements. Real editing tests must remain controlled and synthetic until the transaction/lifecycle work is verified.

## Protocol references

The implementation follows GNOME's [Accessible interface](https://raw.githubusercontent.com/GNOME/at-spi2-core/main/xml/Accessible.xml) and [Text interface](https://raw.githubusercontent.com/GNOME/at-spi2-core/main/xml/Text.xml). Enum values were cross-checked against the installed AT-SPI introspection library without querying applications.

AT-SPI null-object semantics: [GNOME Accessible protocol reference](https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Accessible.html).

## Observed compatibility blocker

On 2026-09-25, the owner-coordinated role/state probe observed a focused editable TEXT object under an active window with SENSITIVE/SHOWING/VISIBLE true but ENABLED false. The production probe refused it. The subsequent GTK 4 compatibility change is described below; a coordinated metadata eligibility check has since succeeded. A separate expanded traversal hit its node limit, so it does not establish globally unique focus.

## GTK 4 state compatibility

The working copy now permits absent ENABLED only when the field's GetApplication reference belongs to the same bus provider and individual Application.ToolkitName/Version properties identify GTK 4 with a numeric three-part version. Missing, malformed or different toolkit metadata keeps the refusal. GTK 4.22.4 exports SENSITIVE according to disabled state but does not set ENABLED in collect_states. Version is used because this GTK release does not expose the newer ToolkitVersion property. No GetAttributes/GetAll requests are used: they could include text-bearing values.

All base requirements and repeated checks remain, and READ_ONLY now explicitly rejects even contradictory EDITABLE state. The private-bus fixture covers GTK 4 without ENABLED, absent SENSITIVE, READ_ONLY, another toolkit, GTK 3, malformed/missing metadata, a password role, and sensitivity lost during revalidation. This is provider-reported compatibility metadata, not authentication or an atomic edit guarantee. A coordinated blank Text Editor metadata check succeeded on 2026-09-25; live editing remains unverified.

Sources: [GTK 4.22.4 state mapping](https://raw.githubusercontent.com/GNOME/gtk/4.22.4/gtk/a11y/gtkatspicontext.c), [GTK application properties](https://raw.githubusercontent.com/GNOME/gtk/4.22.4/gtk/a11y/gtkatspiroot.c), [AT-SPI state semantics](https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/enum.StateType.html).

## Controlled selection and transition checks

The live synthetic selection case returned `selection_present` after the owner was given 30 seconds to prepare it. The read-only `cargo run --offline -p platform-linux --example check_context_transition` diagnostic captures an eligible baseline after 45 seconds and compares metadata after another 120 seconds using ContextGuard. It prints fixed outcomes only and opens no input devices. Its first live run returned unchanged context; the owner confirmed insufficient time to switch. The longer coordinated rerun exited 0 and refused the original context after the switch. The outcome combines changed and unavailable metadata and does not prove detection of a focus-out-and-back transition between snapshots.

## Automatic editing gate

As of 2026-09-25, ordinary `run` refuses before capture or mutation because metadata snapshots cannot protect the legacy global uinput emission sequence. See [replacement safety boundary](REPLACEMENT_SAFETY.md). The successful metadata tests above remain valid; they do not authorize live replacement.
