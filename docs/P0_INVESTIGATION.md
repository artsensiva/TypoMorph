# P0 investigation: Linux correction

Date: 2026-09-24. Implementation plan approved; investigation in progress.
No claim that BUG-001 is fixed.

## Verified technical metadata

- Session: Ubuntu 26.04, GNOME, Wayland.
- Installed package: typomorph 0.2.2-1; executable reports 0.2.2.
- Installed executable and the pre-existing release artifact have identical SHA-256: `d629cfdf2fbab9897cd77ccd6d270538ba48cb2e5b893177b226c1f93cc1f5e5`.
- At inspection, the user service was inactive/dead and no standalone typomorph process was found. This explains why correction cannot happen in that inspected state, but does not explain the owner's earlier active-tray report.
- Chrome: 153.0.8010.47; GNOME Text Editor: 50.1.
- Configured input sources: XKB us and ru.
- All 16 input event nodes passed a read-access check; uinput passed a read/write-access check. These checks did not open an event stream or establish usable keyboard capture.
- The stored input-source `current` value was 1. The installed schema describes this key as deprecated and ignored, so it is not evidence of the active layout.

## Confirmed source mismatch

`GnomeShellSwitcher::switch_to` currently writes the ignored `current` setting and treats successful process exit as a successful layout switch. Its fallback also does not validate the Shell Eval result body. This is a confirmed backend defect/risk, not yet a complete explanation of BUG-001 in a live correction session. No replacement backend has been chosen or deployed.

## First implementation change

The working-copy live daemon no longer prints keys, typed words, device identities, or replacement lengths. Live prompt notifications/hotkey output have been removed. Optional `run --diagnostics` emits only fixed stage labels, disabled by default. A returned OS call is reported as returned, not as verified success.

This does not establish full release privacy compliance: field-safety gating, context/queue bounds, pause behavior, and legacy explicit CLI/browser cloud routes remain outstanding. The installed executable and service configuration have not been changed.

## Validation so far

- Baseline workspace tests: 66 passed; no failures.
- Baseline fmt and Clippy with warnings denied: passed.
- After the diagnostic change: the same 66 tests and Clippy passed; development binary built.
- Device-free synthetic `ghbdtn` simulation: Russian candidate, confidence 0.98, proposed result `привет`.
- Source review found no input-bearing print or notification in the live run path. Explicit stdin simulation still prints its synthetic result.
- A controlled live dry-run was completed after the owner confirmed a blank Text Editor and US layout. Three input devices opened; the process reported `input_received`, `boundary_received`, and `correction_candidate`. The external 30-second timeout stopped it with exit code 124 (expected). No typed content was printed and no layout switch or text replacement was attempted.

## Controlled live protocol

First run only the working-copy binary in `--dry-run --layout us --diagnostics`, under a 30-second external timeout. The owner prepares a blank Text Editor and selects US, waits for the start signal, then types `ghbdtn` plus Space once. No text replacement or OS-layout mutation occurs in this run. Only technical stage labels leave the process.

Stop the process before resuming ordinary typing. Distinguish capture/classification evidence from switching/replacement evidence. A dry-run cannot close BUG-001.

## Live dry-run result

The working-copy capture and classification path reached a correction candidate during the owner-coordinated synthetic test. This narrows the current investigation: capture/classification worked in this controlled session, but it does not reproduce or resolve the installed build's complete failure. The stage labels deliberately do not expose the source or replacement text, so the live output alone cannot verify the proposed replacement characters.

Next investigate a verifiable layout-switch backend and safe replacement before a mutation-enabled test. The existing ignored GSettings key is not a valid success signal. The test process has exited; the installed service remains unchanged.

## Backend verification and fail-closed correction

A read-only Shell Eval call with `1 + 1` returned `(false, '')`. The installed GNOME Shell 50.1 resource `/org/gnome/shell/ui/shellDBus.js` explicitly returns that rejection unless its unsafe mode is enabled. No protection setting was changed. The installed input-source manager resides in `ui/status/keyboard.js`, not the old `global.display` call path.

