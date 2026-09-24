# Architecture: observed implementation and target constraints

Status: documentation review, 2026-09-24.
Product requirements are approved. This document does not finalize a new crate topology, OS API choice, GUI framework, account backend, license format, or implementation plan.

## 1. Current workspace

The actual [Cargo workspace](../Cargo.toml) contains:

| Crate | Observed role | Important limits |
| --- | --- | --- |
| `core-engine` | Ring buffer, heuristic/n-gram language classification, US/RU conversion, prompt detection/cleanup | Seven language profiles do not equal seven usable layouts; scoring allocates strings/collections |
| `platform-linux` | Multi-device evdev capture, uinput replacement, GNOME-oriented switching | No cross-platform adapter; field safety/IME integration not established |
| `daemon` | CLI, boundary-triggered classification, tray, legacy license use | Debug input logging; no approved GUI/undo/account lifecycle |
| `licensing` | Lemon Squeezy activation, hashed metadata, integrity checksum | Not the approved Stripe/trial/perpetual system; checksum is not authenticity proof |
| `prompt-cloud` | Legacy BYOK/managed prompt transport | Network text path excluded from the new release; managed endpoint documented as a placeholder |
| `native-host` | Browser-spawned stdio protocol process | Reuses core/cloud crates; runs independently of the daemon |

There are no current `platform-windows`, `platform-macos`, `lang-packs`, or `common` workspace crates. Prior diagrams showing them were proposed architecture, not implemented components.

## 2. Observed live path

1. The daemon loads legacy licensing state and opens multiple input devices.
2. Keycodes are converted using the daemon's current layout string and limited tables.
3. Characters accumulate in a 32-character ring buffer, with scan codes in a separate vector.
4. A whitespace boundary triggers candidate evaluation.
5. A legacy licensed developer-window filter may bypass replacement.
6. Delivery is suppressed during layout switching and uinput emission.
7. The daemon updates its layout state and clears buffers.

Source evidence:
- [Daemon](../crates/daemon/src/main.rs): input/buffer stderr output, prompt hotkey, boundary behavior, pause loop, `xdotool` filtering, and an inactivity-reset TODO.
- [Layout module](../crates/core-engine/src/layout.rs): two candidate paths, partial physical US/RU tables, and target-layout routing.
- [Linux adapter](../crates/platform-linux/src/lib.rs): switching, event suppression, and explicit per-key sleeps during emission.
- [License store](../crates/licensing/src/lib.rs): status/checksum persistence and legacy feature access.
- [Native host](../crates/native-host/src/main.rs): its own license load and request loop.

Suppression of events delivered to the daemon is not proof that the foreground application loses keystrokes. It does, however, explicitly omit genuine input from analysis during replacement, and there is no demonstrated safe transaction with the target field. Race behavior needs reproduction and testing rather than a claimed root cause for BUG-001.

The 32-character analysis buffer also does not prove that every auxiliary buffer is bounded. Current scan-code, string, and n-gram structures must be assessed before making performance/memory claims.

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

A server-authenticated local entitlement is a target security property. Token format, signing algorithms, key rotation, clocks, recovery, and service hosting are still proposed-design work. No production keys, endpoints, accounts, or Stripe resources are created by this documentation update.

## 8. Browser extensions

The current host process is independent of the daemon and still includes prompt handling. The approved product does not yet select a final extension architecture.

First measure native browser coverage. If an extension is needed, propose how local safety, settings, access state, lifecycle, and correction ownership are coordinated. A stopped/missing desktop installation and duplicate correction must have explicit behavior before approval. Extension store delivery also requires resolution.

Do not describe current native-host execution as already implementing a unified running-app lifecycle.

## 9. Distribution, diagnostics, and evidence

The current CI runs fmt, clippy, tests, and dependency audit on Ubuntu. The release workflow builds/signs checksums for Debian assets. Neither establishes native Windows/macOS compatibility or the target signed updater.

Release input logs must be eliminated under the later approved implementation plan. Development-only diagnostic affordances need a separate safety boundary. Verify account, payment, update, reporting, and failure paths cannot receive input content.

See [TESTING.md](TESTING.md) for evidence and [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) for unresolved architecture choices. No compliance certification, zero-allocation guarantee, or unconditional platform support is implied.
