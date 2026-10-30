# TypoMorph project state

Updated: 2026-10-28. Branch: `diagnostic/device-echo-investigation`.

## Phase and authority

Linux safety/integration feasibility (P1–P3), with independent P6–P8 components
in progress. **Version 1 is not complete or release-ready.** The owner requests
autonomous implementation within the approved specification, without repeated
routine approval questions. No scope/privacy/commercial reduction is authorized.
The owner authorized a development checkpoint commit and GitHub push on
2026-10-01; this does not authorize a release, website deployment or live payments.
Checkpoint `764794a` was pushed to the current branch. Subsequent layout/keymap
work is uncommitted. Linux v1 is the current implementation priority; other approved
platform requirements have not been removed.
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
- Linux beta scope (D-38, Product Owner, 2026-10-28): Linux is the first beta
  target; input/layout languages are the six above plus Greek, Turkish, Polish,
  Portuguese and Italian. The five are not UI-localization additions. Windows and
  macOS stay in the longer-term scope. The first-public-release baseline is NOT
  rewritten as eleven-language or Linux-only. Scope decision only; nothing is
  implemented, tested or ready for these languages or for a beta. Undecided: Linux
  layout IDs/variants, quality thresholds, beta audience/access model (and whether
  it is the D-35 closed beta), pricing/licensing, distribution, and public-release
  status of the five. No demand evidence was provided. Beta stays inside the
  documented Ubuntu GNOME scope; safety/privacy rules are unchanged.
- No fixed release date or silent scope reduction. Stores/additional repositories,
  languages beyond the D-38 Linux-beta scope and other deferred features remain as specified in SPEC.md.

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
  A separate native XKB mapper covers bounded physical-key proposals for eight
  explicitly selected Linux IDs. Five test groups cover modifier/regional cases
  and all 56 directed pairs at one key position. Dead keys/composition refuse;
  active-layout identity, full quality coverage and live integration remain pending.
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
The primitive still refuses GtkTextView accessibility handlers, buffer subclasses,
tags, external legacy edit handlers and commit observers. A private Broadway
GtkTextView probe confirms correction/reversal refusal and success after detachment.
Its earlier leak-enabled run reported Fontconfig/Pango allocations; behavioral
checks pass with leak detection disabled and ASan/UBSan retained.

2026-10-01 layout milestone: migrated GTK's own four layout observers to explicit
internal cache/cursor maintenance hooks. Six tests cover two warmed caches,
correction/reversal/undo/redo, callback edits and layout detachment, bidi,
preedit/layout-count limits, external observer refusal and ordinary multiline edits.
Twenty upstream GTK buffer tests also pass on private Broadway. Display/layout
tests retain the stated leak-check limitation. All eight patched source files
reproduce exactly; repeat preparation refuses without modifying them. GtkTextView
accessibility and GtkSourceBuffer semantics remain separate gaps. This does NOT
fix real applications. No installed GTK was changed. Evidence, limits and hashes:
integrations/gnome/gtk-transaction/README.md.

2026-10-01 accessibility milestone: a bounded AT-SPI snapshot primitive publishes
old/new payloads without rereading the widget. Four scenario groups pass against
the actual GTK backend and real registry/listener in an isolated headless GNOME
Wayland session: Unicode offsets, maximum payload, invalid-input refusal,
callback-event order and unrealized-context refusal. The fixture supplies fixed
payloads independently of storage replacement; there is NO GtkTextView transaction
wiring yet. D-Bus delivery is not atomic; notification-failure recovery remains open.
GTK now compiles Wayland + Broadway; neither tests nor builds use the owner desktop.
The ten-file patch reproduces exactly. `verify-prepare.py` also checks repeat and
modified-source refusal without changes. No installed library/editor was replaced.

