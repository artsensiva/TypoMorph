# ROLE & OBJECTIVE
You are a Principal Systems Architect, Rust Systems Engineer, and Commercial Software Strategist.
We are building a commercial, ultra-low-latency (<1ms), cross-platform intelligent input utility (layout switcher & auto-correction engine) named "TypoMorph" running natively on macOS, Windows, and Linux (Ubuntu X11/Wayland).

We operate strictly under the Spec-Kit / Spec-Driven Development (SDD) framework.
Do NOT output application code yet. First, produce the complete Product Specification and Architecture blueprints.

---

### 1. GLOBAL MARKET REVENUE & MONETIZATION
- Target Markets: Global knowledge workers, developers, multilingual writers, enterprises (B2C Pro licenses + B2B team seats).
- Revenue Model:
  - Free Core: Fast 2-language heuristic auto-switching.
  - Pro Tier ($39 lifetime or $4/month): Unlimited language packs, developer IDE mode (auto-disables inside code blocks / strings), cloud sync of custom dictionaries, smart clipboard history.
  - Enterprise Tier: Zero-Keystroke Retention guarantee, SOC2/GDPR compliance, fleet deployment policies via MDM / Group Policy.

---

### 2. MULTI-TIER LANGUAGE ENGINE (GLOBAL COVERAGE)
The core engine must support modular plug-and-play language packs:
1. Tier 1 (Direct Alphabetic Matrix):
   - English (US/UK), Russian, Spanish, Portuguese (BR/PT), German, French, Italian, Ukrainian.
   - Dual-layer classifier: Laplace-smoothed bigram/trigram log-probabilities + Illegal Sequence Hard-Filter Trie.
2. Tier 2 (IME-Aware Hybrid Routing):
   - Simplified Chinese (Pinyin), Japanese (Romaji), Hindi (ITRANS/InScript), Bengali.
   - Smart IME Passthrough: Detection of Pinyin/Indic phonetics versus English plain text without breaking native OS Composition Windows.

---

### 3. CROSS-PLATFORM SYSTEM ABSTRACTIONS
- Core (Rust):
  - Ring buffer (ephemeral, 32 tokens max, zero heap allocations on keystroke).
  - Fast probabilistic language scorer with confidence threshold delta.
- Platform Input Hooks & Synthetic Emitters:
  - macOS: CGEventTap / IOHidManager + Carbon TIS (Text Input Source) switching + Accessibility API.
  - Windows: Low-Level Keyboard Hook (`SetWindowsHookExW`), `SendInput` API, and `PostMessage` WM_INPUTLANGCHANGEREQUEST.
  - Linux: Kernel-level `/dev/input/event*` via `evdev` + injection via `/dev/uinput` + GNOME Shell D-Bus / Fcitx / IBus switcher (Wayland and X11 support via non-root udev rules).
- Antivirus & Privacy Hardening:
  - RAM-only volatile buffer. Zero disk logging.
  - Clear architectural posture against keylogger heuristics (EV Code Signing, Apple Notarization compatibility).

---

### 4. SPEC-KIT DELIVERABLES REQUIRED
Generate the following formal documents in Markdown:
1. `SPEC.md`: Detailed functional, non-functional requirements, edge-case matrices, and error recovery state machine.
2. `ARCHITECTURE.md`: Cargo workspace topology (`core-engine`, `lang-packs`, `platform-macos`, `platform-windows`, `platform-linux`, `daemon`), memory layout, and IME routing logic.
3. `ROADMAP.md`: Phased execution plan from synthetic benchmarks to OS-specific bindings and GUI/tray integration.

Begin by outputting the comprehensive `SPEC.md`.GEMINI