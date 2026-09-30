# TypoMorph product specification

Status: product baseline and documentation approved.
Decision basis: owner-approved discovery review, 2026-09-24.
This specification describes the first public release, not the capabilities of the current prototype.

## 1. Authority and purpose

The Product Owner approves product changes. [DECISIONS.md](DECISIONS.md) records the final choices and superseded alternatives. Open details are listed in [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md); a TBD is not a promise of support.

TypoMorph repairs text typed under the wrong keyboard layout for everyday multilingual writing. The core experience is local, conservative, and compatible with ordinary browsing, messaging, and document editing.

The product must not change correct text solely because its language differs from the selected keyboard layout's name. It does not provide spelling correction, translation, AI prompt improvement, or general writing assistance.

## 2. First-public-release boundary

All of the following are required, with detailed matrices in [COMPATIBILITY.md](COMPATIBILITY.md):

- Input languages: English, Russian, Ukrainian, German, French, Spanish.
- OS targets: Ubuntu LTS GNOME Wayland/X11 and Windows 11 on x86-64; macOS on Apple Silicon and Intel.
- Automatic correction during word entry, manual correction, safe undo, and context suppression.
- Tray/menu-bar controls, settings, onboarding, and UI localization in the six languages.
- Shared email account, trial, annual/perpetual entitlements, three devices, Stripe payments, and self-service device transfer.
- Signed direct-download distribution and consent-based installation of verified updates.
- Automated tests, real-application tests, and a closed beta before public release.

There is no calendar deadline. Release as soon as the required scope and readiness gates are met; do not silently reduce scope to ship sooner.

Browser extensions are conditional on native compatibility evidence and further owner approval. Their absence must not categorically disable a browser. Safari extension is deferred.

## 3. Functional requirements

### FR-01: Correct wrong layouts during typing

Evaluate the evolving word and apply a correction as soon as evidence is sufficient, without requiring a space. Wait for more characters or abstain if ambiguous.

Acceptance:
- Correctly typed text remains unchanged, including English written using a suitable non-English layout.
- A detected language alone never authorizes a switch.
- Genuine keystrokes and their order survive correction.
- Low-confidence input is not force-corrected.

### FR-02: Select and maintain exact layouts

Only explicitly selected supported layouts installed in the OS participate. Onboarding proposes discovered layouts for confirmation.

New installed layouts are detected during operation or at the next start and offered for inclusion; they are never silently enrolled. Changing manually to an unselected or unsupported layout clears context and suspends correction.

All unambiguous directed conversions between selected supported layouts are in scope. Distinguish language identification from actual layout mapping and correction support. Exact OS identifiers must be tested before appearing in the supported list.

### FR-03: Manual correction

A configurable shortcut corrects the last word or an explicitly selected fragment in a supported safe field.

- Apply a unique confident candidate immediately.
- Offer candidate previews when ambiguous.
- Last-word correction changes the system layout to the target.
- Selection correction preserves the current system layout.
- No system-clipboard copy/paste fallback is allowed; explain unavailable direct selection access.

### FR-04: Undo

A configurable shortcut restores the latest automatic correction's original text and previous layout while preserving subsequent input whenever restoration is still demonstrably safe in the same field.

Suppress re-correction of the canceled current word until its completion. Do not create a persistent word exception. If cursor/focus/selection changes or other events invalidate the undo context, make undo unavailable instead of guessing. Retain only bounded, transient undo information in RAM; timing and size limits remain open.

### FR-05: Respect manual layout changes

On manual layout change, clear analysis context and suspend automatic correction until the next word boundary. Resume only if the resulting layout is selected, supported, and safe in the current context. Do not immediately reverse the user's choice.

### FR-06: Context lifecycle and stale decisions

Reset analysis context on a field/window change, cursor movement, selection change, paste, and typing inactivity. Specify the timeout and exact boundary rules through tests before beta.

Analysis and undo state have different purposes: resetting analysis is not permission to retain invalid undo state or to discard a still-safe undo without the agreed policy. Revalidate context before mutation. Delayed decisions cannot be applied to a different field or stale text.

