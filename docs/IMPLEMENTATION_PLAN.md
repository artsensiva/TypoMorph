# Implementation plan

Status: approved by the owner; P0 implementation is in progress.
Prepared: 2026-09-24.
Baseline: approved documentation committed as `e42e77c`.

## 1. Objective and authorization

Deliver the complete first public release defined in [SPEC.md](SPEC.md), with the language/platform matrix in [COMPATIBILITY.md](COMPATIBILITY.md) and commerce rules in [LICENSING.md](LICENSING.md).

The owner approved the documentation, its local commit, and this implementation plan. Local implementation and relevant controlled validation within this scope are authorized. It does not itself authorize public deployment, production payments, store publication, paid-service purchases, or changes to the commercial/legal offer. Git pushes and additional commits follow the owner's instructions.

No application code or configuration was changed while preparing this plan. The unrelated untracked `.htaccess` is excluded from this work.

## 2. Delivery strategy and dependencies

Start with a small working Linux correction path and evidence of safety feasibility on the other target platforms. An internal Linux milestone is not a reduced public-release scope.

Dependency order:

1. P0 establishes reproducibility and controlled diagnostics.
2. P1 validates platform capabilities and informs P2's shared contracts.
3. P2 establishes safety and state handling; P3 delivers the first complete Linux correction/undo path.
4. P4 extends exact layout coverage; P5 implements and validates remaining adapters.
5. P6 integrates product controls; P7 implements account/payment/offline access.
6. P8 prepares trusted delivery; P9 performs complete release validation.

Synthetic corpora, translations, and test-case preparation can proceed while a particular device is unavailable. No untested OS is marked supported. Do not lock in a GUI framework, adapter API, service host, or token format before the relevant feasibility/design result.

Maintain small reviewable changes. Each completed package reports changed behavior, checks run, failures/limitations, and the next dependency. Update evidence records and open questions rather than re-interviewing the owner about settled requirements.

## 3. Work packages

### P0 — Reproducible Linux diagnosis and development privacy

Primary locations: `crates/daemon/src/main.rs`, `crates/platform-linux/src/lib.rs`, existing integration tests; evidence in `docs/BUGS.md`.

Actions after approval:
- Identify the running/installed binary, source revision, launch method, service state, actual session, selected OS layout IDs, app versions, and device access. Read only necessary technical metadata; do not dump environment secrets or existing journals containing typed text.
- Record initial automated-check results, separating existing failures from new regressions.
- Before live testing, remove input-bearing output from the normal execution path and provide content-free stage outcomes. Prefer no raw-text capture; any necessary development-only text diagnostics must be explicitly enabled, bounded, local, synthetic, and absent from release behavior.
- Reproduce `ghbdtn` plus Space in an isolated synthetic editor/browser field, tracing capture, interpretation, decision, switch, and replacement outcomes.
- Distinguish the installed build from the development binary. Do not replace or restart the user's active installation silently.
- Add a meaningful regression for the identified cause; apply a narrow fix only when the cause is established.

Checks: device-free simulation, focused regression, release-output inspection, then controlled real-application reproduction. Do not enable live global capture while the owner is typing personal content.

Exit: the failing stage and build are known, or the exact reproducibility blocker is documented. Close BUG-001 only after real-application evidence confirms the fix. A unit test alone cannot close it. If the cause depends on missing platform capabilities, carry the bug into P1–P3 instead of adding an unsafe workaround.

### P1 — Platform feasibility and concrete compatibility baseline

Locations: existing Linux adapter; disposable or test-only probes under the eventual platform test structure; `docs/COMPATIBILITY.md`, `docs/ARCHITECTURE.md`, `docs/OPEN_QUESTIONS.md`.

For Ubuntu GNOME Wayland/X11, Windows 11, and both Mac architectures:
- Confirm actual test hardware/OS versions and available sessions.
- Test ordinary-user permission flows, selected layout identity/change notification, focused field identity, protected/unknown field handling, cursor/selection access, composition state, safe replacement, and continued typing.
- Test Chrome/Firefox and the platform-specific required browsers with synthetic plain text, rich text, password, and inaccessible fields.
- Compare observed text before/after replacement and focus changes; do not infer safety from raw key events.
- Consult current official platform documentation when selecting APIs; record permission/deployment implications and evidence.
- Produce a capability table and a short architecture decision record for each viable adapter.

