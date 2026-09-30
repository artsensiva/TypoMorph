# Architecture: observed implementation and target constraints

Status: documentation approved; implementation design remains subject to the plan's review gates. Updated 2026-09-24.
Product requirements are approved. This document does not finalize a new crate topology, OS API choice, GUI framework, account backend, license format, or implementation plan.

## 1. Current workspace

The workspace contains six crates: `core-engine`, `platform-linux`, `daemon`,
`licensing`, `settings`, and `native-host`. `prompt-cloud` is explicitly excluded;
core prompt modules are no longer exported. Historical AI sources are not built.
There are no Windows or macOS adapter crates yet.

## 2. Current execution paths (2026-09-28)

- Production `run` refuses before capture because no safe replacement backend exists.
- Explicit `run --dry-run` is a controlled diagnostic, never production correction.
  It checks saved pause and initial field metadata before opening devices. Its
  256-event queue is bounded; overflow or device loss stops the stream. Pause
  closes and joins readers, clears input state, and resume requires fresh metadata.
  CLI preference changes are polled every 100 ms; this is not an instantaneous
  acknowledged cross-process pause protocol. AT-SPI ordering remains unproven.
- Recognized developer contexts are excluded regardless of entitlement.
- `settings` serializes updates using an OS file lock and atomic replacement;
  malformed/future settings fail closed. No text or credentials belong in it.
- `licensing` verifies domain-separated Ed25519 tokens against caller-supplied
  trusted public keys, account/device bindings and confirmed access periods.
  It performs no network I/O. No production keys or account service are configured.
- `native-host` accepts frames of at most 4 KiB and reports readiness only. It
  never authorizes correction. Extension sources request only Native Messaging,
  read no page content, and expose no AI/cloud or legacy Free/Pro controls.

Token, analysis buffer, layout decision and raw key event Debug output is redacted.
No live input was captured while implementing these changes. The installed binary
is unchanged. Pure tests and private-bus fixtures do not establish native application
correction, safe undo, or protected-field guarantees.

## 3. Target responsibility boundaries

These are responsibilities required by [SPEC.md](SPEC.md), not a mandated crate split:

| Responsibility | Required boundary |
| --- | --- |
| Local decision engine | No network or persistence of input; exact selected-layout candidates; abstention |
| Platform integration | Input and context observations, composition status, actual layout identity, safe replacement capabilities |
| Safety/context policy | Protected/unknown-field suppression, exclusions, pause, context invalidation, user overrides |
| Replacement and undo | Preserve genuine input, verify current context, bounded transient restoration data |
| UI/settings | Tray, localized settings, onboarding, sounds, non-content status reasons |
| Entitlement client | Local access decision from authenticated service-issued state; no typed content |
| Account/payment service | Email verification, shared trial, device slots, Stripe state, issuance/recovery of offline access |
| Update delivery | Authentic packages, consent before install, shared version stream |
| Conditional browser integration | Reliable field access where needed, shared behavior, one owner of each correction |

Existing pure core logic can be assessed for reuse after implementation authorization. Legacy AI/cloud routes cannot remain a reachable first-release feature merely because they already compile.

## 4. Required local control flow

Before collecting text, establish that access is valid, the application is not paused/excluded, the layout is selected, no IME is active, and the field is safe. Unknown context is a blocked state.

For eligible input:
1. Interpret text-producing events under the actual selected OS layout.
2. Maintain bounded transient context and consider plausible alternatives during word entry.
3. Abstain until evidence is sufficient; do not treat language classification as a switch command.
4. Before mutation, verify that focus, cursor, field safety, source text, and concurrent input still permit it.
5. Apply a correction without losing/reordering genuine input, or skip/defer it.
6. Keep only the minimal still-valid undo information.

This requires evidence of reliable context access on each platform. Raw key capture alone does not fulfill the requirement. If a required environment cannot satisfy it, report the incompatibility to the owner before changing scope or the safety policy.

## 5. Conceptual states and precedence

