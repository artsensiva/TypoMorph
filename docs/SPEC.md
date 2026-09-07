# TypoMorph Product Specification

## 1. Overview

TypoMorph is a commercial, ultra-low-latency, cross-platform intelligent input utility designed to automatically switch keyboard layouts and apply contextual auto-correction across multilingual writing workflows. The system operates natively on macOS, Windows, and Linux, with a strong emphasis on user privacy, deterministic performance, and compatibility with modern input methods (including IME-driven languages such as Chinese, Japanese, Hindi, and Bengali).

The product is positioned as a premium utility for knowledge workers, software developers, multilingual writers, and enterprise customers who require high-confidence input switching without sacrificing typing performance or privacy.

## 2. Product Mission

TypoMorph exists to reduce friction in multilingual text entry by:

- inferring the active language and keyboard layout from recent keystroke context;
- switching layouts before the user perceives a disruption;
- correcting likely transcription mistakes with minimal false positives;
- respecting OS-native composition and IME boundaries;
- avoiding disk-based logging and retaining input data only in volatile memory; and
- enabling enterprise-grade policy controls for managed environments.

## 3. Problem Statement

Modern multilingual users frequently switch among languages and keyboard layouts while working in browsers, editors, terminals, and messaging clients. Existing solutions are often:

- too slow for real-time high-speed typing;
- prone to incorrect language detection when users mix scripts or keyboard layouts;
- disruptive to IME-driven languages and composition windows;
- invasive from a privacy standpoint; and
- unreliable in code-writing contexts where auto-correction should be suppressed.

TypoMorph addresses these issues by combining a fast probabilistic scoring engine with OS-aware input bridge logic and strict privacy safeguards.

## 4. Goals

### 4.1 Primary Goals

1. Deliver a language switching engine that is responsive enough to feel instantaneous to users.
2. Support direct alphabetic languages and IME-aware hybrid languages in a modular way.
3. Maintain compatibility with macOS, Windows, and Linux input systems without requiring root privileges for standard use.
4. Keep all transient typing data in RAM only and avoid zero-disk logging.
5. Enable premium features for power users and enterprise customers without degrading the core experience.

### 4.2 Secondary Goals

1. Provide transparent, low-noise UX that does not unexpectedly rewrite text.
2. Offer developer-friendly behavior that disables correction inside code-like contexts.
3. Support cloud-synced custom dictionaries as an optional premium feature.
4. Allow fleet deployment and policy enforcement in enterprise deployments.

## 5. Non-Goals

The system is not intended to provide:

- a full general-purpose AI writing assistant;
- permanent local text logging or analytics storage;
- root-required Linux input interception for normal usage;
- broad desktop automation beyond keyboard input management and text correction; or
- invasive monitoring of user content beyond ephemeral language classification decisions.

## 6. Target Users

### 6.1 End Users

- multilingual knowledge workers;
- developers working in multiple languages;
- writers producing bilingual or multilingual content;
- users with keyboard layouts that differ from their active language;
- users who rely on Latin alphabet languages and IME-based input methods.

### 6.2 Enterprise Users

- organizations with privacy requirements;
- IT admins managing fleet deployments across macOS, Windows, and Linux;
- teams needing zero-keystroke retention guarantees and policy controls.

## 7. Scope

### 7.1 In Scope

- proactive language detection;
- keyboard layout switching across supported platforms;
- contextual auto-correction heuristics for alphabetic languages;
- IME-aware routing for Pinyin, Romaji, and Indic phonetics;
- code-aware input suppression for developer scenarios;
- secure system integration using platform-appropriate input APIs;
- enterprise policy support and compliance posture.

### 7.2 Out of Scope

- translation service integration in the first release;
- generating large language model summaries inside the typing loop;
- cross-device learning models that require cloud processing for core detection;
- supporting every niche keyboard layout in the initial rollout.

## 8. Functional Requirements

### FR-01: Multi-Language Detection
The product shall detect and classify the active language from recent keystroke sequences using a fast scoring model. Detection shall support at minimum:

- English (US/UK)
- Russian
- Spanish
- Portuguese (BR/PT)
- German
- French
- Italian
- Ukrainian
- Simplified Chinese (Pinyin)
- Japanese (Romaji)
- Hindi (ITRANS/InScript)
- Bengali

The language classifier shall operate as a low-latency decision engine with confidence thresholds that allow quick switching without excessive false positives.

Acceptance criteria:
- The classifier produces a ranked language candidate list within the input event processing window.
- A confidence delta above threshold is required before a layout switch is triggered.
- Mixed-language sessions do not force a layout switch after a single ambiguous token.

### FR-02: Keyboard Layout Switching
The product shall change the active system keyboard layout or input source when language confidence exceeds the configured threshold. Switching must respect the active operating system and input stack.

Acceptance criteria:
- Layout change occurs within the product latency target under normal system load.
- The user is not forced into a switch when the current language remains highly probable.
- Layout changes are suppressed or deferred during active IME composition when the system requires native composition.

### FR-03: Inline Auto-Correction
The product shall offer contextual correction for common typing mistakes in supported alphabetic languages. Corrections shall be based on language model scores, sequence validity, and a strict blacklist of illegal character transitions.

Acceptance criteria:
- Corrective actions are only triggered when the confidence score exceeds the rule threshold.
- High-risk corrections are rejected when they would alter user intent in ambiguous contexts.
- Corrective behavior is disabled when the user is in a high-stakes composition or code-writing context.

### FR-04: Dual-Layer Language Scoring
The product shall implement a dual-layer classifier combining:

1. Laplace-smoothed bigram/trigram log-probability scoring; and
2. illegal sequence hard-filter trie validation.

This model must be tunable per language pack and support lightweight runtime updates.

Acceptance criteria:
- Illegal sequences are rejected before probabilistic scoring is used for final selection.
- Model updates remain backward compatible with existing language pack contracts.
- Low-resource environments still produce acceptable classification performance.

### FR-05: IME-Aware Routing
The product shall recognize native composition contexts and avoid breaking OS-managed IME flows. The engine shall distinguish plain English text from Pinyin/Indic phonetic input and avoid direct rewriting during active composition windows.

Acceptance criteria:
- Pinyin and Indic typing is not mistaken for plain Latin typing when the OS composition stack is active.
- Native composition windows remain under OS control.
- IME passthrough mode preserves user-recognized input without introducing extra synthetic events.

### FR-06: Developer Mode
The product shall provide a developer mode that automatically suppresses correction and layout switching inside code blocks, strings, and other code-like contexts.

Acceptance criteria:
- A code editor or IDE integration can signal a code context block to the engine.
- Correction triggers are deferred or disabled in code contexts.
- The user can override the mode for specific windows or programs.

### FR-07: Smart Clipboard History
Premium users shall have access to smart clipboard history features that integrate with the same language-aware context model without introducing persistent disk logging.

Acceptance criteria:
- Clipboard snapshots are retained only within the in-memory session policy.
- Clipboard history is not written to disk by default.
- Clipboard entries can be purged on demand without corrupting the active input session.

### FR-08: Language Pack Modularity
The system shall support plug-and-play language packs that can be enabled, disabled, and updated independently. Tier 1 and Tier 2 pack types shall be supported with distinct routing and model behavior.

Acceptance criteria:
- A language pack can be added without rebuilding the core engine.
- Invalid or incompatible packs fail gracefully and are quarantined from runtime use.
- Pack metadata includes supported scripts, confidence ranges, and routing policy.

### FR-09: Cross-Platform Input Abstraction
The platform layer shall abstract input hook registration, layout switching, and synthetic event emission behind a common interface.

Acceptance criteria:
- The core engine depends only on platform abstraction interfaces.
- Platform-specific logic is isolated to the OS implementation modules.
- Failures in one platform integration do not crash the core detection engine.

### FR-10: Enterprise Controls
Enterprise deployments shall support policy management for zero-keystroke retention, user/group enforcement, and managed installation.