Working-copy changes:
- Removed writes to the ignored GSettings `current` key and fixed US/RU index assumptions.
- Validate both the Eval success flag and exact target identity; false, absent, or mismatched confirmation cancels replacement.
- Validate layout identifiers before constructing the expression.
- Check backend availability before opening keyboard capture or uinput in normal `run` mode. The explicit dry-run still bypasses switching and captures input only when deliberately invoked.
- Added `check-layout-backend`, a non-capturing, non-mutating diagnostic command.

The new command returned the expected exit status 1 on the owner's session, with a fixed unsupported-backend error and no capture startup. Full workspace tests: 70 passed; Clippy with warnings denied and build passed. Four new regression tests cover rejected replies, wrong target confirmation, preflight denial, and expression-injection inputs.

This deliberately stops the normal working-copy daemon early on the current GNOME configuration instead of claiming a working switch. No replacement or active-layout mutation was tested. The installed application and user service remain untouched. BUG-001 is still open.

### Next integration work

Evaluate a narrowly scoped GNOME integration exposing current source identity and acknowledged source activation, without text access or arbitrary script execution. A GNOME Shell companion is a candidate, not a browser extension or an installed component at this stage. Confirm supported-version behavior, session locking, lifecycle and permission boundaries before installation or mutation-enabled validation. Do not enable unsafe mode, synthesize blind layout-toggle shortcuts, or infer active layout from the ignored setting.

## P1 local GNOME Layout Bridge prototype

Implemented a narrow Shell companion and typed Rust client under `integrations/gnome/` and `crates/platform-linux/src/gnome.rs`. The Eval fallback is now removed entirely. The client requires protocol version 1, eligible XKB state, an exact expected source on switch, explicit target confirmation and a state re-read. Each operation has a 250 ms failure deadline. A timeout is not a rollback guarantee.

The normal daemon now initializes from the reported current US/RU source; unsupported initial sources fail before capture. The companion refuses locked/greeter/IME/unknown states and stopped instances. It receives layout identifiers only, never text or key events. This does not implement field safety or a complete replacement transaction.

Evidence:
- 70 ordinary Rust workspace tests passed, including four bridge validation/deadline tests replacing the previous Eval tests.
- Eight Node service-policy tests passed.
- A separate Rust/GJS protocol test passed on a private D-Bus session using the actual exported service wrapper with synthetic source state. It is intentionally ignored in ordinary workspace runs and executed through the private-bus runner.
- Clippy, formatting, and JavaScript syntax checks passed.
- The ZIP package was built and all three entries compared byte-for-byte to their sources.
- A real-session non-capturing probe rejected the absent bridge as expected, without opening input devices.

Package: `target/gnome/layout-bridge@typomorph.com.shell-extension.zip`.
SHA-256 of this build: `a1e3f4a3023a67dd4a1c18f359d0a5868fa6a34a65a1a550e75d02522a592c29`.

No companion has been installed/enabled, no GNOME setting changed, and no live switching or text replacement was performed in this step. The module advertises only GNOME 50 pending real-session/version testing. See [bridge documentation](../integrations/gnome/README.md) for the protocol, limits, test runner, package, and controlled installation procedure. Next: controlled user-level installation and a read-only backend check, followed by separately coordinated switching validation.

## User-approved companion installation

The owner explicitly authorized installing the module and checking the connection without input capture or layout/text mutation. The existing user/system extension paths were absent. The verified package was installed in the user's GNOME extension directory, and all three installed files matched the package sources.

The current GNOME session does not know the new UUID, so `gnome-extensions enable` returned extension-not-found. Inspection of the installed Shell code confirmed that `ReloadExtension` is deprecated/not supported and that enabling requires an already discovered extension. The companion's UUID was added to `enabled-extensions` for the next login, with unrelated entries preserved; it is not explicitly disabled and user extensions are globally allowed.

The non-capturing backend check still reports the bridge unavailable in this old session. No success is claimed yet. A user-controlled logout/login is required before the next read-only connection check. No logout/restart, unsafe mode, input capture, layout switch, text replacement, or installed-daemon update was performed.

