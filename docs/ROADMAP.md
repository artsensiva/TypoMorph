# TypoMorph Product Roadmap

## 1. Objective

This roadmap provides a phased execution plan from the initial proof-of-concept and synthetic benchmarks through operating-system bindings, packaging, developer mode, and enterprise readiness. The goal is to validate the core language engine and privacy model before expanding to OS-specific integrations and user-facing features.

## 2. Guiding Principles

- Validate the latency and privacy model before adding broad feature surface.
- Build the detection engine around robust but conservative confidence thresholds.
- Prioritize IME safety and developer-mode suppression over aggressive correction.
- Keep platform adapters isolated behind stable contracts.
- Treat enterprise deployment and compliance as a late-stage but essential product milestone.

## 3. Phase 0: Product Definition and Benchmark Design

### Goal
Establish the technical baseline and validate the feasibility of the core architecture.

### Scope

- finalize supported language scope and pack priorities;
- define the runtime latency budget and benchmark harness;
- document engine contracts and privacy boundaries;
- identify target OS-specific APIs and compatibility constraints;
- align the product on a zero-disk, volatile-memory processing model.

### Deliverables

- benchmark specification for classification latency;
- language pack prioritization matrix;
- initial architecture contract for event capture and decision flow;
- acceptance criteria for confidence thresholds and fallback handling.

### Exit Criteria

- all critical scenarios are traceable to product requirements;
- initial hot-path architecture has a bounded memory model;
- benchmark plan includes direct alphabetic and IME-aware cases.

## 4. Phase 1: Core Engine and Direct Alphabetic Language Packs

### Goal
Implement the fast, stateless detection engine and support the first wave of direct alphabetic languages.

### Supported Languages

- English (US/UK)
- Russian
- Spanish
- Portuguese (BR/PT)
- German
- French
- Italian
- Ukrainian

### Scope

- implement the ring buffer and input normalization layer;
- build bigram/trigram scoring with Laplace smoothing;
- implement the illegal sequence hard-filter trie;
- support language confidence scoring and hysteresis logic;
- validate switching rules for multilingual sessions;
- build deterministic fallback logic for low-confidence states.

### Deliverables

- core scoring engine with configurable thresholds;
- first-pass direct alphabetic language pack registry;
- internal benchmark harness for scoring and switch behavior;
- engine unit tests for illegal sequence rejection and false-positive suppression.

### Exit Criteria

- the engine can classify the initial supported languages with acceptable confidence separation;
- false switches are controlled by hysteresis and threshold logic;
- no heap allocations occur in the measured hot path.

## 5. Phase 2: OS Bindings and Native Input Integration

### Goal
Connect the portable engine to each supported operating system with minimal platform-specific risk.

### Scope

#### macOS

- CGEventTap or equivalent event bridge;
- Text Input Source switching and layout detection;
- composition state and accessibility compatibility checks.

#### Windows

- low-level keyboard hook integration;
- SendInput and WM_INPUTLANGCHANGEREQUEST integration;
- composition guard logic for active IME windows.

#### Linux

- /dev/input/event* capture and event normalization;
- /dev/uinput injection support;
- GNOME Shell D-Bus, Fcitx, and IBus support paths;
- X11 and Wayland compatibility model.

### Deliverables

- three platform adapter crates with standard runtime interfaces;
- common daemon that orchestrates adapter registration and state transitions;
- OS-specific fallback policy for failure and permission changes.

### Exit Criteria

- the same core engine is successfully used across all three platforms;
- switch and detection behavior is portable across OS differences;
- composition state does not break native IME workflows.

## 6. Phase 3: IME-Aware Hybrid Routing

### Goal
Support phonetic and IME-driven languages without breaking native composition behavior.

### Scope

- add Pinyin classification support;
- add Romaji routing for Japanese input;
- add Indic input handling for Hindi and Bengali;
- build explicit IME passthrough paths and composition guards;
- detect script transitions between Latin and phonetic input streams.

### Deliverables

- IME-aware language pack definitions;
- hybrid routing decision logic;
- project-specific integration policies for composition-safe behavior;
- benchmark coverage for ambiguous switching, phonetic sequences, and composition windows.

### Exit Criteria

- Pinyin and Indic workflows operate without breaking active OS composition;
- no direct text mutation occurs during native IME composition;
- switch decisions are conservative when the system cannot confidently separate plain text from phonetic input.

## 7. Phase 4: Developer Mode and Editor Safety

### Goal
Protect professional, code-heavy workflows from unsafe corrections or layout churn.

### Scope