### FR-07: Protected and unknown fields

Do not collect text into buffers or correct input in recognized protected/password fields or when field safety cannot be reliably established. Clear previous input state on entry. Do not treat missing context information as permission.

An absent browser extension does not disable all browser fields. Use reliable native context when available; suppress only protected or unresolved contexts.

### FR-08: Application exclusions and privileges

Disable processing by default in recognized terminals, IDEs, games, remote-desktop clients, and VM windows. Users can explicitly opt applications in where the shared safety rules still hold. Code-block-level editor integration, remote-session support, and game-chat compatibility are not promised for this release.

Run as the ordinary user. Request necessary OS permissions during setup, but do not run the entire application permanently as administrator/root or add a privileged compatibility mechanism for this release.

### FR-09: Composition and shortcuts

With an active IME, suspend input processing and clear buffers; IME-language correction is deferred.

For dead-key sequences in supported alphabetic layouts, wait for composition to complete before considering the resulting character. Handle casing, punctuation, and diacritics according to the actual layout.

Exclude command chords involving Ctrl, Cmd/Meta, or command-use Alt from text interpretation. Handle Shift and AltGr/Option as character modifiers when appropriate for that layout. Do not replay command chords as text.

### FR-10: Concurrent typing and failures

If continuing user input prevents safe replacement, skip or defer the correction and revalidate before retrying. Never discard genuine keys or reorder characters in order to finish a correction.

On capture/replacement failure, stop corrections, clear transient context, and expose an error state without blocking ordinary typing. Automatically resume only after readiness is re-established. Repeated failures require explicit user resumption; retry limits are to be defined.

### FR-11: GUI and onboarding

Provide a tray/menu-bar icon and a settings window for selected layouts, configurable shortcuts, application exceptions, sound controls, and license/account state.

Offer autostart during onboarding, enable it after confirmation, and allow changes later. Represent temporary suspension and its reason in the icon/menu without a popup at each context transition. Do not include field contents or typed text in status messages.

### FR-12: Persistent pause

Manual pause persists across restarts until explicit resumption. Stop background input collection and clear buffers. Manual correction while paused is limited to explicitly selected text obtained on demand in a safe field; last-word correction without selection is unavailable.

Paid/trial entitlement and field safety remain required for any manual action.

### FR-13: Sounds and localization

Provide two short, distinct signals: automatic correction with a layout change, and undo. Emit one signal per action. Sounds are off by default, may be enabled during setup, and have a global menu toggle.

Localize onboarding, settings, and user-facing errors into English, Russian, Ukrainian, German, French, and Spanish. Start with the supported OS UI language, otherwise English; allow an independent manual UI-language choice. Technical documentation and source code remain English.

### FR-14: Access, accounts, and commercial lifecycle

Implement the approved rules in [LICENSING.md](LICENSING.md):
- One seven-day trial per verified-email account, no card required, up to three computers sharing the same start/end dates.
- USD 7 annually with automatic renewal after explicit paid enrollment, or USD 19 perpetual access with all future released updates.
- No permanent free tier or language-based payment gate.
- Shared features and safety protections across valid trial and paid access.
- All correction disables on expiry, except the agreed seven-day failed-renewal grace.
- Ordinary keyboard use never depends on entitlement or service availability.

### FR-15: Distribution and updates

Distribute from the website as a Debian package, signed Windows installer, and signed/notarized macOS application in a DMG.

Automatically check for updates with an opt-out; install only after consent. Missing/invalid authenticity verification blocks installation and leaves the current version in place. A warning-confirmation bypass is not allowed.

Use one update stream for annual and perpetual customers. Store publication and additional repositories are deferred. Website prices are USD, applicable taxes included; optional dated EUR estimates have no effect on the charged currency.

### FR-16: Support and diagnostics

Offer email or a contact form with no guaranteed response time, equally for trial, subscription, and perpetual users. Publish common troubleshooting instructions.

Release diagnostics exclude typed text and key sequences. No automatic telemetry/crash uploads. The user can inspect and deliberately send a technical report. Development-only local text diagnostics are permitted when necessary under the boundaries in [PRIVACY.md](PRIVACY.md).