2026-10-28 attached-view milestone: `test-a11y-view.c` drives the prototype splice
through one realized GtkTextView in a presented window on the private headless
Wayland session with a real AT-SPI listener. Six groups pass (4/4 runs): ordered
delete/insert payloads and offsets after commit, with the buffer already holding
the replacement at the first event; undo/redo replay events; a callback edit's
event follows the replacement pair; ordinary deletion still reports removed text;
an unrealized second view or unrealized primary context refuses splice, undo and
reversal without changing text/revision or emitting events. No defect was found;
no patch input changed. Limitation: presenting/focusing the window did not realize
the view context and an AT-SPI client query could not reach the window object, so
the test realizes it explicitly. Multiple ready views, disconnect and
notification-failure recovery, GtkSourceBuffer and real applications remain
unverified. The twelve-file patch reproduces (verify-prepare.py). Synthetic only.

2026-10-28 natural-readiness result (`test-a11y-natural.c`, no explicit realize).
Three scenarios each run in a fresh process ("cold" = first and only window of
that process), 6 runs each, exit status enforced: `primary` - a view present at
first presentation is NOT ready after presentation, mapping or a 3 s passive wait,
and is ready after an AT client walks Registry -> application -> window
`GetChildren`; `late-added` - a view added to an already realized window is ready
without a walk, but 3.15-3.42 s after presentation (registration completion);
`listener-gap` - a characterization, not a desired behavior (below). Failure
propagation was checked with scratch-copy mutations. The gap scenario's rare
exits (timeout before readiness -> 3, gap absent -> 7) have harness self-tests; a
timer-cleanup defect that aborted the timeout path was found in review and fixed. An earlier combined-process
draft did not isolate the primary measurement and was corrected.

Reproduced listener gap (6/6 plus 5/5 development runs, state-detected, no
sleeps): at root registration GTK's listener flag turns false and the view becomes
ready in the same dispatch step; the table is learned 1.4-5.0 ms later. A splice
inside that interval commits, sets no failure flag and delivers no AT-SPI event;
later splices deliver. Events are lost silently; not fixed, no recovery contract.
Why registration takes ~3 s and why the two coincide are not established. Not
tested: real assistive technology or desktop session, multiple views. The earlier
note that a client query failed was a defect in a draft probe. No patch input
changed. Synthetic only; no language or beta readiness is implied.

2026-10-29 subclass boundary (real Text Editor 50.1 `EditorDocument`, fixed
synthetic text, private compositor, patched application fixture - NOT unmodified
apps and NOT a beta decision): the GTK bounded transaction cannot be applied to the
actual GtkSourceBuffer subclass today and stays refusing. On the live document the
subclass type, a non-empty tag table, external edit handlers and (from the runner's
`GTK_A11Y=none`) no AT-SPI context were all present together; the one splice call was
refused (6/6 runs) with text, revision, `changed`/`title` counts, undo state,
document identity and view binding unchanged. This shows combined refusal only: it
does not identify the refusing guard or show that any single condition suffices;
no per-condition variant was run. Source trace: the splice
skips the default-handler chain, including GtkSourceView's private highlight-engine
and cursor updates and EditorDocument's title/cursor-jump handling; an ordinary edit
was observed to notify `title`. Missing: a supported hook that tells every layer
and external observer of one atomic replacement. The two post-deletion
competing-callback cases still fail (exit 2) and the two earlier passing cases still
pass. Listener gap: unchanged, no GTK change; evidence needed for an explicit
unknown-state refusal is defined in the GTK README. Details: integrations/gnome/
text-editor/README.md.