## Connection verified after graphical login

After the owner selected the next read-only check, GNOME reported `layout-bridge@typomorph.com` enabled and ACTIVE. The working-copy `check-layout-backend` command succeeded. A direct `GetState` call returned protocol 1, ready=true, XKB source `ru`, and installed sources `us` and `ru`.

This verifies discovery, module activation, session-bus connectivity, and source-state reporting in the real GNOME session. It does not verify layout mutation or text replacement. No input capture, source switch, text access, or installed-daemon update was performed. BUG-001 remains open. Next validation: a coordinated source switch and restoration without capturing or replacing text.

## Real-session source switch and restoration verified

The owner explicitly selected a source-switch-only test. Initial state was ready XKB `ru`. `Switch(ru, us)` returned `(true, xkb, us)` and a separate `GetState` confirmed `us`. `Switch(us, ru)` returned `(true, xkb, ru)` and a separate final read confirmed restoration to `ru`.

No keyboard capture, field text access, injected key events, or text replacement was performed. This establishes real-session GNOME source switching and restoration through the companion; it does not establish visible replacement correctness, input integrity, or protected-field safety. BUG-001 remains open until controlled end-to-end correction succeeds. Next: validate the replacement operation, including the already-typed delimiter and target/context safety, before running an editing test.

## Word-plus-space replacement planning

Source inspection confirmed an off-by-one deletion: the boundary space is already delivered to the application, but the old emitter removed only the word's key count and did not append the space. For the controlled plain-text model, `ghbdtn ` would become `gпривет` rather than `привет `.

The working copy now builds a complete validated replacement plan before attempting a layout switch: delete the word plus one space and insert every mapped replacement character plus a space. It rejects unsupported layouts/delimiters, unmappable or lossy case conversion, differing source/replacement lengths, and truncated source buffers. A bounded counter replaces the unbounded scan-code vector; once a word exceeds the ring capacity, the count remains a rejection sentinel until the next boundary.

Six new tests cover the forward/reverse correction, prefix/suffix preservation in a synthetic field, every word length from 1 through 32, truncated/mismatched buffers, unsupported characters/case, layouts, and delimiters. These test the shared plan used by the daemon; they do not inject keys into an application.

The test model assumes the complete source word and delimiter are immediately before the cursor, with no concurrent typing or editing. Real field identification, cursor/context revalidation, modifier handling, protected-field suppression and input races are still outstanding. Therefore no end-to-end replacement success or general input-integrity guarantee is claimed. Next implement field/cursor eligibility checks before a live editing test. The installed daemon and GNOME module were not changed by this step.

## Metadata context guard implementation

Added bounded AT-SPI metadata discovery and core context continuity checks. Normal and dry-run capture now require initial eligible field metadata. Input-word accumulation is rejected/reset on observed identity/position/count changes, unknown/selected/protected fields, repeats, modifiers, or changed/unavailable GNOME source. Pre-switch and pre-emission snapshots must match. A new `check-input-context` command reports eligibility without text reads or key capture.

Validation: 82 ordinary workspace tests passed, including six new context tests; Clippy and build passed. The dedicated private-bus AT-SPI protocol test passed, including protection appearing during a snapshot. No live application field was queried during this implementation step; only library constants and synthetic services were examined. Existing installed binaries and the GNOME companion were not changed.

These are snapshot guards, not an atomic edit transaction. See [context guard limits and protocol references](CONTEXT_GUARD.md). Next: a read-only metadata check on a blank Text Editor before any live replacement test. BUG-001 remains open.

## Read-only real-session metadata diagnosis (2026-09-24)

The coordinated blank-editor metadata check failed without opening keyboard capture or reading text. Added fixed rejection codes and reproduced `object_role_unavailable`. A bounded metadata-only investigation found `GetRole` returning `UnknownMethod` on the AT-SPI null-object sentinel at depth 2. No application names, window titles, field contents, or object identifiers were printed.

