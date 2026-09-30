# TypoMorph project state

Updated: 2026-10-01. Branch: `diagnostic/device-echo-investigation`.

## Phase and authority

Linux safety/integration feasibility (P1–P3), with independent P6–P8 components
in progress. **Version 1 is not complete or release-ready.** The owner requests
autonomous implementation within the approved specification, without repeated
routine approval questions. No scope/privacy/commercial reduction is authorized.
The owner authorized a development checkpoint commit and GitHub push on
2026-10-01; this does not authorize a release, website deployment or live payments.
Work since baseline `e42e77c` is being checkpointed on the current branch.
Preserve unrelated `.htaccess` and `TypoMorph.code-workspace`, excluded from that
checkpoint. English-only workflow; see AGENTS.md.

## Confirmed baseline

- Local wrong-layout correction during word entry, manual repair, safe undo;
  no AI/spelling feature, clipboard fallback, input transmission or persistence.
- Six input/UI languages: EN/RU/UK/DE/FR/ES; exact layouts in COMPATIBILITY.md.
  Only explicitly selected supported installed layouts; no automatic enrollment.
- Ubuntu GNOME Wayland/X11 x86-64, Windows 11 x86-64, macOS Intel/Apple Silicon.
  Native browser coverage first; extensions conditional, Safari extension deferred.
- Protected/unknown/IME/excluded contexts refuse analysis; pause stops collection.
  Sounds optional and off by default; autostart requires consent.
- Verified-email account, shared seven-day card-free trial, three installations.
  USD 7/year with explicit purchased renewal or USD 19 perpetual; equal features
  and all future released updates. Confirmed rights work offline; accepted limits
  on revoking offline licenses. Stripe account separate from the owner's project,
  USD taxes-inclusive pricing; optional dated EUR display on the website only.
- No fixed release date or silent scope reduction. Stores/additional repositories,
  extra languages and other deferred features remain as specified in SPEC.md.

## Completed in the working copy

- Unsafe production `run` and uinput replacement refuse before capture/mutation.
  The installed 0.2.2 application and its permissions have not been replaced.
- Built AI/prompt modules, cloud dependencies and legacy license CLI are removed.
  Historical AI sources remain excluded/uncompiled. Recognized developer contexts
  are excluded independently of payment state.
- Native host is readiness-only with 4 KiB frames and content-free errors.
  Extension sources request only Native Messaging, with no page/content access.
- Ed25519 offline verifier implements bounded signed tokens, trusted keys,
  account/device binding, trial/annual/perpetual periods and signed renewal grace.
  No production keys, activation/account service, Stripe integration or clock
  rollback protection yet; daemon does not consume entitlements yet.
- New settings crate uses locked atomic writes, strict version/size validation,
  private Unix file mode, and no input/credentials. CLI pause/resume/sounds persist.
  Full GUI, sound playback and autostart integration remain pending.
- Diagnostic evdev queue bounded at 256 events. Pause joins/closes readers and
  clears state; resume requires fresh metadata. Overflow/device loss stops the
  stream. Settings polled every 100 ms, not an acknowledged instantaneous pause.
  Lifecycle tests are device-free; real hardware teardown timing remains untested.
  `--dry-run` is absent from normal release builds, available in debug builds only.
- Token/buffer/layout-decision/raw-event Debug output is redacted.
- Unknown physical layout IDs now abstain instead of aliasing US; candidate
  target IDs match the actual conversion table, and invalid thresholds abstain.
  Exact six-language mapping support remains incomplete.
- Installer requires explicit installation, exact Sigstore workflow/tag identity
  and package hash; no verification bypass or autostart. New Debian packaging
  omits broad udev rules. Release automation prepares drafts only. No artifact
  signed or installed; tests use fake external commands (DISTRIBUTION.md).

## Safety evidence and unresolved core bug