Exit: a concrete initial OS/app/layout matrix and demonstrated capability limits. If a required target cannot meet safety with ordinary-user privileges, present the conflict and recommended options to the owner before changing scope. Decide whether browser extensions are needed from measured gaps; extension inclusion remains a separate product decision.

### P2 — Shared safety policy and input-state lifecycle

Locations: `core-engine`, `daemon`, platform adapters; introduce a small shared contract module/crate only if the dependency boundary requires it.

Implement:
- Explicit input eligibility: valid access, pause, application exclusion, selected layout, field safety, and IME/composition state.
- Context identities/generations so delayed candidates cannot mutate a newly focused field.
- Separate bounded analysis and undo state; clear both where safety requires it.
- Reset/invalidation for focus, cursor, selection, paste, inactivity, and manual layout changes.
- Genuine vs injected input handling, ordered event processing, and failure/recovery without replaying stale queues.
- Persistent manual pause semantics and content-free suspension reasons.
- Remove the legacy paid-only safety boundary and exclude legacy prompt/AI actions from the release path, including any retained browser-host build.

Before implementation, record concrete buffer bounds, inactivity/undo timing, boundary rules, and retry policy with rationale and tests. These are technical parameters subject to adjustment from evidence; do not invent quality claims.

Checks: state-transition tests with fake adapters; protected/unknown/IME/pause cases; focus races; queue bounds; injected-event behavior; no content in logs or errors.

Exit: shared safety rules are independently testable and a missing context signal causes suspension. No entitlement or app override can bypass field safety.

### P3 — Complete Linux correction, manual repair, and undo

Locations: `core-engine/src/layout.rs`, `daemon`, `platform-linux`; focused engine and platform integration tests.

Implement:
- Evaluate evolving words; correct only when a wrong-layout alternative is sufficiently convincing.
- Preserve correctly typed text even when the language differs from the layout label.
- Revalidate context and input state immediately before replacement; skip/defer on uncertainty.
- Preserve genuine input during replacement without arbitrary key loss or reordering.
- Manual last-word repair with layout change; selection repair preserving layout; ambiguous preview choices.
- Direct selection access only; no clipboard fallback.
- Safe undo restoring text/layout and preserving subsequent input; refuse stale undo and suppress re-correction of the canceled current word.

Checks: synthetic end-to-end cases in Chrome and the identified Text Editor; fast continuing typing, rapid focus changes, selection/cursor moves, repeated keys, permission loss, and undo after additional characters.

Exit: BUG-001 is resolved on the confirmed environment, or remains explicitly blocked by a documented capability limitation. The first complete path passes safety/integrity checks. Do not optimize away required revalidation to reach a latency target.

### P4 — Six-language exact layout support and quality baseline

Locations: `core-engine` mapping/scoring data and tests; layout discovery in adapters; compatibility/benchmark evidence.

Implement and verify the approved standard US/UK, RU, UA, Germany QWERTZ, France traditional AZERTY, Spain, and Latin American families with exact OS identifiers. Validate native Apple/PC variants where required and actually present.

Separate language scoring from physical/text mapping. Handle casing, punctuation, Shift, AltGr/Option, dead keys, and command chords. Only confirmed selected layouts participate; newly installed layouts are offered, never silently enrolled.

Create synthetic and appropriately licensed test corpora with correct-input controls and ambiguous cases. Cover each supported direction separately, including Latin-to-Latin ambiguity. Keep evaluation samples separate from tuning samples where feasible.

Checks: mapping fixtures, candidate ranking/abstention, new/unselected layout changes, per-direction false corrections and misses. Define sample sizes and propose numerical acceptance thresholds after baseline measurement, before beta.

Exit: exact variant coverage recorded, no language advertised from a profile alone, and thresholds/data limitations reviewed. Additional languages remain deferred.

### P5 — Windows and macOS implementations

Locations: proposed `crates/platform-windows` and `crates/platform-macos`, selected shared contracts, platform entry points, Cargo target configuration, CI.

Create adapters based on P1 evidence. Keep core decisions reusable while respecting each OS's permissions, layout identity, text access, composition, tray/menu integration, and lifecycle.

Port the P2–P4 behavior, not merely raw key capture. Build and test on Windows 11, macOS Intel, and macOS Apple Silicon. Validate Chrome/Firefox everywhere, Edge on Windows, and native Safari on macOS.

Checks: native builds plus real-app safety, replacement, undo, composition, and restart/permission tests. A hosted CI build does not substitute for real desktop testing.

