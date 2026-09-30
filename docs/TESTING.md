# Validation and release evidence

Status: required validation, not a record of passing product tests.
Recorded: 2026-09-24.

No application tests, builds, live capture, payment operations, or downloaded-artifact verification were performed for this documentation update. Existing tests and CI configuration establish test assets, not current pass results or platform support.

## 1. Evidence levels

Label every result as source inspection, synthetic automated test, simulated input, real-application reproduction, or beta observation. Record exact build/commit, platform/session, layout IDs, app/version, fixture, expected/actual behavior, and date.

Use deliberately synthetic non-sensitive text. Do not collect personal typing histories, password examples from real accounts, or field content from unrelated applications. Release diagnostics and reports must meet [PRIVACY.md](PRIVACY.md).

## 2. Existing automated entry points

For the owner's Ubuntu 26.04.1 / Wayland and proposed Windows 10 Home checks,
see [local machine testing](LOCAL_MACHINE_TESTING.md). It separates device-free
tests from unavailable production autocorrection and experimental OS coverage.

The current Linux-oriented workspace has these standard commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The [CI workflow](../.github/workflows/ci.yml) also runs a dependency audit. Follow the actual workflow for tool installation/version details. Linux development dependencies are described in [CONTRIBUTING.md](../CONTRIBUTING.md).

Device-free simulation:

```bash
printf '%s\n' 'ghbdtn' | cargo run -p daemon -- test-input
```

The simulation prints input/output and bypasses real capture, context access, layout switching, and replacement. A passing classifier/keymap test cannot close BUG-001 or establish protected-field safety.

Current browser extension packaging is a separate build step documented in [extensions/README.md](../extensions/README.md). Packaging success is not extension safety or browser compatibility evidence.

## 3. Required requirement coverage

| Requirements | Necessary cases and evidence |
| --- | --- |
| FR-01, FR-02 | Within-word correction; correct text unchanged; ambiguous abstention; exact selected-layout IDs; all supported directed conversions; newly installed/unselected layouts |
| FR-03 | Unique manual result; ambiguous preview; selection vs last-word layout policy; safe refusal without direct selection access; no clipboard use |
| FR-04 | Undo restores text/layout and preserves later input; current-word re-correction suppressed; invalid context refuses undo; bounded retention |
| FR-05, FR-06 | Manual switch, word boundary, focus/window/cursor/selection change, paste, inactivity, stale queued decisions |
| FR-07, FR-08 | Password/protected and unknown fields never buffered/corrected; ordinary safe browser fields without extension; default app exclusions; opt-in cannot bypass safety |
| FR-09 | Active IME clears/suspends; dead-key completion; Shift/AltGr/Option; shortcuts never interpreted or replayed as text |
| FR-10 | Typing during every replacement stage; no lost/reordered input; failure leaves typing working; verified recovery; repeated faults require resume |
| FR-11, FR-12 | Setup permissions/layout confirmation; autostart consent; accurate state reasons; persistent pause and no background collection; explicit selection action while paused |
| FR-13 | Six UI languages, OS fallback/manual language selection; sound off by default; one signal per action; global mute |
| FR-14, NFR-07 | Account-wide trial; three devices; renewal/cancel/failure grace; full-price perpetual conversion; offline dates; transfer/refund limits; authenticated entitlements |
| FR-15, NFR-08 | Signed target packages; valid/invalid/missing authenticity metadata; no bypass; update consent/opt-out; old version retained after verification failure |
| FR-16, NFR-04 | No release text in logs, files, reports, requests, or failures; manual report review; no automatic telemetry/crash upload |
| NFR-01, NFR-05 | End-to-end integrity and fault injection on every adapter, including rapid focus changes and continuing typing |
| NFR-02, NFR-03 | Reproducible latency distributions and false/missed correction rates by direction; declared sample/hardware/load conditions |
| NFR-06 | Real tested OS/layout/app combinations with versions; missing coverage visibly unverified |

Valid entitlement never substitutes for a safety check. Test paused, expired, excluded, and unknown-context paths as well as normal operation.

## 4. Language and accuracy matrix

Use [COMPATIBILITY.md](COMPATIBILITY.md) as the target inventory. For each selected source/target layout variant, test:

- intended wrong-layout words and fragments;
- valid source-language text and valid foreign-language text entered on a suitable layout;
- short/ambiguous words, names, numbers, punctuation, casing, and supported diacritics;
- keyboard modifiers and composing sequences;
- every promised direction independently, including Latin-to-Latin ambiguity;
- automatic abstention and manual candidate choice where appropriate.

Separate false corrections from missed corrections. Do not report a single combined accuracy figure that hides a poor direction or counts unsupported samples as successes.

Baseline data, corpus provenance/usage rights, minimum sample sizes, and numeric thresholds must be reviewed before beta. Do not fabricate a percentage or claim full support from the number of classifier profiles.

## 5. Performance measurement

The target is p95 at most 100 ms from correction decision to short-word replacement completion on agreed hardware, without perceptible ordinary-typing delay.

Before measuring, define word length, timing endpoints, repeats, warm/cold conditions, input rate, load, hardware, and app/session configuration. Report evidence accumulation before the decision separately. Report real replacement latency, not just classifier execution time.

CPU/RAM budgets and transient analysis/undo limits remain open. Auxiliary vectors/queues and allocations need measurement; the ring buffer size alone is not a memory bound for the application.

Further reduction of latency is deferred beyond the initial target, not an exemption from meeting it.

## 6. Real-device and browser coverage

| Environment | Available evidence now | Required next evidence |
| --- | --- | --- |
| Owner's Ubuntu 26.04.1 / GNOME 50 / Wayland | Owner-reported failure, BUG-001 | Identify build/apps/layouts; reproduce; validate any eventual fix |
| Ubuntu GNOME X11 target | No test session confirmed | Select an actually available supported configuration and run the same suite |
| Windows 11 x86-64 | Owner has a machine; version/build not recorded | Adapter plus native tests |
| macOS Apple Silicon | Owner plans access via friends | Confirm hardware/OS, permissions, native tests, signed package |
| macOS Intel | Owner plans access via friends | Same; do not infer results from Apple Silicon |

Required browser families: Chrome and Firefox on all target platforms, Edge on Windows, Safari on macOS. Record versions and native vs compatibility backend where relevant. Cover plain editable fields, rich text, protected fields, inaccessible/unknown contexts, and focus transitions. Unsupported field types must produce safe, understandable behavior.

Native desktop editors, messaging apps, and document editors need a concrete test list. "Text Editor" in BUG-001 is not yet an identified package/version. Cross-platform permission, layout, IME, and update behavior cannot be validated by Linux CI alone.

## 7. Commerce, privacy, and delivery tests

Use test payments and controlled accounts after authorization. Cover first activation, shared trial dates, device limit/release, email recovery, cancellation, renewal success/failure, the seven-day grace, first-payment refund, and subscription-to-perpetual conversion. Exercise duplicate/out-of-order service events and outages.

Verify confirmed offline rights without periodic validation. Document that an offline deactivated/refunded perpetual installation cannot be immediately revoked; do not write a test that silently assumes the opposite product rule.

Inspect controlled release-build output, files, reports, and requests for synthetic marker leakage. Include error/panic paths and packaging/service defaults. Development-only logging must not be enabled in release. OS swap/dump guarantees require separate design evidence.

Verify actual artifacts and update failures. A workflow containing a signing command is not proof that a downloaded release was authenticated.

## 8. Closed beta and release decision

Closed beta is mandatory after the concrete compatibility matrix and numeric quality thresholds are approved. Agree participants, duration, coverage, and exit criteria before starting it; no date or sample count has been approved yet.

Release requires resolved BUG-001, required matrix coverage, passing checks and measured targets, no known blocking safety/privacy/integrity/authenticity defects, validated commerce, and public claims aligned with evidence. See [SPEC.md](SPEC.md) and [ROADMAP.md](ROADMAP.md).

Documentation checks may validate Markdown links, consistency, and edit scope. They must never be reported as application test success.

## Latest implementation evidence

See [PROJECT_STATE.md](PROJECT_STATE.md) for the 2026-09-28 workspace, private-bus,
release-profile and mocked installer results. These supplement, and do not replace,
the native safety failures and required cross-platform release matrix above.