The reader now ignores explicit null-object references, which denote no accessible object. All other inaccessible objects still reject discovery; eligibility requirements and deadlines are unchanged. The private-bus fixture includes repeated null children in valid and protected scenarios, plus an unavailable non-null child that must remain rejected. The protocol regression test, debug build, Clippy, and whitespace checks passed. The installed daemon and companion remain unchanged. This fixes one discovery blocker; it does not establish end-to-end correction or close BUG-001.

After rebuilding, the coordinated probe advanced past the null-reference failure and returned `focused_object_ineligible`. This is still a refusal, not a successful Text Editor compatibility test. No application identity or individual role/state values were recorded, so the evidence cannot distinguish unexpected focus from an incompatible role/state representation. Next: a coordinated metadata-only role/state diagnostic in the blank editor; do not loosen eligibility checks or proceed to live replacement based on this result.

## Owner-restarted role/focus check (2026-09-25)

At the owner's request, restarted the 15-second preparation interval for a blank Text Editor. The bounded role/state-only diagnostic observed zero focused objects in the traversed tree; inactive window subtrees were excluded, as in production. The immediately following production probe returned `no_eligible_focus` (exit 1). No text, accessible names, window titles, key events or clipboard contents were queried, and no input/layout mutation was performed.

This does not establish that Text Editor lacked keyboard focus: inactive-window filtering or missing accessibility state can also prevent discovery. Next isolate window ACTIVE reporting and descendant FOCUSED reporting with a metadata-only diagnostic; do not weaken production eligibility on this evidence.

## Window and field state isolation (2026-09-25)

The owner selected separate window/focus metadata diagnosis. An expanded role/state-only traversal (including inactive windows) reported two ACTIVE frames and one FOCUSED but non-ACTIVE window. It reached its 512-node limit, with no query errors, so it is incomplete and cannot identify the editor or establish global focus uniqueness. No application/window names or text-interface data were requested by that diagnostic.

A subsequent coordinated traversal using the production window filter observed one focused TEXT object (role 61) under an ACTIVE window. EDITABLE, SENSITIVE, SHOWING and VISIBLE were true; ENABLED was false. The immediately following production check returned `focused_object_ineligible`. Thus the observed state combination fails the existing mandatory ENABLED check. Application identity was deliberately not collected, so attribution to Text Editor relies on the owner-controlled focus setup rather than verified identity.

No eligibility rule was changed. Next verify GTK/AT-SPI ENABLED versus SENSITIVE semantics and select a justified compatibility rule with regression coverage before attempting live correction. Input capture and text mutation remain untested and BUG-001 remains open.

## GTK 4 ENABLED compatibility fix (2026-09-25)

Verified the installed GTK library is 4.22.4 without accessing application fields. Its upstream collect_states implementation sets SENSITIVE for non-disabled widgets but omits ENABLED. Added a narrow compatibility path: absent ENABLED is allowed only for same-provider GetApplication metadata identifying GTK with a numeric 4.x.y Version. Other/missing/malformed toolkit metadata remains rejected. Toolkit identity is rechecked during field revalidation, within the existing timeout; it is not treated as a security identity. Explicit READ_ONLY state now overrides EDITABLE.

Validation: all 10 platform unit tests passed; the private-bus regression scenario passed for standard fields and GTK-without-ENABLED, and rejected disabled/read-only/password fields, unsupported/malformed/missing toolkit data and sensitivity loss during revalidation. Workspace Clippy, debug build and whitespace checks passed. Only synthetic services were queried during implementation; no live field test, keyboard capture, text change, installed-binary replacement or commit was performed. Next: owner-coordinated read-only eligibility check on the blank Text Editor. BUG-001 remains open. See CONTEXT_GUARD.md for pinned source references.

## Successful metadata eligibility check after GTK compatibility fix (2026-09-25)

The owner selected a read-only check of the corrected debug build and was instructed to focus a blank Text Editor. After a 15-second preparation interval, `target/debug/typomorph check-input-context` exited 0 and reported eligible field metadata. No text content, key events or clipboard data were read; no layout switch or text mutation was performed. Application identity was not logged; attribution to Text Editor depends on the coordinated focus setup.