Exit: each required environment has recorded evidence. If hardware is unavailable, continue independent work and mark those results untested; do not silently remove a platform.

### P6 — Settings, onboarding, and localized controls

Locations: existing tray and application entry points; proposed UI/settings/localization modules selected after toolkit feasibility.

Implement:
- Permissions/readiness onboarding and confirmed installed-layout selection.
- Settings for shortcuts, exclusions, UI language, sounds, account/access, and update checking.
- Autostart only after consent; persistent pause across restarts.
- Two distinct optional sound signals, off by default, with a global toggle.
- Non-content state/error reasons and manual candidate previews.
- English, Russian, Ukrainian, German, French, and Spanish UI; OS-language fallback and independent manual choice.

Checks: first run, restart, denied/revoked permissions, shortcut conflicts, locale switching, pause without collection, one sound per action, and settings containing no input history.

Exit: ordinary users can configure and understand the product without a CLI or persistent elevation. Translation review is recorded; technical docs remain English.

### P7 — Account, Stripe, and authenticated offline access

Locations: `crates/licensing`, application access gates, proposed minimal account/entitlement service and test fixtures. Select hosting/storage/email and cryptographic format in a design record before building the service.

Implement against test transports and Stripe test mode:
- Verified-email passwordless account; one shared seven-day trial and three random installation IDs.
- USD 7 annual / USD 19 perpetual, same features and updates, tax-inclusive product totals.
- Provider-hosted checkout, authenticated payment notifications, duplicate/out-of-order event handling, and server-issued offline entitlements.
- Annual paid-through access, indefinite perpetual offline access, cancellation, confirmed failed-renewal grace, self-service device release, and first-payment refunds.
- Full-price perpetual conversion that stops future annual renewal after success.
- No keyboard data in accounts, billing, reports, or service metadata; signing secrets never ship to clients.
- Remove obsolete Lemon Squeezy and Free/Pro behavior from the release path.

Resolve reminder timing, later-perpetual refund wording, metadata retention, clock/recovery handling, and offline grace delivery before the affected implementation/publication. Review the actual German seller/tax/customer terms separately; do not infer them from self-employed status.

Checks: mocked unit/integration tests followed by test-mode lifecycle tests, signature tampering, outages, clock changes, expired/canceled access, account-wide dates, transfers, refund and conversion races.

Exit: commercial lifecycle and offline semantics match the specification. Immediate revocation of an offline perpetual copy is explicitly not promised. Production account configuration and live payment collection require a concrete separate release action.

### P8 — Trusted distribution, updates, and public materials

Locations: `scripts/`, `.github/workflows/`, platform packaging, updater, `landing/`, release instructions.

Implement Debian, signed Windows, and signed/notarized macOS packaging with consistent versions/artifact identity. Define signing-key custody, verified update metadata, opt-out checks, consent to install, and failure behavior.

Remove unreachable/out-of-scope features from shipped artifacts and verify packaging does not enable text logging. Update website prices, platform claims, terms/privacy links, support route, and approximate dated EUR display with stale-rate fallback.

Checks: clean install/update/uninstall on targets; valid/tampered/missing authenticity metadata; interrupted downloads; rejected updates keep the current installation. Verify actual artifacts, not just workflow syntax.

Exit: reviewable release artifacts and matching public copy. Signing-account access, production secrets, publishing, and deployment are explicit operational steps. Stores/additional repositories and Safari extension remain deferred.

### P9 — Closed beta and release readiness

Locations: `docs/TESTING.md`, compatibility records, bug log, release checklist, regression and benchmark suites.

Before beta, agree exact quality thresholds, short-word benchmark conditions, CPU/RAM limits, participant coverage, duration/exit criteria, and the remaining required app list.

Run full automated checks and real-device matrix. Measure decision-to-replacement p95 against 100 ms, separately from evidence accumulation while typing. Report accuracy and latency by direction/environment; do not hide weak combinations in pooled averages.

Use controlled synthetic markers to inspect release files, logs, errors, reports, and network paths. Confirm no automatic telemetry/crash upload and record the actual OS-memory privacy boundary.

Exit: closed-beta blockers addressed; BUG-001 closed with evidence; required layouts/platforms covered; no known blocking integrity, protected-field, disclosure, or authenticity defect; commerce/support/public claims consistent. Present a concrete release candidate and outstanding limitations for the release decision.