BUG-001 remains open: ordinary Text Editor/Chrome correction is not working.
The GNOME layout-only bridge was installed and switching verified, but AT-SPI
metadata cannot establish atomic field ownership. Distinct fields share IBus
contexts. Private native Wayland testing passed four cases and failed five,
including wrong-target draft/delayed-key delivery. See NATIVE_WAYLAND_EXPERIMENT.md.
A client-owned GTK transaction fixture passed 20 cases; that does not fix ordinary
apps. See CLIENT_TRANSACTION_EXPERIMENT.md and NATIVE_OWNER_API_AND_PATCH.md.

Native Text Editor 50.1 experiment is now **compiled and runtime-tested** under
owner-approved dependency/build authorization. Official upstream commit:
`d1bc58ff790267224be1536a86e1af236a9f7ea7`. Baseline and patched GCC builds pass;
baseline startup smoke and two upstream metadata tests per build pass.

Real private Wayland results: fixed `ghbdtn -> привет` plus ordinary undo/redo
works; a begin-action callback edit is preserved by pre-deletion refusal. Both
post-deletion competing-edit cases reproduce unsafe partial edits (exit 2),
including a real GTK callback. Production safety is NOT established. Runtime
validation fixed spurious invalidation on undo-availability notifications only.
The runner, fixed-case driver and content-free assessment are exercised; ten
Python tests and three patch-verifier regressions pass. Full evidence and exact
versions/hash: NATIVE_TEXT_EDITOR_RESULTS.md. System dependencies are installed;
additional itstool/Python binding are extracted only into /tmp. No sudo blocker
remains for these builds. No installed application was replaced.

Follow-up toolkit API probe: GtkSourceView 5.18.0 search replacement succeeds on
the normal case with ordinary undo/redo, but reports success after a vetoed
insertion (original lost) or deletion (duplicated text). A post-delete callback
also sees intermediate empty state. GTK commit notifications track individual
edits, not a combined transaction. Four fixed-buffer cases compiled with strict
warnings and ran without diagnostics (exits 0, 2, 2, 2); these are synthetic toolkit
tests, not additional real-app evidence. See GTK_REPLACEMENT_API_RESULTS.md.
The owner approved an isolated toolkit transaction prototype on 2026-09-30,
extending the earlier app-only patch boundary. No installation, production
enablement, upstream submission or permanent fork is authorized by that approval.

First toolkit storage prototype now builds against a separate GTK 4.22.4 source
tree. A bounded same-character-count segment splice preserves marks and emits
change notifications after storage commit; callbacks can append/delete without
losing their edits, and nested splices refuse. All 29 synthetic test groups passed
with ASan/UBSan, leak detection and GTK tree checks; no runtime diagnostics.
Two unchanged upstream CSS/sort objects needed a sanitizer-build warning exception;
new source/test checks stayed strict. Source preparation reproduces byte-for-byte.
Ordinary undo/redo now uses one replacement history entry, preserving savepoints,
surrounding history and callback edits. Refused reversal preserves its entry.
Targeted reversal now preserves later disjoint edits in the restricted buffer.
A bounded, latest-correction record tracks up to 256 actual edits; overlap,
untracked mutations, cancellation and supersession invalidate its token. Reversal
is a new chronological undo entry. Pending ordinary mutations refuse reentrant
transactions. Field/focus authorization and cancellation wiring remain pending.
The primitive
refuses views, subclasses, tags, legacy edit handlers and commit observers.
A private Broadway GtkTextView/layout probe confirmed attached-view refusal of
correction and reversal, with both succeeding after detachment. Its earlier leak-enabled
run reported Fontconfig/Pango allocations; behavioral checks passed with leak
detection disabled and ASan/UBSan retained. This does NOT fix real applications. Evidence,
limits and hashes: integrations/gnome/gtk-transaction/README.md. System GTK unchanged.