Acceptance criteria:
- Policies can be enforced through MDM or Group Policy.
- Zero-retention mode is enforced at the service design level.
- Enterprise config can disable features not required for the deployment.

### FR-11: User Override and Safety
Users shall be able to disable automatic switching, disable correction, or temporarily pause TypoMorph for a session. The product shall respect explicit user preferences.

Acceptance criteria:
- Manual override takes precedence over inference-based changes.
- User-level controls are immediately effective without requiring a restart.
- Unsafe or ambiguous actions are suppressed rather than auto-applied.

### FR-12: Logging and Privacy Controls
The product shall not persist raw keystrokes or user text to disk. Only ephemeral in-memory buffers are allowed for runtime processing.

Acceptance criteria:
- No disk logging occurs in default operation.
- Typed session buffers are bounded and discarded after use.
- Privacy mode disables cloud sync or analytics features by default.

## 9. Non-Functional Requirements

### NFR-01: Latency Budget
The system shall maintain a sub-1ms end-to-end processing target for the core keystroke classification path under steady-state conditions.

Requirements:
- the ring buffer must be fixed-size and zero-heap during ordinary operation;
- the scoring engine must avoid allocations on the critical path;
- platform bridging must be optimized to minimize event delays.

### NFR-02: Reliability
The product shall maintain a high availability posture for foreground typing sessions and shall recover from transient input failures without user-visible interruption.

Requirements:
- automatic re-registration of hooks after OS events or permission changes;
- graceful fallback to passive monitoring when a platform integration is temporarily unavailable;
- no crash loops during invalid keyboard layouts or composition state changes.

### NFR-03: Memory Discipline
The system shall not allocate dynamically on the hot path for ordinary key processing.

Requirements:
- ring buffer size capped at 32 tokens;
- probabilistic scoring performed on static or preallocated structures;
- volatile memory boundaries enforced to prevent retention beyond session time.

### NFR-04: Security and Trust
The product shall be designed to minimize the perception of a keylogger while satisfying OS compliance requirements.

Requirements:
- code signing and notarization compatibility for macOS;
- Windows hook behavior aligned to low-level input-safe patterns;
- Linux udev rules and user-space injection patterns that avoid unsafe privilege acquisition.

### NFR-05: Portability
TypoMorph shall run on supported Ubuntu X11/Wayland, Windows, and macOS systems without requiring fundamental architectural differences in the detection engine.

### NFR-06: Compliance Readiness
The product shall support compliance requirements for enterprise environments, including:

- SOC2-oriented control design;
- GDPR-aligned privacy processing;
- localized policy constraints for managed deployments;
- zero-retention configuration for enterprise customers.

## 10. Product Constraints and Assumptions

- The product is designed for real-time keyboard interaction, so critical-path performance is prioritized over exhaustive model complexity.
- OS input APIs differ in composition semantics; platform-specific routing is required.
- IME systems can block direct text mutation when composition is active; therefore behavioral guarding is required.
- Some code editors and terminals may require custom integration to fully suppress unwanted correction.
- Some enterprise customers may require strict privacy mode and local-only operation without cloud sync.

## 11. Edge-Case Matrix

| Scenario | Risk | Expected Behavior |
| --- | --- | --- |
| User types in English while Russian layout is active | False positive switch | Require confidence delta and sequence validity before switching |
| User enters Chinese Pinyin text | Misclassification as Latin English | Route through IME-aware passthrough and preserve native composition |
| User types in IDE with code strings | Incorrect auto-correction | Suppress correction inside code-like contexts |
| User alternates between languages mid-sentence | Rapid toggle churn | Use hysteresis and trailing context buffer to avoid oscillation |
| User enters accented characters or dead keys | Invalid sequence misclassification | Accept valid sequences and delay correction during composition |
| User toggles layout manually | Engine override conflict | Manual input mode takes precedence over auto-switching |
| User switches app focus during typing | Stale event context | Flush transient event buffer and reinitialize session context |
| User enters text in terminal with shell escapes | Erroneous correction | Detect non-editing contexts and suppress mutation |
| User uses an IME candidate selection menu | Interference with composition | Do not synthesize replacements during active IME selection |
| User copies large text or clipboard content | Buffer contamination | Ignore clipboard-driven path unless explicitly enabled in premium mode |
| User types rapidly with mixed scripts | Sequence ambiguity | Provide conservative fallback; defer correction until confidence rises |
| User has multiple keyboard layouts installed | Wrong target layout chosen | Map active language to exact configured layout and verify OS state |

