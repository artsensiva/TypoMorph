# Product roadmap and approval gates

Status: product scope approved; updated documentation awaiting review.
Date: 2026-09-24.

There is no desired fixed release date. Work as quickly as practical and release when the complete agreed scope meets its readiness criteria. Speed does not waive data safety, testing, or approval.

## 1. Authorization

| Gate | Status |
| --- | --- |
| Interactive discovery and final requirements review | Approved by the owner |
| Update English technical documentation | Authorized; this documentation change |
| Owner review of the revised documents | Pending |
| Detailed implementation plan | Must be proposed after documentation approval |
| Explicit implementation-plan approval | Pending; required before application code changes |
| Release readiness | Not reached |

The stages below are proposed sequencing from the approved review, not a detailed implementation plan or permission to edit code, fix bugs, refactor, create commits, or deploy.

## 2. Proposed delivery stages after approval

| Stage | Purpose | Required evidence / exit |
| --- | --- | --- |
| A. Reproduction and feasibility | Reproduce BUG-001; establish safe field/context access and replacement feasibility on each OS | Actual root-cause evidence; concrete OS/application matrix; unresolved incompatibilities returned to the owner |
| B. Input and safety foundation | Within-word decisions, context lifecycle, exclusions, composition safety, genuine-input preservation, manual correction and undo | Regression coverage and safe real-application behavior |
| C. Required language and platform coverage | Six language families and selected variants on all target platforms | Per-direction mapping/accuracy evidence, both Mac architectures, GNOME Wayland/X11, Windows 11 |
| D. Product controls and access | Localized UI, onboarding, pause/sounds, accounts, Stripe, offline licenses and transfers | End-to-end user and billing lifecycle tests without keyboard-content transmission |
| E. Trusted delivery | Packages, update verification/consent, public copy, privacy/customer terms, support instructions | Reproducible artifact identities and checks; complete publishing prerequisites |
| F. Closed beta and public release | Validate ordinary workflows and eliminate blockers | Automated and real-device matrices passed, closed-beta findings addressed, claims match evidence |

Browser-extension inclusion is a decision gate after native coverage is measured. Do not make extensions unconditional or remove the safety policy to avoid them.

Exact order, dependencies, engineering choices, estimates, and any safe concurrency belong in the later implementation plan. A Linux-only or two-language public release would require a new product decision.

## 3. First-release readiness

The first public release requires:

- all six input/UI languages and approved layout variants;
- Ubuntu GNOME Wayland/X11, Windows 11, macOS Intel and Apple Silicon with explicit tested versions;
- the mandatory browser matrix and an approved native desktop-application matrix;
- correction, manual repair, safe undo, exclusions, IME coexistence, pause, settings, sounds;
- measured latency and agreed accuracy thresholds for each supported direction;
- no known blocking input loss, protected/unknown-field processing, input disclosure, or signature bypass;
- seven-day shared trial, three computers, USD 7/year or USD 19 perpetual, agreed offline behavior;
- correct renewal/cancellation/grace/refund/device-transfer handling;
- signed distribution, consent-based updates, current public terms/copy, and support instructions;
- closed-beta evidence, not just unit tests.

Current source and package presence do not count as passing these gates. BUG-001 remains open.

## 4. Post-release priorities and explicit deferrals

| Area | Later work |
| --- | --- |
| Performance | Reduce replacement latency below the first-release p95 target of 100 ms |
| Input languages | Greek, Turkish, Polish, Portuguese as next expansion priorities |
| Further languages | Italian, Czech, Bulgarian, Hebrew, Arabic subject to demand and validation |
| Layout variants | Additional regional, alternative, phonetic, and custom layouts |
| Platforms | Additional Linux environments/distributions, Windows 10 if reconsidered, Linux/Windows ARM |
| Browsers | Safari extension; other browsers only after evidence |
| Input contexts | Remote desktops, VM windows, deeper IDE/game integration, IME correction |
| Product additions | Spelling correction, user dictionaries/learning, clipboard history/sync, AI features only after new discovery |
| Commercial/distribution | Enterprise/fleet controls, stores/additional repositories, other display/payment currencies |

Deferred features are not an automatic promise to build all of them. No AI subsystem or Enterprise compliance milestone should delay the agreed first release.

## 5. Current publication mismatch

The existing landing page and artifacts contain historical offers and platform placeholders. Update them only in the later authorized release work. Existing source licensing remains in place until a separately reviewed customer agreement is prepared.

The revised documents record these gaps rather than silently changing deployment, HTML, source, or legal agreements during the documentation phase.