2026-10-01 layout investigation: GtkTextLayout registers four legacy insert/delete
callbacks even without GtkTextView. A cache-invalidation-only draft still refused;
the draft was removed and the verified six-file patch restored byte-for-byte.
Two new standalone layout tests prove correction/reversal refusal with warmed
Pango caches, no mutation, and success after layout detachment. ASan/UBSan pass
with process-exit leak checks disabled for the recorded font-stack limitation.
Next requires a compatible layout observer contract, plus GtkTextView accessibility
notifications; do not bypass legacy-handler or attached-view gates.

PolterType review pinned to `88709a6efd168a4a2f47382b7b224ebd76cb3573` is complete
(POLTERTYPE_REVIEW.md). Global key replay does not meet approved field safety;
no PolterType code/data was copied or executed. No live typing captured this turn.

## Latest verification

- Rechecked 2026-10-01: 81 Rust tests passed; two private-bus tests excluded from default
  run then passed separately using isolated synthetic GJS services.
- Strict workspace/all-target clippy and formatting passed.
- Optimized workspace build passed; 21 focused release tests passed, including
  refusal of the live-input diagnostic flag. Linker warns about a deprecated
  optimization setting; no build/test failure.
- Five installer orchestration tests passed with mock network/signing/install
  commands, plus ten native-runner/assessment Python tests. Browser popup JavaScript
  syntax and Git whitespace checks passed. All 29 GTK buffer/history tests were
  rerun with ASan/UBSan and leak checks; layout/view tests retain the stated limits.
- Landing page now presents development status and approved planned pricing;
  old checkout, install and download links are removed. README is updated.
  Desktop/mobile Chromium previews and local fragment-link checks passed. No
  website deployment was performed; source publication is a separate Git step.
- These results do not certify real app correction, release packages or payments.

## Next work and release blockers

Owner's local test request: Ubuntu version is confirmed as 26.04.1 LTS / GNOME
50 / Wayland (supersedes `24.06`); Windows 10 Home is also available. Windows 10
testing is experimental, not a change to the approved release support matrix.
Safe commands and limitations: LOCAL_MACHINE_TESTING.md. Linux core tests were
rerun (45 passed) and fixed `ghbdtn` simulation proposes `привет`; no Windows
execution or ordinary application autocorrection is established by these checks.
The owner also ran the device-free simulator successfully on this Ubuntu machine:
`ghbdtn -> привет`, Russian confidence 0.98. This is owner-reported simulation
evidence, not a live application correction result.

1. Establish an app/toolkit range replacement transaction and targeted undo.
   The real editor experiment proves grouped delete/insert is insufficient; see
   NATIVE_TEXT_EDITOR_RESULTS.md and GTK_REPLACEMENT_API_RESULTS.md. Extend the
   approved isolated storage prototype to view/subclass notification semantics
   and bind correction-record cancellation to field/focus ownership, retaining
   refusal guards until each is demonstrated.
   No permanent toolkit fork or replay fallback is authorized by these results.
2. Complete exact layout mappings, within-word policy and measured quality corpus.
3. Implement/validate Windows and macOS adapters on real target hardware.
4. Finish localized onboarding/settings, manual repair, exclusions and sounds.
5. Implement account/device/Stripe service and entitlement integration/recovery;
   production credentials, legal/tax checkout settings and signing keys unavailable.
6. Complete trusted updater, native installers/signing/notarization, current website
   beta matrix and release criteria. Landing source now reflects development status;
   release download and checkout links remain intentionally unavailable.

Source code and tests can continue independently where these blockers do not apply.
Do not equate infrastructure progress with a functioning first release.

## Interruption recovery

The owner requested automatic continuation after usage limits reset. No available
session tool or verified local CLI control provides that trigger; automatic
restart is **not configured**. No scheduler, watchdog or alternate agent was
installed. Current checkpoint and reusable continuation prompt:
[RESUME_DEVELOPMENT.md](RESUME_DEVELOPMENT.md). Read this state before resuming;
do not repeat discovery or treat temporary build artifacts as installed software.
