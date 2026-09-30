# IBus feasibility investigation

Date: 2026-09-25. Decision: pursue an isolated preedit prototype, not enable live correction. This is a source/API review and local capability check, not an application compatibility test.

## Local evidence

Read-only checks found IBus 1.5.34-rc2 running, with current engine `xkb:us::eng`. The installed IBus GI binding exposes process-key-event, focus-in-id/focus-out-id, content-type, reset, preedit update and commit methods. PASSWORD, PIN, TERMINAL, PRIVATE and HIDDEN_TEXT constants are available. No engine was registered, no input source selected, no daemon restarted, and no user text or keyboard events were read.

## What the interfaces establish

| Mechanism | Evidence | Limit |
| --- | --- | --- |
| IBus engine processing | Key handling and text signals use input contexts | Not a global background filter over every existing XKB source |
| Focus/content type | Installed API has focus identifiers and sensitive-purpose/hint support | Availability does not prove every application supplies timely, accurate field metadata |
| GNOME integration | Shell bridges IBus preedit/commit and content hints into its input method | Shell uses a shared IBus context; its identifier alone must not be treated as a unique editable-field identity |
| Delete surrounding + commit | IBus exposes separate messages | No demonstrated compare-and-replace operation with an expected field revision |
| Preedit | An engine can display composition and commit the resulting word | Changes composition behavior; requires lossless focus/reset/error handling |
| Wayland text-input v3 | Related changes are applied together at `done` | Its serial mismatch behavior is not rejection of stale text operations |

The installed-version [IBus context implementation](https://raw.githubusercontent.com/ibus/ibus/1.5.34-rc2/bus/inputcontext.c) routes engine signals through context objects. It also contains optional post-process queues; this alone does not establish transaction semantics for every client.

[GNOME Shell 50.1 inputMethod.js](https://raw.githubusercontent.com/GNOME/gnome-shell/50.1/js/misc/inputMethod.js) creates a `gnome-shell` IBus context, forwards focus/reset and purpose/hint changes, and handles preedit and commit. Focus resets and content updates must be studied as an ordered sequence, not independent permission booleans. Unknown or outdated content state must keep correction suspended.

The [Wayland text-input-v3 specification](https://raw.githubusercontent.com/wayland-mirror/wayland-protocols/main/unstable/text-input/text-input-unstable-v3.xml) groups pending operations under `done`. On a serial mismatch the client still applies text changes; the serial therefore cannot be used as an assumed compare-and-swap safety guarantee. Starting preedit can also replace an existing selection, so selection handling remains necessary.

## Recommended experiment

Use a separate IBus test environment and synthetic clients, with no connection to the desktop input bus, no display access and no registration in the user's sources. Start with a pass-through engine and verify lifecycle/ordering. Then test a bounded current-word preedit model that commits one resulting string instead of deleting committed text. Reuse the core classifier only after the transport and lifecycle tests pass.

This is a candidate design, not an approved change to release behavior. Composition can be visually underlined and interact with shortcuts, cursor movement and application undo. Correcting a draft while it is being composed could meet during-word correction, but switching the actual OS layout must also preserve that draft. The existing companion currently rejects non-XKB sources; it cannot simply be reused unchanged for an IBus engine. Manual repair of already committed text and undo remain separate unsolved requirements.

Prototype gates:

1. Ordered press/release, rapid typing and boundary handling: no duplicate, reordered or lost characters.
2. Focus A to B and A to B to A: no old draft committed into a new target; do not silently drop the draft either. Test both CLEAR and COMMIT focus policies rather than selecting one by assumption.
3. Missing/stale content type, password/PIN/private/hidden fields, composition conflicts and selection: no TypoMorph analysis buffer for ineligible input; preserve ordinary application input.
4. Reset, disable, source switch and engine failure: explicit disposition of the draft, no stale replay or blind rollback.
5. Privacy: no surrounding-document request, text logging, files or network. Audit unsolicited surrounding-text delivery and base-library caching before claiming that no document text reaches the engine. Focus metadata alone is insufficient proof.
6. After isolation passes, coordinate real Text Editor and Chrome tests separately. Determine Chrome's actual native Wayland/XWayland and IME path in that session; none was verified here. No browser-wide disable rule or browser-extension decision follows from this review.

## Recommendation and status

IBus/preedit is a reasonable next feasibility experiment because it can avoid the destructive backspace phase. It is not yet a proven safe backend or a substitute for the complete platform matrix. Keep `require_safe_replacement_backend` refusing automatic run until real adapter guarantees and tests justify replacing it. No additional system installation is needed for the initial isolated prototype: the required IBus executable and GI binding are already present.

## Protocol experiment completed

The [isolated Engine prototype](../integrations/ibus/README.md) passed eight synthetic protocol tests. It uses the real IBus.Engine library but no IBus daemon or application UI. Unchanged ContentType notifications and lossless preedit disposition on focus transitions remain blockers. Next test actual daemon/client routing in isolation; do not activate the fixture on the desktop.

## Private daemon experiment completed

Seven additional tests now exercise the installed IBus daemon on a private temporary socket with synthetic client contexts. Server-side COMMIT policy retained the draft on the original context across focus loss and reset. Client-side COMMIT is still a test model. Unchanged ContentType on normal-to-normal transitions leaves correction suspended; a fresh field-bound eligibility protocol remains necessary. Both suites total 15 passing checks, without testing GTK/Chrome. See the prototype README for exact isolation and limitations.

## Generation-gate experiment

Explicit synthetic field acknowledgements now replace the artificial content-type handshake. 22 tests pass across direct and private-daemon suites. The stale-response state machine is implemented; the trusted real field observer and lossless draft handling during revocation are not. No desktop-ready eligibility claim is made.

## Real metadata binding check

The [field-binding investigation](FIELD_BINDING.md) observed different accessible fields sharing the same current IBus context in the coordinated desktop session. Do not promote the synthetic observer to a production authority using IBus path equality. Field-level lifecycle/event ordering is the next feasibility gap.