## 12. Error Recovery and State Machine

### 12.1 State Model

The runtime shall maintain a bounded state machine with explicit transitions for error and recovery handling.

State definitions:

- Idle: no active typing sequence or active detection event.
- Monitoring: collecting a bounded keystroke window and computing scores.
- Candidate: a language candidate set has been produced and confidence is under evaluation.
- Switching: platform-specific layout change is being requested.
- Composing: native IME composition is active; corrections are suppressed.
- Correction: a probable correction is being validated before application.
- Fallback: the engine cannot confidently determine a target language and reverts to current layout.
- Paused: user-disabled or enterprise policy disabled.
- Error: repeated hook failures, invalid OS state, or integration fault.
- Recovery: re-registering hook, reinitializing ring buffer, or re-establishing platform permissions.

### 12.2 Transition Rules

1. Idle -> Monitoring when input events are observed.
2. Monitoring -> Candidate when enough evidence exists to rank languages.
3. Candidate -> Switching when confidence delta exceeds threshold and no IME conflict exists.
4. Candidate -> Fallback when confidence is weak or ambiguous.
5. Monitoring/Correction -> Composing when the OS reports active composition or IME window.
6. Any active state -> Paused when user override or policy disables the engine.
7. Any state -> Error on invalid platform state, repeated hook failure, or unsupported layout.
8. Error -> Recovery where the system re-establishes the input bridge and clears volatile buffers.
9. Recovery -> Idle after a successful re-initialization.

### 12.3 Recovery Policies

- Clear the ring buffer on unrecoverable ambiguity.
- Re-register OS hooks after permission changes or sleep/resume events.
- Suppress corrective actions when OS composition state is unstable.
- Prefer degraded functionality over unsafe language switching.
- Log only operational failures internally and never raw keystrokes.

## 13. Acceptance Criteria Summary

The product is considered acceptable when:

1. It accurately handles supported direct alphabetic languages with minimal false switches.
2. It preserves native IME composition behavior for Chinese, Japanese, Hindi, and Bengali workloads.
3. It maintains typing responsiveness within the performance target.
4. It disables unsafe corrections in high-risk contexts such as code editing and active composition.
5. It does not write raw keystrokes or user content to disk.
6. It supports platform-specific integrations on macOS, Windows, and Linux.
7. It provides enterprise-grade policy controls and privacy mode.

## 14. Risks and Mitigations

### 14.1 Risk: False-positive layout switches
Mitigation: use a confidence delta, hysteresis, and a minimum context window before switching.

### 14.2 Risk: IME destruction or composition breakage
Mitigation: treat IME windows as high-priority composition states and route through passthrough logic.

### 14.3 Risk: Over-correction during typing bursts
Mitigation: require strong score confidence and legal sequence validation before applying edits.

### 14.4 Risk: Privacy complaints
Mitigation: limit all processing to volatile memory and avoid disk persistence by default.

### 14.5 Risk: OS-specific API instability
Mitigation: isolate platform logic behind stable interfaces and include graceful failure recovery.

## 15. Open Questions

- Which first-wave language packs are mandatory versus optional for launch?
- What level of IDE integration is required for the initial version: generic suppression, editor-specific hooks, or both?
- How should the premium clipboard history feature be governed for enterprise zero-retention standards?
- Which Linux desktop environments will be supported in the first GA release?

## 16. Definition of Done

TypoMorph shall be considered ready for implementation planning when:

- the language scope and pack priorities are approved;
- the platform abstraction contracts are stable;
- privacy constraints and retention policies are explicitly documented;
- the detection and correction confidence rules are accepted by product stakeholders; and
- the developer mode and IME-safe behavior criteria are approved for beta validation.
