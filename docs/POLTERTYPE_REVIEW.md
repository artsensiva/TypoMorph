# PolterType implementation review

Date: 2026-09-27. Source-only review of public repository
https://github.com/Just-Code-NET/PolterType at
`88709a6efd168a4a2f47382b7b224ebd76cb3573` (shallow checkout in a temporary directory).
No PolterType build, setup script, binary or input capture was executed. Its runtime
claims below remain upstream reports, not independent TypoMorph validation.

## Main conclusion

PolterType demonstrates a practical alternative engineering strategy: OS-wide key
capture, physical-layout mapping, local detectors and deletion/retyping, with key
holding and repair mechanisms for overlap. It does not require a patched editor
for that strategy. We should not mistake our chosen owner-transaction experiment
for the only way to build a useful layout switcher.

However, the inspected path does not establish TypoMorph's stronger guarantees:
no protected/unknown collection, no dropped/reordered genuine input, no stale-field
edit and no clipboard fallback. Adopting it wholesale would require changing
approved requirements, not just replacing our unfinished adapter. No such change
has been approved. This review neither proves every global approach impossible
nor certifies the current native-cooperation approach deployable.

## Implementation findings

All links below are pinned to the reviewed revision.

| Area | Source finding | Consequence for TypoMorph |
| --- | --- | --- |
| Linux Wayland emission | [Emitter](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/crates/poltertype-input/src/linux/wayland/emitter.rs) uses uinput backspaces/key replay with pacing | Useful implementation reference; still global event delivery, not an atomic field edit |
| X11 | [Emitter](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/crates/poltertype-input/src/linux/x11/emitter.rs) implements the XTest path; the listener is separate | Avoid conflating X11 and Wayland permissions/capabilities; do not infer either one's safety from the other |
| Windows/macOS | [Windows](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/crates/poltertype-input/src/windows/emitter.rs) uses SendInput; [macOS](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/crates/poltertype-input/src/macos/emitter.rs) uses CGEvent posting and echo timing | Concrete starting points for platform feasibility, not substitutes for field ownership and device tests |
| Correction ordering | [correction.rs](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/crates/poltertype-core/src/engine/switcher/correction.rs) switches layout before deletion, checks switch persistence where available, releases modifiers, holds keys and performs bounded intrusion repairs | Useful adversarial schedules: modifier release, layout reversion, delayed echoes, overlapping typing and remappers |
| Unknown layout verification | In that correction source, switch_held treats unavailable independent verification as success; active-layout query failure can continue | TypoMorph must not treat unavailable safety evidence as affirmative authorization |
| Held input | emit_held_keys can fall back from raw replay to rendered text; unsupported non-text keys are counted/dropped, with Backspace handled separately | Not compatible with our requirement to preserve genuine input semantics; do not reuse fallback as-is |
| Focus | [FocusTracker](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/crates/poltertype-input/src/focus/traits.rs) exposes best-effort process identity and optional geometry/caret hints | This interface is not a field-generation/revision-checked mutation authority; do not infer that from caret data |
| Selection repair | correction.rs copies a selection and pastes replacement, then attempts clipboard restoration | Explicitly excluded by our no-clipboard-fallback decision |
| Protected selections | [Known gaps](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/docs/KNOWN-GAPS.md), selection-conversion section, documents no password distinction there | This finding is specifically about that selection path; it is not a claim that we tested all password behavior |
| Detection | [DictionaryDetector](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/crates/poltertype-detect/src/dictionary_detector.rs) separates current-word acceptance, alternative dictionary evidence and no-opinion fallback; short words have specific handling | Useful for a future measured detector baseline, particularly Latin-to-Latin ambiguity; do not import overlays/learning, data or thresholds without review |
| Key gate recovery | [hold.rs](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/crates/poltertype-input/src/hold.rs) has a deadline checked by the event decision rather than relying only on a watchdog | Adopt the general principle of deadlines checked at use; this does not authorize global grabs in TypoMorph |
| Logging | [logsafe.rs](https://github.com/Just-Code-NET/PolterType/blob/88709a6efd168a4a2f47382b7b224ebd76cb3573/crates/poltertype-types/src/logsafe.rs) redacts word output unless explicitly opted into debug-only disclosure | Useful privacy practice; do not assume all upstream types/log sites were audited |

The README describes word-end automatic correction, whereas TypoMorph requires
within-word correction when confident. Their AI, learning and clipboard features
are not approved additions here. Their published platform list and latest release
must not be imported into our compatibility claims. Known-gap records distinguish
measured environments from untested behavior; retain that discipline.

## Development performed from the review

A focused privacy audit found that TypoMorph Token and RingBuffer derived Debug,
which could expose typed characters if embedded in a diagnostic. Replaced these
with explicit content-free implementations: Token is redacted; RingBuffer exposes
capacity and length only. Normal text-processing access remains unchanged. Both
compact/pretty and nested formatting are covered, including wrapped buffer input.
This is independently written code; no PolterType code/data was copied.

The improvement is within the approved privacy implementation plan and does not
change product scope, platform architecture, settings or live capture. It does
not certify all logging: explicit as_string/ch access, candidate types, legacy CLI
and browser paths still need the release-wide privacy audit.

Validation: all 54 core-engine unit tests and both targeted release-format tests
passed; core clippy with -D warnings passed. Release linking emitted a deprecated
optimization-setting warning; no test failed. No hardware/application test
was run for this change. The production replacement gate stays in place.

## How this changes execution priorities

1. Use this pinned source as a reference for platform APIs and concrete edge-case
   tests rather than repeatedly designing those mechanisms from scratch.
2. Preserve the distinction between key serialization and target authorization.
   The reviewed global replay path does not discharge the current safety blocker.
3. Continue independent core/privacy/layout work while Text Editor build dependencies
   are unavailable; do not spend every development turn extending documentation or
   standalone fixtures. Significant classifier/data changes still need a concrete
   reviewed proposal and direction-specific baseline.
4. Keep the source-only Text Editor experiment and native-cooperation direction as
   approved, but require evidence for its deployability. Neither an endless editor
   fork nor an unsafe replay fallback is silently selected for shipping.
5. Before reusing source, retain the required upstream license/attribution and review
   bundled data separately. The root license observed is MIT; that observation is
   not blanket clearance for all third-party dictionary assets. No reuse performed.

Next engineering work: complete the content-bearing type/log-site inventory and
small privacy fixes under the existing plan, then propose the exact six-language
mapping/detection baseline. Package installation, upstream messages, deployment
and changes to product safety requirements remain outside this approval.