| State | Meaning | Exit condition |
| --- | --- | --- |
| Ineligible | Trial/paid access absent or expired | Valid local entitlement |
| Paused | Explicit persisted user pause | Explicit resume |
| Suspended | Unsafe/unknown field, unsupported layout, active IME, or excluded context | Verified safe eligible context |
| Monitoring | Eligible input accumulated locally | Confident candidate or context invalidation |
| Candidate | Proposed correction, not yet applied | Context revalidation or discard |
| Replacing | Safe operation in the current field | Completion or immediate safe fault handling |
| Undo available | Last correction safely restorable | Undo, invalidation, or bounded retention expiry |
| Faulted | Capture/replacement failure | Readiness check; repeated failures need user action |

These labels do not commit to a particular implementation type or concurrency framework. Safety precedes entitlement, user app overrides, and confidence. A resumed process must not replay a stale event backlog or stale correction.

Undo and analysis contexts need separate lifetimes. Clearing one does not authorize retaining protected content in the other.

## 6. Language data and OS adapters

Use exact OS layout identities and capabilities rather than assuming a language maps to one universal key table. Validate modifier behavior, casing, punctuation, dead keys, layout changes, and all supported conversion directions.

A representation allowing language expansion without disrupting platform code is desirable, but independently downloadable packs and any pack-update protocol are not approved first-release commitments.

Windows/macOS capture, accessibility, layout switching, and text replacement mechanisms must be evaluated before selection. On Linux, investigate BUG-001 without assuming that Wayland, classification, permissions, or GNOME switching is already proven to be the cause.

## 7. Licensing and network separation

The service holds email/account/payment/activation metadata only. Stripe's payment status is an input to an entitlement service, not a keyboard-classification dependency.

Required semantics:
- account-wide seven-day trial;
- three installation slots using random IDs, not hardware fingerprints;
- annual access through the confirmed paid-through date offline;
- perpetual offline access without repeated validation;
- explicit paid enrollment, cancellations, first-payment refunds, and confirmed failed-renewal grace;
- account-based device release with the accepted offline-revocation limitation.

A server-authenticated local entitlement is a target security property. The local verifier now uses the wire contract documented in LICENSING.md. Production key provisioning/rotation, clock handling, recovery, and service hosting remain incomplete. No production keys, endpoints, accounts, or Stripe resources are created by this documentation update.

## 8. Browser extensions

The host remains independent of the daemon but now only reports unavailable correction; all prompt handling has been removed from the working build. The approved product does not yet select a final extension architecture.

First measure native browser coverage. If an extension is needed, propose how local safety, settings, access state, lifecycle, and correction ownership are coordinated. A stopped/missing desktop installation and duplicate correction must have explicit behavior before approval. Extension store delivery also requires resolution.

Do not describe current native-host execution as already implementing a unified running-app lifecycle.

## 9. Distribution, diagnostics, and evidence

The current CI runs fmt, clippy, tests, and dependency audit on Ubuntu. The release workflow prepares draft Debian assets with a signed checksum bundle. Neither establishes native Windows/macOS compatibility or the target signed updater.

The working build removes live-path input logs and redacts content-bearing Debug output; release privacy validation remains incomplete. Development-only diagnostic affordances need a separate safety boundary. Verify account, payment, update, reporting, and failure paths cannot receive input content.

See [TESTING.md](TESTING.md) for evidence and [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) for unresolved architecture choices. No compliance certification, zero-allocation guarantee, or unconditional platform support is implied.

## P1 working-copy update: GNOME layout integration

The discovery observations above describe the earlier source baseline. The working copy now replaces ignored-setting/Eval switching with a typed local GNOME companion protocol. See [bridge documentation](../integrations/gnome/README.md) and [investigation evidence](P0_INVESTIGATION.md). Only layout identity and acknowledged activation are exposed; field safety, context transactions, and full cross-platform support remain outstanding. The layout-only companion was installed and switching validated; it does not provide field ownership or a safe edit transaction.
