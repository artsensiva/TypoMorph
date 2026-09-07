# TypoMorph Architecture Blueprint

## 1. Architectural Summary

TypoMorph is structured as a layered, cross-platform input intelligence system with a low-latency core decision engine and OS-specific adapters. The architecture separates runtime classification from platform integration so that language detection, scoring, and policy logic remain portable while native keyboard and IME behavior remains platform-specific.

The design follows a strict hot-path rule: the critical keystroke processing path must remain allocation-free, bounded, and resistant to persistence. All user-typed data is retained only in ephemeral in-memory buffers during active processing.

## 2. Architectural Principles

### 2.1 Deterministic Low-Latency Core
The central engine prioritizes performance over broad model complexity. The best possible behavior is achieved by staying in a bounded execution envelope:

- fixed-size ring buffer for recent input;
- constant-time scoring operations;
- no heap allocation in the keystroke hot path;
- confidence-based gating before layout switching or correction;
- strongly typed platform abstraction boundaries.

### 2.2 Platform Isolation
The product uses OS-specific adapter modules for keyboard hooks, layout switching, and composition control. The core engine never directly calls platform APIs.

### 2.3 IME Safety First
IMEs are treated as privileged user input contexts. Where composition is active, TypoMorph reduces risk by deferring switching and suppressing mutation.

### 2.4 Privacy by Design
The system is intentionally designed to avoid raw keystroke persistence. Any temporary state is bounded, volatile, and removed after the relevant input window is resolved.

## 3. Cargo Workspace Topology

The package layout is organized as a Rust workspace with explicit responsibilities:

```text
TypoMorph/
├── Cargo.toml
├── crates/
│   ├── core-engine/
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lang-packs/
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   └── packs/
│   ├── platform-macos/
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── platform-windows/
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── platform-linux/
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── daemon/
│   │   ├── src/
│   │   └── Cargo.toml
│   └── common/
│       ├── src/
│       └── Cargo.toml
├── docs/
│   ├── SPEC.md
│   ├── ARCHITECTURE.md
│   └── ROADMAP.md
└── scripts/
```

## 4. Component Responsibilities

### 4.1 core-engine

The core engine provides the central detection and decision logic for the product.

Responsibilities:

- maintain the ring buffer and recent token history;
- compute n-gram language likelihoods;
- enforce illegal sequence filters;
- decide when confidence is sufficient for switching;
- apply correction gating and policy checks;
- expose platform-neutral interfaces for command decisions.

This crate must be portable, fast, and free from OS-specific runtime dependencies.

### 4.2 lang-packs

This crate hosts all language-model definitions and pack metadata. It organizes language support by tier and behavior class.

Responsibilities:

- maintain direct alphabetic language pack definitions;
- maintain IME-aware hybrid routing metadata;
- declare legality rules and script constraints;
- expose model parameters for scoring and fallback behavior;
- allow runtime pack validation and safe loading.

Language packs are likely to be structured as:

- pack metadata
- tokenizer / script classifier
- legal-sequence trie or transition table
- bigram/trigram model
- confidence tuning values
- OS routing policy

### 4.3 platform-macos

This crate implements macOS-specific input hooks and state discovery.

Responsibilities:

- use CGEventTap or low-level event capture where permitted;
- query and change input source state;
- detect composition state and IME contexts;
- handle accessibility features where required;
- inject synthetic text or layout changes in a way consistent with macOS input semantics.

### 4.4 platform-windows

This crate implements Windows-specific hooks and layout mating logic.

Responsibilities:

- register low-level keyboard hooks with SetWindowsHookExW or equivalent call patterns;
- detect active layout and composition windows;
- use SendInput and WM_INPUTLANGCHANGEREQUEST for layout change orchestration;
- preserve behavior during Chinese, Japanese, and Indic IME composition;
- adapt to layout changes and user overrides.

### 4.5 platform-linux

This crate implements the Linux input abstraction across X11 and Wayland.

Responsibilities:

- read from /dev/input/event*;
- inject events through /dev/uinput;
- integrate with GNOME Shell D-Bus, Fcitx, and IBus where supported;
- respect user-session policy and udev rules;
- maintain fallback behavior for unsupported or partial desktop environments.

### 4.6 daemon

The daemon acts as the runtime orchestrator and controller.

Responsibilities:

- install and manage OS event hooks;
- maintain global state and config;
- coordinate platform adapters through a unified runtime interface;
- enforce user preferences and enterprise policy;
- manage lifecycle, failure recovery, and thread boundaries.

### 4.7 common

The common crate contains cross-platform contracts and shared abstractions.

Responsibilities:

- define the event protocol and state transitions;
- define the policy model and configuration schema;
- expose safe shared types for scoring decisions;
- encapsulate privacy and retention rules.

## 5. Runtime Control Flow

### 5.1 Event Capture Path

1. A platform adapter captures keyboard or input events.
2. Input is normalized into a cross-platform event representation.
3. The event stream is validated for composition and context safety.
4. The ring buffer receives the event tokens. 
5. The core engine updates the probabilistic score.
6. Candidate language confidence is computed.
7. The system decides whether to switch layout, suppress correction, or continue monitoring.

### 5.2 Decision Path

The decision pipeline is:

- monitor input window;
- score candidate languages;
- validate legal sequences;
- check IME/composition state;
- evaluate confidence delta and hysteresis;
- apply user or policy override;
- perform platform switch or text mutation only when safe.

### 5.3 Correction Path

Correction is not a blind rewrite. It is a guarded, confidence-driven action:

- language candidate selected;
- token sequence evaluated for legal transitions;
- correction candidate built with minimal edit distance;
- code-context and IME-context checks performed;
- only then is the replacement emitted.

## 6. Memory Layout and Hot-Path Design

The critical processing path uses a bounded, static memory profile:

```text
+---------------------------+
| ring buffer (32 tokens)    |
| fixed-size event queue     |
+---------------------------+
| current script / layout    |
| active confidence scores   |
| legal sequence state       |
+---------------------------+
| platform event metadata    |
| composition state flag     |
| user override state        |
+---------------------------+
| temporary correction buf   |
| bounded, volatile only     |
+---------------------------+
```

### Key Rules

- no dynamic heap allocation in the hot path;
- all user data is ephemeral and tied to active session scope;
- input buffers are cleared on fallback or shutdown;
- no raw keystroke archive is maintained;
- cloud sync is explicitly opt-in and separated from the core engine.

## 7. Language Detection and Scoring Design

### 7.1 Tier 1: Direct Alphabetic Matrix

Supported languages in this tier include:

- English (US/UK)
- Russian
- Spanish
- Portuguese (BR/PT)
- German
- French
- Italian
- Ukrainian

The scoring engine uses:

- Laplace-smoothed bigram and trigram probabilities;
- probabilistic ranking across recent token windows;
- deterministic legal sequence validation;
- confidence delta thresholds to avoid unnecessary toggles.

### 7.2 Tier 2: IME-Aware Hybrid Routing

The hybrid tier supports:

- Simplified Chinese via Pinyin
- Japanese via Romaji
- Hindi via ITRANS/InScript
- Bengali

This tier introduces an explicit routing distinction:

- plain Latin entry is classified as direct alphabetic input;
- phonetic entry is marked as IME-aware input;
- native composition state is respected before any mutation or switch;
- the system avoids disrupting the OS input method or composition engine.

## 8. IME Routing Logic

### 8.1 General Policy

The system should not rewrite text while the operating system is actively composing text. IME routing logic enforces a three-level gate:

1. Detect whether the active application is in IME composition mode.
2. Detect whether the text stream matches phonetic or script-aware patterns.
3. Allow native OS control for composition while only logging or scoring the token stream in volatile memory.

### 8.2 macOS

The macOS adapter should detect and preserve:

- Text Input Source switching;
- composition state from the input stack;
- accessibility-safe checks for layout transitions.

### 8.3 Windows

The Windows adapter should preserve:

- native IME composition ownership;
- active keyboard layout mapping;
- text input composition boundaries before emitting synthetic changes.

### 8.4 Linux

The Linux adapter should preserve:

- X11 or Wayland input semantics;
- GNOME Shell, Fcitx, and IBus compatibility;
- general user-space input control without requiring unsafe privileges.

## 9. Security and Privacy Architecture

### 9.1 Privacy Hardening

TypoMorph follows a strict privacy model:

- zero disk logging by default;
- volatile in-memory processing only;
- reduction of raw token retention to bounded session state;
- no persistent keyboard analytics store;
- explicit opt-in for cloud sync features.

### 9.2 Trust and Safety Considerations

The design aims to avoid keylogger-like behavior by ensuring:

- kernel-level hooks are limited to the necessary event surface;
- critical state remains in the application runtime, not a broad telemetry system;
- installation and device integration support signing and notarization requirements;
- enterprise configuration can disable telemetry or cloud features entirely.

## 10. Failure Handling and Recovery

The system includes a lifecycle-safe recovery model for platform-level failures.

### Recovery principles

- if hook registration fails, the system remains passive and reports the error without crashing;
- if a layout request fails, the system falls back to the last known stable layout;
- if IME composition is uncertain, corrections are suppressed;
- if a user override is active, automatic actions are disabled immediately;
- if a platform integration becomes unstable, the daemon re-registers or reinitializes the adapter without disrupting the session.

## 11. Enterprise and Policy Architecture

The daemon integrates policy enforcement and admin controls for enterprise environments.

### Policy dimensions

- zero-keystroke retention mode;
- allowlisted or denylisted apps for correction;
- managed layout switching rules;
- MDM or Group Policy deployment support;
- custom dictionary and cloud sync toggles.

Policies are enforced centrally through the daemon and are not bypassed by language packs or app-level plugins.

## 12. External Interfaces

### 12.1 Core Interfaces

The core engine exposes the following abstract interfaces:

- InputEventSource
- LayoutDecision
- LanguageModel
- CompositionGuard
- CorrectionPolicy
- SessionState

### 12.2 Platform Adapters

Each OS adapter implements the same interface contract but with different runtime details.

### 12.3 Configuration API

The daemon exposes configuration for:

- enabled language packs;
- thresholds and hysteresis values;
- developer mode toggles;
- enterprise permissions and retention settings;
- user override and pause controls.

## 13. Scalability and Extensibility

The architecture is built to support incremental growth:

- add language packs without changing the scoring interface;
- extend policy layers without breaking the core engine;
- add new platform adapters behind the same abstraction;
- support premium features like clipboard history or cloud sync as optional modules.

This ensures the product can evolve from a focused utility into a broader multilingual input platform without redesigning the foundation.

## 14. Key Design Decisions

### Decision 1: Bounded memory and allocation-free hot path
This is the foundation of both performance and privacy.

### Decision 2: Separation between core engine and OS adapters
This prevents platform differences from leaking into the detection model.

### Decision 3: IME-first safety gates
This ensures TypoMorph does not disturb native composition behavior.

### Decision 4: Conservative, confidence-driven correction
This reduces false positive rewrites and is essential for credibility in professional scenarios.

## 15. Architecture Acceptance Criteria

The architecture is accepted when:

- the core engine remains portable and independent from OS APIs;
- the hot path is bounded and does not allocate in the steady-state keystroke flow;
- the platform adapters implement the same abstractions across macOS, Windows, and Linux;
- language packs are modular and independently loadable;
- the IME routing logic prevents composition disruption;
- enterprise policy enforcement is available without altering the core decision engine.

## 16. Future Architecture Evolution

The next architectural extensions may include:

- a tray or background service UX layer;
- richer IDE and app-context detection integration;
- optional premium sync and dictionary management modules;
- more advanced script-aware scoring for additional language families;
- observability and diagnostic tooling for enterprise support teams.

These additions should remain modular so they do not compromise the performance and privacy guarantees of the core product.
