# Privacy requirements and data boundaries

Status: product policy and documentation approved, 2026-09-24.
This is an engineering/product policy, not a completed public privacy notice. The current prototype does not yet comply.

## 1. Input boundary

Release builds must not transmit or intentionally persist typed text, raw key sequences, input history, or undo text. Keep only bounded, short-lived RAM state needed for the current correction and safe undo.

Protected fields, unknown-safety fields, active IMEs, excluded contexts, and manual pause must not collect text into the background analysis buffer. Clear prior input state when entering unsafe/suspended contexts. Manual selection access while paused is an explicit, on-demand exception only for safe fields.

No:
- network classification or local/cloud AI prompt feature;
- input logging to files, stderr, service journals, traces, diagnostic reports, or crash payloads;
- automatic telemetry, usage analytics, or automatic crash-report uploads;
- persistent learned words, user dictionaries, clipboard history, or dictionary sync;
- selection extraction through system clipboard copy/paste.

Timing, memory-size limits, crash-dump/swap/hibernation boundaries, and safe handling of partially completed replacement must be explicitly resolved. Do not claim that RAM-only application design prevents every OS-level copy of memory.

## 2. Data classes

| Data | Approved purpose | Storage/transmission boundary |
| --- | --- | --- |
| Current input and conversion candidates | Local correction | Transient RAM only; never part of service requests or logs |
| Undo text and context | Safe restoration of last correction | Bounded transient RAM; cleared when context is invalid/unsafe |
| Explicit manual selection | Requested correction/preview | Local on-demand processing only; no clipboard fallback |
| Settings | Layout choices, shortcuts, pause, sounds, exclusions, UI language | Local persistence is allowed; no typed-text history |
| Random installation ID | Account device slots and activation | Local and minimal activation-service metadata; no hardware fingerprint |
| Verified email and account identity | Sign-in, shared trial, purchase recovery | Account service; exact retention/deletion rules TBD |
| Entitlement and billing references | Paid-through/perpetual access, renewals, devices | Local access proof and appropriate account/payment services; exact schema TBD |
| Payment details | Checkout | Provider-hosted collection; no keyboard content in payment metadata |
| Update requests | Version checks/downloads | Content-free requests; automatic check can be disabled |
| EUR reference rate | Website display | Website-side refresh; no desktop currency requests |
| Technical support report | User-requested troubleshooting | No typed text or key sequences; user inspects before sending |

The no-input-retention rule does not forbid necessary non-content settings or licensing metadata. Conversely, calling data "diagnostics" does not permit retaining input.

Normal network infrastructure can observe connection metadata. Server access logs, IP handling, provider records, retention, deletion/export, and notice wording must be defined before launch; "no telemetry" must not be advertised as "no account or payment data."

## 3. Development diagnostics

The owner permits local text capture when necessary for development/debugging, but not in release behavior.

Operational safeguards to implement under the approved plan:
- an explicit development-only diagnostic path;
- controlled, non-sensitive synthetic test input;
- clear indication that input capture is enabled;
- no capture from ordinary personal sessions or automatic upload;
- no input-bearing diagnostics, traces, or test artifacts in shipped release builds.

A precise enablement/cleanup procedure and guard against accidental release inclusion remain implementation details. This documentation change does not authorize starting input capture or collecting existing personal journals.

## 4. Support and reports

No automatic crash submissions. The user can review and deliberately send a technical report containing such items as build, OS/session, error category, and permission state, with a final approved field list.

Do not include field contents, window/document titles containing personal text, raw device identifiers, credentials, or memory dumps by default. Reports and account data require an explicit retention policy; it is not yet fixed.

## 5. Current prototype discrepancies

The diagnostic daemon inspected at discovery printed characters, analyzed buffers, and locally improved prompt content through stderr. P0 removes these live-path outputs in the working copy; the installed binary is unchanged. Systemd may retain that output. As of 2026-09-28, the working build removes legacy CLI and browser cloud-prompt paths. The cloud crate is excluded from the workspace; historical source remains uncompiled.

These historical behaviors are not the approved release policy. Current extensions request no page access, and their host refuses correction. Capture queues are now bounded and readers are joined on diagnostic pause. Token/buffer/decision/raw-event Debug formatting is redacted. Persistent preferences contain no input or credentials. These controls are tested in isolation; general protected-field safety and production integration remain unresolved. See [ARCHITECTURE.md](ARCHITECTURE.md), [SECURITY.md](../SECURITY.md), and [BUGS.md](BUGS.md).

## 6. Verification

Use controlled synthetic content to check release stdout/stderr, service logs, files, network traffic, error handling, updates, and account flows. Confirm absence of input-bearing persistence/transmission, including failure paths.

Check no background text collection during pause and no retention across protected/unknown contexts. Test activation/update services independently from layout analysis. Publish privacy claims only after those boundaries have evidence in [TESTING.md](TESTING.md).