This verifies a successful real-session metadata snapshot after the GTK compatibility change, not end-to-end correction, latency, atomic replacement, or complete protected-field safety. BUG-001 remains open. Next recommended step: controlled read-only checks of selection refusal and context invalidation before any live replacement test. Installed binaries remain unchanged.

## Selection and focus validation preparation (2026-09-25)

The four core context tests and the private-bus metadata regression test passed. Added a manually invoked `platform-linux` example, `check_context_transition`, that waits 15 seconds for an eligible baseline, then compares a second metadata snapshot after 25 seconds using the production ContextGuard. It opens no keyboard devices and does not read or modify text. Build and platform all-target Clippy passed. The live focus-transition example has not yet been run.

For the selection test, the owner was instructed to type and select only `test` in Text Editor. After 20 seconds the read-only production probe exited 0 (eligible), so refusal was NOT demonstrated. Whether selection was actually active at sampling time is unconfirmed; owner clarification was requested. This result is neither a passed selection-safety test nor proof of a reader bug. No text content or key events were collected. Do not proceed to live replacement until this uncertainty is resolved.

The owner clarified that the initial selection had not been prepared in time. Repeated with a 30-second preparation interval: the production probe exited 1 with `selection_present`. This establishes selection refusal for the coordinated synthetic editor case.

The metadata-only focus example was adjusted to 30 seconds before baseline and 30 seconds between snapshots. An eligible baseline was captured; the owner was then instructed to switch windows. The second snapshot matched the first, and the example exited 2 (`context_unchanged; transition rejection not demonstrated`). Whether the owner completed the switch in time is pending clarification. Do not count this as a passed focus-transition test or proceed to live editing.

The owner confirmed insufficient time to switch during the first focus attempt. Repeated with 45 seconds before baseline and 120 seconds between snapshots. An eligible baseline was obtained; the owner was instructed to switch to another window. The example exited 0 with `context_changed_or_unavailable; original replacement refused`. This confirms that ContextGuard refused the original snapshot in the coordinated transition scenario. The fixed outcome intentionally does not distinguish an eligible different field from unavailable/ineligible metadata. No key capture, content read, source switch or edit occurred.

The controlled selection-refusal and transition-refusal cases are now demonstrated. They do not establish detection of focus-out-and-back between snapshots, idle invalidation, or atomicity during emission. Next recommended implementation work is event-driven context invalidation and the replacement/input-delivery lifecycle before a controlled live editing test. The diagnostic example's formatting, Clippy and build passed.

## Sequential emission race and safe refusal (2026-09-25)

Source review found that suppress_delivery only hides physical events from analysis while physical events continue reaching applications. The sequential uinput deletion/insertion has no target binding or input serialization; focus/new-input races remain even with repeated snapshots. Disabled ordinary run before setup side effects and removed the daemon's emission/suppression/layout-mutation candidate path. The library replace_text entry point also refuses. Controlled dry-run now stays on its actual configured source instead of adopting a hypothetical target after candidate analysis.

83 ordinary workspace tests passed, including a new CLI early-refusal regression; two private-bus integration cases remain intentionally ignored by the ordinary suite and their implementations were unchanged in this step. Clippy, debug build and whitespace checks passed. No installed binaries, settings or live field contents were changed; no capture/emission was started. This is containment, not completed race-safe correction: event-driven invalidation and a target-bound serialized editing adapter remain to implement. BUG-001 stays open. See REPLACEMENT_SAFETY.md for the required contract and next feasibility work.

## IBus feasibility review (2026-09-25)

Read-only runtime metadata confirmed running IBus 1.5.34-rc2 and current `xkb:us::eng`; local GI exposes key processing, focus IDs, content type, reset, preedit and commit APIs plus sensitive-field constants. Reviewed upstream IBus, GNOME Shell 50.1 and Wayland text-input-v3 sources. Context routing is promising but separate delete/commit messages and Wayland serials do not prove stale-edit rejection. GNOME's shared IBus context is not a unique field identity.