2026-10-29 stale text-input-v3 characterization (stock system GTK 4.22.4 `Gtk.Text`,
GNOME Shell 50.1 headless, private IBus, test-only engine, fixed synthetic text -
NOT Chrome/Firefox, NOT an unmodified-application correction path): a deliberately
stale `delete_surrounding_text` + `commit_string` was applied by the client, and the
`done` serial did not prevent it. Nine schedules, run 27 times in fresh sessions
(deterministic): four pass (the control and three coincidentally benign cases), five
FAIL and are retained failures: commit inserted mid-field with the original word kept
(cursor moved), a different word replaced (word changed), and a commit delivered to
the wrong field; both serial-match and in-flight serial-mismatch variants corrupt.
In the two matched cases whose deletion exceeded the compositor's cached cursor,
Mutter 50.1 returned before sending the delete event, but its independent commit
path still sent the commit (partial application; confirmed by source audit).
Shell's fallback is not involved because Mutter rejects later while processing
the queued event. An unconfirmed possible
out-of-bounds read in GTK's delete handling was not confirmed by an uninformative
ASan run. Existing native cases unchanged (4 pass / 5 fail); 20 client-transaction
checks pass. This shows the existing protocol path gives no compare-and-replace for
this client; no workaround was added and no architecture or beta scope was decided.
Details: docs/NATIVE_WAYLAND_EXPERIMENT.md.

PolterType review pinned to `88709a6efd168a4a2f47382b7b224ebd76cb3573` is complete
(POLTERTYPE_REVIEW.md). Global key replay does not meet approved field safety;
no PolterType code/data was copied or executed. No live typing captured this turn.

## Latest verification

- Rechecked 2026-10-01: 86 Rust tests passed; two private-bus tests excluded from default
  run then passed separately using isolated synthetic GJS services.
- Strict workspace/all-target clippy and formatting passed.
- GitHub checkpoint CI: formatting, Clippy and tests passed. Audit reported no
  known vulnerabilities but flagged unmaintained `ansi_term`/`atty` through
  ksni -> dbus-codegen -> clap build dependencies, plus an `atty` unsoundness
  warning. Its check-publication step failed for missing `checks: write`.
  The permission fix is local and has not been tested by another GitHub run.
- Optimized workspace build passed; 21 focused release tests passed, including
  refusal of the live-input diagnostic flag. Linker warns about a deprecated
  optimization setting; no build/test failure.
- Five installer orchestration tests passed with mock network/signing/install
  commands, plus ten native-runner/assessment Python tests. Browser popup JavaScript
  syntax and Git whitespace checks passed. All 29 GTK buffer/history tests were
  rerun with ASan/UBSan and leak checks; six layout and 20 upstream buffer tests
  pass with the display-test limitations above. Four private Wayland AT-SPI
  scenario groups pass with ASan/UBSan and fatal probe warnings; font-stack leak
  checking is disabled. Rust source is unchanged since its recorded validation.
- Rechecked 2026-10-28 after rebuilding GTK under /tmp: 29 buffer/history, six
  layout, four direct AT-SPI, six attached-view AT-SPI, Broadway view refusal and
  20 upstream buffer tests pass; pinned-source reproducibility check passes.
  The natural-readiness scenarios (primary, late-added, listener-gap) each passed
  6 of 6 fresh-process runs; the primary view was never ready passively.
  Rust and CI were not rerun.
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
   approved isolated storage prototype beyond single-view GtkTextView snapshot
   publication (synthetic, 2026-10-28) to multiple views, disconnect recovery,
   real-session context realization and GtkSourceBuffer subclass semantics,
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

Linux beta blockers (D-38): exact Linux layout variants for all eleven languages
(the mapper covers eight IDs; none for Greek, Turkish, Polish, Portuguese, Italian);
mapping and per-direction quality validation with agreed thresholds; safe correction
in ordinary applications (BUG-001, production correction disabled); and beta
readiness criteria, audience and access model, none yet defined. No beta is ready
and no date is committed.

Source code and tests can continue independently where these blockers do not apply.
Do not equate infrastructure progress with a functioning first release.

## Interruption recovery

The owner requested automatic continuation after usage limits reset. No available
session tool or verified local CLI control provides that trigger; automatic
restart is **not configured**. No scheduler, watchdog or alternate agent was
installed. Current checkpoint and reusable continuation prompt:
[RESUME_DEVELOPMENT.md](RESUME_DEVELOPMENT.md). Read this state before resuming;
do not repeat discovery or treat temporary build artifacts as installed software.