- code context detection for IDEs, terminals, and editor shells;
- mode toggles for correction suppression and layout pause;
- app-specific exclusions or allowlists;
- safe behavior inside strings, comments, and code blocks.

### Deliverables

- developer mode configuration model;
- app-context detection framework;
- code-aware suppression rules;
- UI controls for user overrides and workspace-specific exceptions.

### Exit Criteria

- code-writing experiences remain stable without accidental auto-correction;
- developers can override suppression or enable a stricter policy mode;
- editor boundaries do not cause false-positive switching.

## 8. Phase 5: Premium Features and User Experience Layer

### Goal
Introduce advanced functionality for the Pro tier while preserving zero-disk and low-latency design goals.

### Scope

- smart clipboard history integration;
- unlimited language pack support;
- cloud sync for custom dictionaries; 
- configuration UI or tray controls;
- user onboarding and first-run experience.

### Deliverables

- premium feature gating and entitlement system;
- optional cloud sync service with explicit privacy controls;
- tray/menubar UI and settings panel;
- session controls for pause, override, and safe mode.

### Exit Criteria

- premium features remain optional and segregated from the core engine;
- no default disk persistence or raw keystroke storage is introduced;
- the UX remains lightweight and nonintrusive for ordinary users.

## 9. Phase 6: Enterprise Hardening and Compliance

### Goal
Prepare the product for enterprise deployment at scale with policy and compliance controls.

### Scope

- zero-keystroke-retention policy model;
- MDM and Group Policy support;
- fleet deployment package management;
- SOC2 and GDPR alignment for data handling and architecture;
- auditability of policy enforcement and configuration.

### Deliverables

- enterprise configuration schema;
- legal and privacy review support documentation;
- deployment docs for managed environments;
- support tooling for policy validation and troubleshooting.

### Exit Criteria

- enterprise policy mode can fully disable retention and cloud features;
- deployment is manageable across managed workstations;
- product posture is compatible with zero-retention deployment requirements.

## 10. Phase 7: Beta, GA, and Expansion

### Goal
Move from technical validation to confident product maturity and market launch.

### Scope

- beta testing across supported OSes and major workflows;
- user feedback validation for false switching and correction noise;
- packaging, installer, and update pipeline readiness;
- launch support for B2C and team-based licensing.

### Deliverables

- beta release candidate;
- documentation for installation, troubleshooting, and privacy guarantees;
- launch-grade controls for user override, compliance mode, and updates;
- GA readiness checklist signed by technical, product, and compliance stakeholders.

### Exit Criteria

- product demonstrates stable behavior in real-world multilingual use cases;
- user-facing latency and reliability meet target expectations;
- support and privacy controls are ready for public deployment.

## 11. Release Gate Criteria

Each phase is considered complete only when the following are satisfied:

- core scenarios pass automated benchmark validation;
- no regression in deterministic latency behavior;
- false-positive and over-correction rates remain below acceptable product thresholds;
- IME safety is preserved across target systems;
- privacy and retention guarantees remain intact;
- enterprise controls remain effective when enabled.

## 12. Risk Review by Phase

### Phase 1 risk
High risk of false switching in mixed-language typing. Mitigation: conservative thresholds and strict fallback behavior.

### Phase 2 risk
Platform API mismatches across OSes. Mitigation: stable adapter contracts and fallback capabilities.

### Phase 3 risk
IME composition disruption. Mitigation: composition-first routing and passive-only monitoring during active IME states.

### Phase 4 risk
Over-correction in code workflows. Mitigation: strong editor and terminal context suppression.

### Phase 5 risk
Premium features unintentionally increase privacy issues. Mitigation: keep cloud sync opt-in and segregated from default operation.

### Phase 6 risk
Enterprise policy complexity. Mitigation: enforce policy at the daemon boundary and test under managed deployment scenarios.

## 13. Recommended Sequencing

The recommended execution order is:

1. Phase 0: benchmark and architectural baseline
2. Phase 1: direct alphabetic engine
3. Phase 2: OS bindings
4. Phase 3: IME-aware routing
5. Phase 4: developer-safety mode
6. Phase 5: premium UX and features
7. Phase 6: enterprise hardening
8. Phase 7: beta and GA

This sequence minimizes the risk of shipping a feature-rich product before the core engine and privacy design are proven.

## 14. Definition of Done for the Full Product

TypoMorph is ready for broad launch when:

- the direct alphabetic engine is validated and stable;
- IME-aware language routing is safe across supported platforms;
- correction behavior is conservative and user-friendly;
- platform integration is reliable on macOS, Windows, and Linux;
- enterprise zero-retention and deployment controls are available;
- the product demonstrates privacy-first behavior by design rather than by exception.