## 4. Non-functional requirements

| ID | Requirement | Acceptance direction |
| --- | --- | --- |
| NFR-01 | Input integrity | No lost/reordered genuine input or stale-target replacement; known reproducible violations block release |
| NFR-02 | Latency | At least 95% of short-word replacements complete within 100 ms from the correction decision on agreed hardware; no perceptible slowing of ordinary typing |
| NFR-03 | Accuracy | Measure false corrections and missed corrections separately for every supported direction; approve numeric thresholds after baseline measurement and before beta |
| NFR-04 | Privacy | Transient bounded input/undo state; no release input persistence or transmission; no automatic telemetry/crash reporting |
| NFR-05 | Recovery | Safe cessation on fault; readiness check before recovery; persistent pause after repeated faults |
| NFR-06 | Compatibility | Validate exact OS/layout/application combinations; do not infer support from a build, dictionary, or matching language name |
| NFR-07 | Offline access | Entire confirmed trial/annual period offline; perpetual use offline indefinitely after activation |
| NFR-08 | Release trust | Verify update authenticity before installation; ordinary-user runtime; signed/notarized target packages |

The evidence-accumulation time while a user types is separate from NFR-02. Define short-word length, end-of-replacement observation, sample count, hardware/load, and confidence thresholds before using the target as a pass/fail measurement.

The old universal sub-millisecond end-to-end and zero-allocation promises are not acceptance claims for the existing implementation. Memory/CPU budgets and OS-level swap/dump boundaries remain open. Lower latency is a post-release priority.

## 5. Safety precedence and required examples

Safety and input integrity take precedence over automatic/manual mode, valid payment, user app overrides, and confidence. Losing a signal must not convert an unsafe context into an allowed one.

| Scenario | Required behavior |
| --- | --- |
| Confident wrong-layout word fragment | Correct before a delimiter when safe |
| Correct English on a German layout | Leave text and layout unchanged |
| Multiple plausible targets | Automatic abstention; manual candidate choice |
| User undoes and keeps typing | Preserve later characters; do not re-correct the current word |
| Focus changes before replacement | Discard the stale action |
| Protected/unknown field | No text buffering or correction; clear preceding state |
| IME active | Suspend, clear context, leave native composition alone |
| Dead key still composing | Wait; evaluate only after the resulting character is available |
| Typing races with replacement | Preserve genuine input; skip/defer correction |
| No direct selection access | Explain unavailability; do not use clipboard fallback |
| Paused after restart | Remain paused; no background collection |
| Entitlement expires | Stop correction, not ordinary typing |
| Update authenticity fails | Keep current version; block update |

## 6. Explicit exclusions

Do not build AI prompt improvement (local or cloud), general spelling correction, user-word learning/dictionaries, clipboard history, dictionary sync, Enterprise/fleet controls, code-aware editor plugins, IME correction, remote-session/VM support, Safari extension, custom/phonetic layouts, other Linux environments, Windows 10, Linux/Windows ARM, or store/repository distribution for this release.

Next language expansion priorities are Greek, Turkish, Polish, and Portuguese; they are not first-release obligations. Chrome/Edge/Firefox extensions are a conditional decision, not an unconditional deliverable.

## 7. Release acceptance and approval gates

Release requires:
1. [BUG-001](BUGS.md) reproduced, resolved, and covered by regression evidence.
2. Approved concrete compatibility and measurement matrices.
3. Automated checks, real-application validation on all target platforms and a closed beta.
4. Accuracy and latency results reported by direction/context rather than a single pooled score.
5. No known release-blocking input loss, unsafe-field processing, input disclosure, or update-authenticity bypass.
6. Validated commerce/offline access, packaging, UI localization, and consent flows.
7. Public claims, checkout, privacy notice, and customer terms aligned with demonstrated behavior.

Product requirements and documentation were approved. The [implementation plan](IMPLEMENTATION_PLAN.md) has also been approved; local implementation and controlled validation within its scope are authorized.
