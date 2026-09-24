# Open questions and risks

Status: approved product scope, unresolved implementation and release details.
Recorded: 2026-09-24.

These are not a new questionnaire and do not reopen settled product choices. Resolve technical details through evidence and the later implementation-plan review. Return a scope or safety conflict to the Product Owner rather than silently weakening requirements.

## 1. Questions to resolve

| ID | Question / required result | Required by |
| --- | --- | --- |
| Q-01 | Which exact Ubuntu LTS/GNOME Wayland and X11 versions, Windows 11 builds, and minimum macOS versions will be claimed? Verify availability of each session; do not assume GNOME 50 supplies X11. | Compatibility/implementation-plan review |
| Q-02 | Can every required OS expose safe field context, composition state, and reliable replacement at ordinary-user privilege? Which permissions are necessary? | Feasibility work before committing to adapter design |
| Q-03 | What is BUG-001's actual installed build and failing stage? Confirm launch method, permissions, layout IDs, and app versions on synthetic input. | Early authorized investigation |
| Q-04 | Which exact OS layout IDs/variants implement the language matrix, including traditional France AZERTY, Spain/Latin America, and Apple/PC RU/UA variants? | Before claiming layout support |
| Q-05 | Which native editors, messaging/document applications, and browser versions/field types form the release test matrix? | Before closed beta |
| Q-06 | Does measured native browser coverage require Chrome/Edge/Firefox extensions? If so, what are missing/stopped-app behavior, ownership, settings/access synchronization, and permitted delivery? | Owner decision after native coverage; before extension commitment |
| Q-07 | What are the analysis/undo bounds, inactivity timeout, word boundaries, and safe undo invalidation rules? | Implementation plan and tests before beta |
| Q-08 | How are concurrent typing, synthetic-event identification, context freshness, replacement failure, and retry limits handled without losing input? | Input architecture review |
| Q-09 | Which validated data, scoring/mapping representation, and confidence rules support every selected layout direction without changing correct text? | Engine design and baseline evaluation |
| Q-10 | What exact benchmark defines a short word, p95 replacement completion, sample count, hardware/load, and CPU/RAM budget? What numeric accuracy thresholds apply by direction? | Baseline first; approval before beta |
| Q-11 | Which GUI toolkit, shortcuts, conflict-resolution behavior, icons, and localization review process satisfy platform conventions? | Implementation/UI plan |
| Q-12 | Where do account/entitlement services run, and what authenticated offline format, keys/rotation, clocks, recovery, and availability policy are used? | Commerce architecture review |
| Q-13 | How is confirmed failed-renewal grace issued/delivered? How do duplicate/out-of-order payment events and subscription-to-perpetual conversion converge safely? | Commerce design before billing tests |
| Q-14 | What renewal-reminder timing, cancellation wording, and refund treatment apply to a later perpetual purchase by an existing subscriber? | Owner/commercial review before publication |
| Q-15 | What seller/tax configuration, customer terms, withdrawal/refund disclosures, and notices match the actual German business? | Appropriate review before taking payments |
| Q-16 | Which account/payment/activation/email/support metadata is retained, for how long, with which processors, and how are access/deletion requests handled? | Privacy/service review before release |
| Q-17 | What process-level protections address OS swap/crash dumps, and what can TypoMorph accurately promise about those OS-controlled copies? | Privacy implementation review |
| Q-18 | How is development-only text logging explicitly enabled and kept out of release builds and retained service logs? | Before diagnostic implementation or live capture |
| Q-19 | Which signing/notarization identities, update metadata, verification keys, artifact/version rules, and recovery procedures meet the fail-closed requirement? | Packaging/update design before beta |
| Q-20 | What EUR reference source, refresh interval, timestamp, and stale-rate cutoff fit simple website automation? | Website/pricing work before publication |
| Q-21 | Which Mac Intel/Apple Silicon machines and OS versions can actually be borrowed, who performs tests, and what closed-beta coverage is sufficient? | Test planning before beta |
| Q-22 | What support contact/form and security-reporting route are operational, and who maintains troubleshooting instructions? | Release readiness |

Email login link/code choice is an implementation detail within the approved passwordless model. No additional paid tier, device fingerprint, mandatory periodic perpetual check, or AI service is authorized by these open questions.

## 2. Material risks

| Risk | Impact | Required response |
| --- | --- | --- |
| Native field safety unavailable on a required target | Safe automatic correction may be narrower than expected | Prototype and measure after authorization; discuss any unresolved scope conflict with owner |
| Raw-key/injection approach diverges from real text state | Wrong-field edits, missed input, or corrupt replacement/undo | Validate context and races end to end; abstain when unsafe |
| Six languages across several layouts and OS families | Larger test matrix and false-correction surface than profile count suggests | Track exact directions/variants and reject pooled-only accuracy evidence |
| Linux core bug unresolved; Windows/macOS absent | No current basis for a release date or support claim | Reproduce first, build missing adapters under approved plan, record real-device results |
| Legacy text logging and cloud code remain reachable | Current prototype is incompatible with privacy promises | Keep status explicit; remove or isolate under authorized implementation, then verify |
| Low prices and lifetime updates | Fees, taxes, multi-OS upkeep, and support can consume proceeds | Measure real operating costs; do not assume maintenance is negligible or silently change pricing |
| Offline perpetual access | Revocation after transfer/refund cannot be immediate | Accepted product trade-off; disclose accurately and avoid hidden online enforcement |
| Billing/service outage or clock changes | Activation/renewal ambiguity | Design recovery while preserving confirmed offline rights |
| Signing and publication gaps | Users may receive unverified or misleading downloads | Validate packages/updater and align public claims before release |

No risk entry grants permission for production access, deployment, payment collection, or live personal-text logging. This documentation update creates no infrastructure and sets no release date.