## 4. Review points and owner involvement

Do not stop for routine reversible implementation details within the approved scope. Ask one focused question when a product decision or unavailable resource actually blocks work.

| Point | Owner involvement |
| --- | --- |
| Plan approval | Granted by the owner; implementation started |
| Controlled live reproduction | Coordinate an isolated synthetic session; avoid collecting personal input |
| Required-platform feasibility conflict | Decide any change to scope/safety; no silent compromise |
| Native browser coverage measured | Decide extension inclusion only if needed |
| Windows/Mac testing | Confirm device access and actual OS versions |
| Quality baseline available | Review proposed numerical acceptance/beta criteria |
| Commerce/publication prerequisites | Set unresolved commercial wording; provide access only when concrete work needs it |
| Release candidate ready | Review actual artifacts/results before publication |

## 5. First execution batch after approval

1. Recheck branch/status and preserve `.htaccess`; isolate implementation work from unrelated changes.
2. Record baseline workspace checks without live input capture.
3. Identify executable/service/session/layout/app metadata and inspect the existing logging boundary.
4. Make normal diagnostic output content-free and add focused checks before a controlled live reproduction.
5. Trace the synthetic failure and report the established cause or precise blocked stage.
6. Implement and verify the smallest safe correction consistent with P1/P2 constraints; carry any deeper integration work into the next package.

No completion date is asserted before feasibility evidence. Estimate the next concrete batch after P0/P1 instead of assigning a misleading duration to unfinished cross-platform work. All approved first-release scope remains required.

## Current safety finding (2026-09-25)

P1/P2 source review identified a target-binding/input-serialization gap in the uinput adapter. Ordinary working-copy run is gated before capture/mutation; read-only and controlled analysis diagnostics remain. [Replacement safety boundary](REPLACEMENT_SAFETY.md) records evidence, containment and the contract to validate next. P2/P3 are not complete and BUG-001 remains open.

P1 follow-up: [IBus feasibility review](IBUS_FEASIBILITY.md) completed at source/API level. Next is an isolated synthetic-client lifecycle/preedit prototype; no desktop installation or successful application correction is implied.

P1 experiment: isolated IBus.Engine protocol fixture implemented; eight synthetic tests passed. Full daemon/client routing, lossless draft disposition and fresh content-type validation remain pending. See integrations/ibus/README.md.

P1 follow-up: seven private-daemon routing checks passed in addition to eight direct IBus.Engine checks. Server-side draft preservation is demonstrated for synthetic contexts; fresh field eligibility and actual application client behavior remain unresolved.

P2 experiment: field/focus-generation acknowledgement protocol added to isolated IBus prototype; 22 checks pass. Production observation authority, field binding and revocation/draft lifecycle remain pending.

P1 read-only binding check: accessible field changed while IBus context stayed unchanged. See FIELD_BINDING.md. A real field-level observer and ordered binding remain required; no scope change or live correction authorization follows.


### Focus notification feasibility review — 2026-09-25

Reviewed GTK's content-free focused-state notification and the public IBus bus
signal declarations. Separate observer events/queries do not provide an ordered
per-field input generation. See `FIELD_BINDING.md` for source/version limits and
the bounded diagnostic specification. No live subscription or installed changes;
production field eligibility remains blocked, and BUG-001 remains open. Next:
validate narrow event filtering in isolation before a coordinated metadata-only
Text Editor transition check.


### Live focus notification evidence — 2026-09-25

Narrow observer filtering passed its private-bus check. The coordinated live retry
with 90 seconds of preparation completed: one gained/lost focus notification,
a different accessible target, and expected payloads only. See FIELD_BINDING.md.
Notification delivery is established for this session; binding and ordering with
input remain unproven. Next: isolated delayed-notification races and draft
preservation contract before any production eligibility connection.


### Delayed-notification result — 2026-09-25

25 isolated checks passed, including three explicit reproductions of stale
permission/unreported focus transitions/unresolved client preedit. Production
binding remains blocked; see FIELD_BINDING.md. Next: compare integration options
for an authoritative input-ordered field generation and lossless draft handoff;
obtain owner approval before significant architecture changes.


### Integration comparison — 2026-09-25

See INTEGRATION_OPTIONS.md for five alternatives and source limits. Recommended
next milestone is a native GTK/IBus real-widget lifecycle experiment, pending
owner approval and isolated-display setup. No architecture or scope change made;
production correction remains gated.


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
