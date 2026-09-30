# Known bugs and investigation record

Date: 2026-09-24.
Discovery recorded user observations and source findings; no live reproduction, fix, or application test was performed during documentation work.

## BUG-001: No automatic layout correction in the owner's Linux session

| Field | Record |
| --- | --- |
| Status | Open; owner-reported, independently unreproduced |
| Expected | Wrong-layout `ghbdtn` becomes `привет`, with the corresponding layout change |
| Actual | Text remains unchanged, including after a space |
| Applications | Chrome and an application called "Text Editor"; owner reports failure in every tested typing location |
| Runtime indication | Tray icon present and showing an active state |
| Frequency | Always in the owner's reported checks |
| Severity | High; blocks the core product function and normal TypoMorph use |
| OS | Ubuntu 26.04.1 LTS, 64-bit |
| Desktop/session | GNOME 50, Wayland |
| Kernel | Linux 7.0.0-31-generic |
| Hardware | Lenovo ThinkCentre M720q, Intel Core i5-8400T, 16 GiB RAM, Intel UHD Graphics 630 |
| Report provenance | Owner's system report generated 2026-09-23; subsequent interview confirmations |
| Missing details | Exact executable/build, launch method/service state, actual source layout ID, keyboard device, editor package/version, Chrome version/backend, permissions |
| Root cause | Unknown |

### Reported reproduction

1. Run TypoMorph; its tray icon indicates active.
2. Focus Chrome or "Text Editor".
3. Enter `ghbdtn`, intending the Russian word `привет`.
4. Press Space.
5. Observe that the text is not corrected.

The owner confirmed that the failure is not limited to waiting for a word boundary. The current source does analyze on a boundary, so the approved future within-word behavior alone does not explain this report.

An active icon is not evidence of working capture, correct layout identity, accessible field context, or successful replacement. Do not pre-assign the cause to Wayland, permissions, classifier confidence, or GNOME switching.

### Later investigation boundary

After an implementation plan is approved, reproduce on controlled synthetic text and record each stage's technical outcome without collecting a personal typing history. Preserve the user's existing environment and distinguish the installed build from source and simulation.

A fix must have a regression check and real-application evidence. Passing a pure conversion test does not close this bug.

## Other records

The owner reports no other known bugs or usability issues.

Missing Windows/macOS adapters are planned implementation scope, not newly discovered runtime bugs.

The diagnostic logging, incomplete keymaps, inactivity-reset TODO, event suppression, legacy license behavior, and latency assumptions found in source are documented in [ARCHITECTURE.md](ARCHITECTURE.md) and [DECISIONS.md](DECISIONS.md). They are not independent proof of BUG-001's cause.

The existing [CHANGELOG.md](../CHANGELOG.md) describes a prior synthetic-echo fix. That historical entry does not establish that all related runtime failures have been resolved.

## Future bug record template

Record expected behavior, actual behavior, exact reproduction, synthetic example, environment/build, severity, frequency, impact on normal use, evidence provenance, and status. Separate user reports, independent reproductions, code findings, and hypotheses.

## P0 follow-up

After implementation-plan approval, technical environment checks, baseline tests, and live-path diagnostic cleanup began. See [P0 investigation](P0_INVESTIGATION.md). BUG-001 remains open pending controlled real-application reproduction; the stopped service and ignored GNOME setting are distinct findings, not a verified complete root cause.

### Controlled P0 dry-run, 2026-09-24

After owner readiness confirmation, the working-copy binary captured input and reached `correction_candidate` in a 30-second, non-mutating Text Editor session. The timeout stopped the process as expected. No switch/replacement was attempted, so BUG-001 remains open; this is partial pipeline evidence, not a successful end-to-end correction. See [P0 investigation](P0_INVESTIGATION.md).

### Switching-backend finding

Read-only GNOME probes confirmed that the deprecated setting is ignored and Shell Eval is rejected in the owner's session. P0 now rejects unsuccessful/unconfirmed replies and checks access before normal capture. Four regression tests cover these boundaries. The normal working-copy daemon therefore refuses this unavailable backend before editing text. A functional safe GNOME integration is still required; BUG-001 is not closed. See [P0 investigation](P0_INVESTIGATION.md).

## DEV-002: Boundary space omitted from replacement span

Source/model finding, not an additional owner-reported reproduction. The old boundary-triggered daemon removed N characters after N letters plus a space had reached the field. In the synthetic `ghbdtn ` example this left `g` and omitted the trailing space. Working-copy replacement planning now deletes N+1 and emits the complete corrected word plus space, or rejects an incomplete mapping. Six regression tests pass; real application replacement remains unverified. See [investigation](P0_INVESTIGATION.md).

## 2026-09-30 real-editor experiment

Baseline and patched Text Editor 50.1 now compile. The isolated patched app
performs fixed `ghbdtn -> привет` and ordinary undo/redo, but post-deletion
competing edits cause partial replacement, including through a GTK callback.
BUG-001 remains open: this is not a working integration for unmodified apps.
See [native results](NATIVE_TEXT_EDITOR_RESULTS.md). The production gate remains.
