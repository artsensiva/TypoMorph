# Original-target preservation proposal

Date: 2026-09-27. Status: option 1 approved and completed as a bounded prototype.
No production architecture, installed component, or release scope changes.

## Problem and decision

The native GNOME Wayland fixture reproduced old draft and delayed key delivery
into a newly focused field, including old draft delivery into a password field.
Fresh runs produced four passes and five failures; see
[NATIVE_WAYLAND_EXPERIMENT.md](NATIVE_WAYLAND_EXPERIMENT.md).
The reset case retained preedit; it did not demonstrate text loss. The protected
case did not demonstrate analysis of newly typed protected input.

No safe, transparent engine-only fix has been established. A focus query followed
by CommitText is not an atomic field-bound edit. Discarding a late result is safe
only if that result is an optional correction, not the user's consumed original
input. More focus polling cannot supply that distinction or ordering.

Recommend one bounded implementation: a **test-owned GTK editor transaction
prototype**. Let ordinary input reach its owning widget first. Compute a fixed
synthetic correction separately, then validate and apply it in that widget's
serialized edit path. Do not make TypoMorph own or delay original key delivery or
preedit in this prototype. This changes the ownership design under test rather
than claiming to repair the existing IBus transport.

## Options

| Option | Benefit | Cost / limitation |
| --- | --- | --- |
| 1. Test-owned client transaction prototype (recommended) | Smallest boundary under our control for testing preservation, stale-result rejection and correction together | Requires client cooperation; proves neither installation into existing apps nor browser coverage |
| 2. Prepare a coordinated native-stack change proposal | Directly investigates transparent integration through GTK, Mutter, Shell and IBus | No complete origin-aware API established; potentially multiple upstream/protocol changes and external release dependencies |

An extension-only focus check or global uinput fallback is not a third safe
option on the current evidence. Neither option changes approved product coverage.
Option 2 can follow option 1 if the contract proves useful; neither authorizes
publishing a report, installing patched packages, or shipping system forks.

## Proposed prototype contract

1. Normal key handling inserts original input without waiting for analysis.
   TypoMorph does not suppress keys, produce IM preedit, or replay consumed keys.
   Native IME composition remains outside correction scope.
2. Only test-owned, explicitly ordinary fields are eligible. Protected and
   unknown fields are rejected before any text snapshot. Selection, active
   composition, or unavailable safety information also prevents analysis.
3. A bounded synthetic word snapshot carries a session identity, field lifetime,
   focus generation, edit revision, expected range and caret/selection state.
   Focus loss/return, edits, selection or composition changes, safety changes,
   destruction and session replacement invalidate outstanding requests.
4. A delayed result is a suggestion. On the owning UI thread, recheck the full
   identity/state and expected range before changing text. No asynchronous hop
   may separate that check from the edit. The test-owned adapter must account
   for synchronous callbacks/reentrancy; merely running on one thread is not
   sufficient. Unsupported mutation behavior must remain a blocker.
5. A valid suggestion changes only the expected range in its original field.
   A stale, timed-out or disconnected suggestion changes nothing. Never redirect
   it to the active field, restore a destroyed field, or replay the original text.
6. Original text remains ordinary widget content when the user changes focus.
   TypoMorph retains no unfinished preedit needing a cross-field handoff. Bound
   and clear analysis snapshots on invalidation; no typed-content persistence,
   network transfer, clipboard fallback or global capture.

This is a proposed contract, not a claim that arbitrary GTK widgets or external
applications expose an atomic edit API. The prototype controls its own editor;
its exact edit and reentrancy guarantees must be documented from implementation.

## Implementation boundary and affected files

Approved implementation boundary:
- Add integrations/gnome/tests/client_transaction.py: test-only owner state and
  fixed ghbdtn -> привет candidate handling; no production classifier wiring.
- Add integrations/gnome/tests/test_client_transaction.py: real test-owned GTK
  widgets, deterministic delayed results, bounded synthetic-only assertions.
- Add a separate runner mode or runner under integrations/gnome/tests, reusing
  the verified private-compositor isolation and PID-checked input mechanism.
  The new path must not select the delayed echo/preedit fixture as its input
  engine: establish and record the ordinary input route independently.
- Update this proposal, PROJECT_STATE.md and the experiment evidence with actual
  results and deployment prerequisites.

Keep the existing native lifecycle reproducer and its failing expectations
unchanged. The new test is a separate architecture experiment, not a way to make
those platform failures disappear from the report. Production daemon, classifier,
installed layout bridge, account/payment code and safety gate are unaffected.
No package installation is included; report missing prerequisites if encountered.

## Acceptance evidence

Use real GTK widgets and input confined to a fresh private compositor. Verify
original text is present before releasing the delayed candidate. Assertions may
inspect only synthetic test-owned contents. Cover:

| Schedule | Required outcome |
| --- | --- |
| Stable ordinary field | ghbdtn appears first; an accepted candidate yields привет exactly once |
| Focus A -> B while candidate waits | Original stays on A; no correction reaches B |
| Two windows and rapid A/B/A | No retargeting; old generation is rejected even after return to A |
| More typing, Backspace or selection changes | Original edits remain in order; stale candidate cannot overwrite them |
| Ordinary -> protected or unknown | Old original stays on A; zero analysis requests/snapshots from the new field |
| Reset, composition start, cancellation or analyzer disconnect | Original content survives; no stale correction or replay |
| Field destruction or session replacement | Pending result is invalid; no write into another or recreated field |
| Callback/reentrancy during correction | No partial replacement, duplicate insertion, or stale nested mutation |

Use content-free counters for protected/unknown analysis checks; do not read their
contents through the analyzer to prove refusal. Test control can assert its own
synthetic field values. Run relevant existing harness checks if shared code changes.

Passing these cases would establish only the tested client-owned contract.
The old COMMIT-preedit and delayed-consumed-key cases concern another route and
remain unresolved. Standard IME behavior is not fixed by this prototype. Safe
undo, text/layout coordination, within-word classification, multilingual support,
Chrome and unmodified Text Editor integration still require separate evidence.

## Stop conditions and next decision

Stop before production integration if a validated mutation can partially fail,
reenter unsafely, collect protected/unknown content, or lose original input.
Report failures without weakening assertions or enabling the production gate.

After the bounded prototype, present its measured results and the concrete
client/toolkit hooks required for existing apps. The owner then chooses whether
to pursue a native-stack proposal or an application-adapter architecture. Do not
silently accept limited app coverage as the approved desktop-wide product.

Option 1 was approved and implemented. See CLIENT_TRANSACTION_EXPERIMENT.md:
20 prototype checks passed; the original native delayed-key failure still
reproduces. Production architecture remains unapproved.
