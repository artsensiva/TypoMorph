# Product discovery decision log

Status: product baseline, documentation, and implementation plan approved.
Recorded: 2026-09-24.

This log consolidates the final choices from discovery. It does not reconstruct timestamps or option numbers for every interview answer. Later explicit owner decisions replace earlier proposals. Product rules live in [SPEC.md](SPEC.md); implementation observations do not override them.

## 1. Final decisions

| ID | Decision | Consequence |
| --- | --- | --- |
| D-01 | Local wrong-layout correction for ordinary multilingual writing | No spelling, translation, or general writing assistant |
| D-02 | Exclude local/cloud AI prompt improvement | AI entry points removed from the working build on 2026-09-28; historical sources excluded |
| D-03 | Release text stays in bounded transient RAM; no input logs, transmission, telemetry, or automatic crash uploads | Local settings and account/payment metadata are separate from typed content |
| D-04 | Development-only local text diagnostics allowed when necessary | Explicit development controls and synthetic input; no permission to capture personal typing during documentation work |
| D-05 | Six first-release input and UI languages: English, Russian, Ukrainian, German, French, Spanish | Exact supported variants and directions need validation; all six are required at public release |
| D-06 | Select only supported layouts installed in the OS | Discover/propose layouts; user confirms; prompt for newly installed layouts without auto-enrollment |
| D-07 | Correct within a word as soon as confidence permits | Correct text remains unchanged; ambiguous automatic decisions abstain |
| D-08 | Manual last-word/selection repair and safe undo | Preview ambiguous candidates; last-word repair switches layout, selection repair preserves it |
| D-09 | Respect manual layout changes | Reset and wait through the next word boundary; unsupported/unselected layouts suspend processing |
| D-10 | Password/protected and unknown-safety fields receive no text buffering or correction | Application overrides and paid access cannot bypass this |
| D-11 | Native browser support remains useful without an extension | Do not disable every browser field solely because an extension is absent |
| D-12 | Terminals, IDEs, games, remote sessions, and VM windows excluded by default | Explicit app opt-in still requires safety; special integrations are deferred |
| D-13 | No clipboard fallback for selection access; safe IME coexistence required | Explain unavailable actions; clear/suspend during IME activity; wait for dead-key completion |
| D-14 | Preserve input under concurrent typing and failures | Revalidate or skip/defer; no lost keys or stale-target edits |
| D-15 | Ordinary-user runtime | No permanently elevated application or privileged compatibility helper for this release |
| D-16 | Tray/menu bar, settings, onboarding, optional autostart | Autostart requires confirmation; suspension reasons visible without repeated popups |
| D-17 | Persistent manual pause | No background collection; only explicit safe selection repair remains available |
| D-18 | Optional distinct sounds for automatic correction/layout change and undo | Off by default; onboarding opt-in; global menu switch |
| D-19 | Ubuntu LTS GNOME Wayland/X11, Windows 11 x86-64, macOS Intel and Apple Silicon | Exact versions and actual hardware verification remain prerequisites |
| D-20 | Chrome/Firefox on all targets, Edge on Windows, Safari on macOS | Native safety/coverage first; extensions conditional on evidence and owner decision |
| D-21 | One verified-email account, passwordless sign-in, three computers | Additional devices share the account and trial; one person's personal/commercial work |
| D-22 | Seven-day trial from first activation, no card | Adding devices does not restart it; no automatic first charge |
| D-23 | USD 7/year, automatically renewing after explicit purchase | Clear renewal notice and cancellation; no permanent free tier |
| D-24 | USD 19 perpetual with all future released updates | Same features/update stream; no major-version or twelve-month update restriction |
| D-25 | Full-price subscription-to-perpetual conversion | No prorated credit; stop subscription renewal after successful purchase |
| D-26 | Prices include applicable taxes; charge in USD | Optional dated approximate EUR display with stale-rate fallback; other currencies deferred |
| D-27 | Separate TypoMorph Stripe account under the existing login | Replaces Lemon Squeezy; no provider resources created during documentation work |
| D-28 | Online activation, then full confirmed period offline; perpetual access indefinitely offline | No periodic mandatory license checks; annual renewal needs new confirmation |
| D-29 | Seven-day grace for confirmed failed annual renewal | Not trial expiry or voluntary cancellation; delivery to offline clients needs design |
| D-30 | Random installation IDs and self-service device release | No hardware fingerprint; accept imperfect abuse prevention and offline revocation |
| D-31 | Fourteen-day voluntary refund after first payment | Both paid offers; no additional blanket renewal guarantee; statutory rights separate |
| D-32 | Email/form support without SLA for trial and paid users | No separate priority-support promise |
| D-33 | Direct signed downloads and consent-based verified updates | Stores and additional repositories deferred; invalid authenticity blocks installation |
| D-34 | p95 short-word replacement target of 100 ms from decision | Define benchmark before testing; further latency reduction after release |
| D-35 | Per-direction accuracy metrics, real OS/app tests, and closed beta | Numeric accuracy thresholds after baseline, before beta; no pooled score hiding weak directions |
| D-36 | No fixed release date; release as soon as complete and ready | Do not silently reduce languages, platforms, or safety scope |
| D-37 | Documentation review, then implementation-plan review, then code | Documentation authorization does not authorize fixes, refactors, commits, or publication |