Recommended next: an isolated pass-through/preedit prototype on synthetic clients, with no desktop registration. Preedit may avoid deleting committed words, but focus/reset draft preservation, protected-field data delivery, actual layout switching, manual correction and undo still need validation. No engine installation/activation, settings change, input capture or content access occurred. See IBUS_FEASIBILITY.md for sources, capability limits and test gates. Automatic run remains disabled and BUG-001 open.

## Isolated IBus Engine protocol prototype (2026-09-25)

Created integrations/ibus with a real GI IBus.Engine object, a synthetic D-Bus client and a dbus-run-session runner. The runner removes display discovery, isolates XDG paths and blocks desktop IBus address discovery; no IBus daemon or component is registered. All input is synthetic. Eight protocol tests passed, covering the fixed correction fixture, sequential input, long words, Backspace/shortcuts, protected/unknown types, focus/reset/disable and capability changes. Whitespace checks passed. No desktop source, installed binary or running input service was changed.

Discovered and tested that unchanged Engine.ContentType property updates do not notify the engine. Positive synthetic tests use an explicit changed-value handshake; real client propagation remains unresolved. Clearing an old draft prevents cross-focus replay, but lossless client-side disposition is not established. This prototype is not a safe live adapter, does not use the Rust classifier, and does not close BUG-001. Next isolated work: real private IBus daemon routing and draft/content-type lifecycle. See integrations/ibus/README.md for scope and limitations.

## Real private IBus daemon and synthetic contexts (2026-09-25)

Extended the fixture with run_daemon_tests.py, private_factory.py and test_daemon.py. A real installed IBus daemon runs on a unique temporary socket inside a private session bus with no display/XIM/panel/config. The fixture is registered only on that daemon. Its automatically activated IBus portal also belongs to the isolated bus. Test lifecycle cleanup leaves the desktop daemon/source untouched.

Seven routing tests passed. Server-side COMMIT mode preserves a draft on the old context through focus transitions and reset; a separate client-side policy model verifies no duplicate server commit. Password routing and cross-context draft separation pass. Normal-to-normal focus without a changed ContentType remains suspended, preserving pass-through input: this confirms the remaining eligibility-notification blocker rather than solving it. The original eight direct protocol tests also passed after making engine object paths configurable.

No GTK/Chrome application, real key stream or personal text participated. Client-side draft handling is still modeled; source switching, selection and field-generation eligibility are not validated. No production classifier integration or live safety-gate change occurred. Next: establish a fresh field-bound eligibility acknowledgement without relying on cached FREE_FORM or the synthetic test handshake.

## Field-generation eligibility gate in isolated IBus prototype (2026-09-25)

Added a content-free Eligibility gate and a private-test-only observer endpoint. Explicit Begin/Confirm binds permission to context, field and generation; confirmations are one-shot. Focus transitions, reset, disable, capability loss, revocation and input before confirmation invalidate pending permission. Cached IBus normal-purpose metadata cannot grant access; protected metadata remains a veto. Removed the artificial password-to-normal content handshake from both positive test suites.

13 direct protocol checks and 9 private-daemon checks passed (22 total), including late replies, A/B/A, distinct fields under one context, duplicated/wrong-field confirmation, reset/disable/revoke and selected/read-only/composing observations. Fresh synthetic acknowledgement resumes correction between normal contexts without a content-type notification.

This implements the protocol/state machine, not the real observation authority. The observer is a fixture, not an authenticated desktop endpoint. Real field-to-IBus binding, ordered context events, observer failure and lossless draft resolution during arbitrary revocation remain required. No working-session registration, capture, installed-binary change or safety-gate opening occurred. Next: real metadata observer feasibility/read-only binding.

## Real read-only field / IBus identity comparison (2026-09-25)

Added probe_field_binding.py, querying only IBus current-context metadata and accessibility object roles/states/references. With 45 seconds for a Text Editor baseline and 120 seconds to move to Ctrl+F, it reported changed accessible field and unchanged IBus context. IDs were kept in memory, not printed; no text interfaces, capture or mutation were used. Application attribution depends on the owner-controlled setup.