## 2. Language and expansion decisions

First-release exact variants are in [COMPATIBILITY.md](COMPATIBILITY.md). Language profiles alone are insufficient: mapping, modifiers, composition, and correction accuracy must be validated in both directions for each relevant pair.

Greek, Turkish, Polish, and Portuguese are next expansion priorities. Italian, Czech, Bulgarian, Hebrew, and Arabic are later possibilities, not promises. Other regional/custom/phonetic layouts and IME correction are deferred. Safe behavior around an active IME is mandatory now.

## 3. Superseded commercial proposals

The final seven-day / USD 7 annual / USD 19 perpetual model replaces the earlier EUR 19 annual proposal, longer/shorter trial alternatives, voluntary-price or donation-only schemes, free access, and annual/perpetual alternatives discussed during discovery.

No accountless trial, new trial per device, restricted perpetual update window, annual-to-perpetual credit, separate Enterprise tier, or language-based payment boundary is part of the final baseline.

## 4. Contradictions with the repository before this update

| Evidence | Historical/current content | Resolution or remaining gap |
| --- | --- | --- |
| Root README, PROMPT, SPEC, SECURITY, CONTRIBUTING | Inconsistent free-language/free-core promises and Pro/Enterprise positioning | Replaced by trial plus equal-feature annual/perpetual access in these documents |
| README and other product docs | AI/local/cloud prompt improvement as a product feature | Removed from the working build; historical sources remain excluded |
| Commercial docs and landing page | Old prices, tier boundaries, and provider | Docs now use USD 7/19 and Stripe; HTML still needs a separately authorized update |
| Earlier architecture/roadmap | Future platform/common/language-pack crates represented as architecture | Current workspace documented accurately; future topology remains undecided |
| Performance claims | Universal sub-millisecond/zero-allocation statements | Replaced with an unmeasured first-release target and explicit benchmark work |
| Language support descriptions | Classifier profiles described as complete layout support | Separated detection, physical layout coverage, conversion direction, and tested applications |
| Current live daemon | Boundary-triggered limited US/RU processing | Does not satisfy within-word six-language requirements |
| Current live diagnostics | Characters and buffers printed to stderr | Violates release privacy target; not removed in documentation phase |
| Current filtering/pause/context logic | No demonstrated complete protected-field, composition, persistent-pause, or stale-context safety | Safety is required for everyone; implementation/validation still needed |
| Current Linux capture/replacement | Suppresses delivery to analysis during switching/emission | Needs race/integrity investigation; not proof of the reported bug's root cause |
| Licensing | Legacy Lemon Squeezy/checksum implementation removed | Ed25519 verifier implemented; account/Stripe/activation integration still pending |
| Browser host/extensions | Now readiness-only; prompt actions and page access removed | Conditional inclusion; lifecycle, field safety, and correction ownership still incomplete |
| Current platform tree and public placeholders | Linux prototype; Windows/macOS not implemented | Required first-release targets, not existing supported products |
| Landing installer/release automation | Existing checksum flow and Debian signing configuration | Not evidence that the approved verified updater exists or an artifact has been checked |
| Source comments and identifiers | Some historical non-English comments | English remains the engineering-language rule; do not change code during this phase |
| LICENSE | Existing proprietary source notice | Retained; commercial product rules are not a new legal agreement |
| CHANGELOG | Historical fixes and release notes | Retained as history, not evidence that BUG-001 is resolved |

The source, HTML, configuration, legal notice, and deployment state are intentionally outside this documentation edit. Their gaps remain visible in [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md).

## 5. Evidence and implementation impact

The only user-reported runtime defect is [BUG-001](BUGS.md), still independently unreproduced. Source inspection identified additional gaps; no live input was captured, no runtime test was executed, and no fix was attempted.

The requirements affect the decision engine, all OS integrations, field safety and replacement/undo, GUI, privacy boundaries, commerce service/client, packaging/updating, and release validation. [ROADMAP.md](ROADMAP.md) gives proposed stages only. [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) now defines proposed work packages and the remaining technical decision gates.