IBus context identity alone is therefore insufficient for field eligibility in this measured session. No authoritative cross-protocol binding has been established; the test observer remains synthetic. Source review and evidence are recorded in FIELD_BINDING.md. Next investigate field-level lifecycle events and ordering without content-bearing subscriptions. Automatic run remains gated.

## Live narrow focus notification retry (2026-09-25)

The 90-second-preparation probe completed successfully: one focus-gained and one
focus-lost notification, a different accessible target, no unexpected payload.
Only the baseline GTK provider's focused-state notifications were subscribed to;
no text/key capture or installed changes. This proves notification delivery for
the coordinated session, not input serialization or new-field eligibility.
See FIELD_BINDING.md. Next: isolated delayed-notification/input race checks and
a precise ordering contract; automatic run remains gated and BUG-001 stays open.

## Isolated delayed-notification schedules (2026-09-25)

Added three known-gap characterization tests to test_protocol.py. Withholding
field notifications while retaining one IBus context permits old eligibility to
consume subsequent input and emit a synthetic correction. An unreported A/B/A
round trip cannot be distinguished from continuous focus. Revoke clears the
engine's unfinished draft without sending a commit or preedit disposition update.
The tests do not model real application routing or prove actual text loss.

Validation: 16 direct protocol checks and 9 private-daemon checks passed. Three
passing checks explicitly reproduce unsafe schedules; these are evidence of a
blocker, not safety regression guarantees. No engine behavior, desktop input,
installed component or production gate was changed.

Required contract: field changes must invalidate permission before input for the
new target is processed; grants and input need the same authoritative generation;
old preedit must be resolved exactly once on its original target before ownership
changes. An asynchronous AT-SPI observer plus CurrentInputContext polling does not
establish this contract. Next: compare concrete integration options against these
requirements and obtain owner approval before an architectural change.

Documentation discrepancy: CHANGELOG.md, [0.2.2] Unreleased / Fixed, describes
global input suppression during replacement as the active fix. Current working-copy
automatic run is gated and its daemon replacement path removed; global suppression
also does not establish the approved genuine-input-preservation contract. Treat
that entry as stale, not evidence of a safe current implementation. Release-note
reconciliation remains pending; no historical text was translated or erased.


### Approved native GTK experiment — 2026-09-25

Implemented an isolated Broadway/private-IBus real-widget harness. Eleven sync
checks passed; forced async mode produced nine passes and two retained failing
safety assertions. A delayed echo went to the new field, and reset left the draft
as preedit. See GTK_IBUS_EXPERIMENT.md for route and stimulus limitations. No
production safety gate, installed component or release scope changed. Next trace
client/daemon commit ownership before proposing any fix.


### Delayed-commit routing trace — 2026-09-25

Metadata-only private-fixture instrumentation shows stale engine focus while the
daemon target has changed before emission. Source review identifies unbound
CommitText routing through the currently attached context. Sync suite 11/11;
forced async 9/11 with original failures retained. No atomic engine-only remedy
established. See GTK_IBUS_EXPERIMENT.md for source/version limits and prevention
assessment. Next inspect GNOME/Mutter commit targeting before a design proposal.


### GNOME native-route integration proposal — 2026-09-25

Reviewed Mutter IM event construction/text-input routing and GTK Wayland commit
application. These paths do not establish field-bound completion for an external
engine; existing reset/flush behavior prevents inferring a native-route failure
from Broadway alone. GNOME_INTEGRATION_PROPOSAL.md specifies the candidate owner-
validated transaction contract and a bounded headless native-route reproducer.
Implementation approval is pending. No production/system component changed.


### Native GNOME Wayland results — 2026-09-26

The owner-approved isolated headless reproducer verifies the GTK Wayland route
and sends input only through a PID-verified private compositor service. The first
combined run had 3 passes/6 failures; fresh-per-case runs resolved this to 4 passes
and 5 failures. Wrong-target draft and delayed-key delivery reproduce on this
native test route. Protected fixture refusal works, but old draft reaches the
protected widget; reset retains preedit. See NATIVE_WAYLAND_EXPERIMENT.md for
full scope and test-assumption corrections. Next propose original-target ownership
handling; production gate and installed application remain unchanged.
