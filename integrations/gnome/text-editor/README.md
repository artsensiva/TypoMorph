> Current status (2026-09-30): baseline and patched editors now compile and run.
> Fixed replacement with ordinary undo/redo works; both post-deletion conflict
> cases reproduce unsafe partial edits. See [native evidence](../../../docs/NATIVE_TEXT_EDITOR_RESULTS.md).
> Production correction remains disabled. Source-only entries below are historical.

Follow-up: `probe_search_replace.c` tests the installed GtkSourceView replacement
API on fixed in-memory buffers, without launching the editor. The normal case
works; vetoed operations expose partial/duplicate edits. Build commands, four
observed outcomes and architecture implications are in
[GTK API results](../../../docs/GTK_REPLACEMENT_API_RESULTS.md).

# Text Editor 50.1 source-only experiment

Prepared 2026-09-27 under the owner's source-only approval. No dependencies were
installed; no C compilation, app launch, input injection or runtime test occurred.
This is a reviewable development patch, not a working TypoMorph integration.

## Artifacts and provenance

- `0001-private-owner-range-probe.patch`: disabled-by-default development probe.
- `upstream-files.sha256.json`: SHA-256 of the exact upstream input files read from
  `https://raw.githubusercontent.com/GNOME/gnome-text-editor/50.1/`.
- `verify_patch.py`: verifies those inputs, applies/reverses the patch in a temporary
  directory and checks byte-for-byte restoration. It never edits the supplied tree.

Only the necessary upstream files were downloaded to a temporary source fixture.
This is not a complete checkout or a verified upstream commit/archive identity.
The hashes pin the inspected file contents; verify a complete official checkout
against them before building. The patch and new C files use GPL-3.0-or-later,
consistent with the upstream files. No code has been submitted upstream.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 integrations/gnome/text-editor/verify_patch.py /path/to/gnome-text-editor-50.1
```

Observed result: hashes matched; forward application, reverse application and exact
restoration passed. New patch lines passed git's whitespace check. These checks do
not validate C syntax, ABI, signal behavior, build dependencies or runtime safety.

## Changes in the patch

Adds `typomorph_experiment=false` to Meson. Enabling it requires the existing
`development=true`; only then is the owner C file compiled and the page integration
enabled. The ordinary build does not attach an owner or register the probe action.

The page attaches the owner after document/view binding, invalidates before search
focus, and detaches before document shutdown. The owner observes document changes,
marks, view metadata, window activation and preedit. It keeps revision/epoch state,
cancels delayed work on invalidation and retains objects through synchronous
callbacks so page disposal need not immediately free the active completion state.
No code intercepts original keys or swaps document buffers.

The development-only widget action `typomorph.probe` accepts a string mode:

- `normal`: after eligibility checks, hold a fixed `ghbdtn` candidate for 200 ms,
  revalidate, then attempt a grouped document range replacement with `привет`.
- `gap-edit`: insert the fixed competing character `x` after deletion. The probe
  should detect the partial-edit condition, emit `FAULT_PARTIAL_EDIT` and disable
  further probes on that owner. This intentionally exposes a known unsafe schedule;
  it is not a safe replacement implementation or a successful correction test.
- `callback-begin-edit`: connect a one-shot fixed `x` insertion to the real
  `begin-user-action` signal. Expected outcome (not executed): preserve the
  competing edit and refuse before deleting the original word.
- `callback-delete-edit`: connect a one-shot fixed `x` insertion after the real
  `delete-range` default handler. Revalidate the signal iterators after the edit.
  Expected outcome (not executed): detect the partial replacement, retain the
  competing edit, and disable this owner with `FAULT_PARTIAL_EDIT`.
- Other values report `UNSUPPORTED`. No arbitrary replacement payload is accepted.

The action has no keyboard binding, menu entry, auto-trigger or public D-Bus API.
A private in-process driver can invoke it via `gtk_widget_activate_action` on the
actual page. The source now also provides `typomorph.check` assertions described
below; the isolated application launcher/control driver is still pending.

The probe refuses outside the established private-compositor environment and an
additional `TYPOMORPH_TEXT_EDITOR_SYNTHETIC=1` marker. It requires an active focused
editable ordinary view, an unsaved six-character document, no selection/loading/
busy/shutdown state, and fixed original content. It reads at most that synthetic
word after metadata checks. Diagnostics print status only, never document contents.
Environment checks prevent accidental ordinary-session use; they are not security
authentication against a malicious process. Use only the isolated runner design in
`docs/NATIVE_TEXT_EDITOR_BUILD.md`, never the personal desktop.

Any preedit notification suspends the owner without inspecting its payload. No
composition-end grant is implemented. Initial composition eligibility is not
proven; this is another reason the probe cannot be a production capability.

## Deliberate limitations and next evidence

The full private API specification is **not implemented**. There is no advertised
atomic `owner_complete`, generalized word/range capture, IPC, classifier, targeted
revert, layout coordination or certified protected-field adapter. `normal` reports
`RANGE_PAIR_ATTEMPTED`, not APPLIED or PASS; a harness must verify actual output,
callbacks, document state and undo. Even that output would not prove isolation.

The grouped delete/insert diagnostic can partially modify its synthetic document.
On a detected conflict it stops without trying to restore a whole-buffer snapshot
or replay input. That behavior is an experimental failure, not acceptable product
recovery. No guard is claimed to block arbitrary GTK signal handlers. The injected
`gap-edit` case is deterministic competing-edit simulation, not yet a callback
reentrancy test; The two new callback-driven schedules are prepared but have not been executed.

Before any runtime claim: acquire approved build dependencies, compile the full
matching source, validate baseline startup in the private compositor, add the
real-page harness, exercise actual reentrant callbacks and undo/redo, and report
failures without weakening assertions. Existing GNOME/IBus native failure tests
and the production gate are unchanged. Unmodified Text Editor and Chrome are not
fixed by this patch.

## Follow-up source review

Continued within source-only approval, without package installation:

- Replaced the strong window reference with a tracked weak pointer. The window
  owns the page, so the previous page -> owner -> window chain could prevent
  disposal. Document/view references still protect an active completion callback.
- Kept the nested-probe guard active through begin/end-user-action callbacks,
  including refusal cleanup. Internal eligibility checks no longer mistake the
  active diagnostic itself for an external nested request.
- Recheck eligibility after deletion before further edits. Check revision, bounded
  final contents and eligibility after insertion/end-user-action; detected conflicts
  now emit FAULT_PARTIAL_EDIT and disable further probes rather than merely reporting
  an attempted pair. This detects some failures; it neither prevents intermediate
  mutation nor supplies rollback/atomicity.
- Replaced verifier assertions with explicit failures so Python -O cannot disable
  restoration/new-file checks. Added test_verify_patch.py: valid round trip, altered
  upstream rejection and corrupt-patch rejection, each verifying input immutability.

Validation: all three verifier regressions passed under Python -O; final forward/
reverse application and byte-for-byte restoration passed; git diff --check passed.
These results cover patch packaging and source inspection only, not execution of
C callbacks, reference lifetimes or GTK mutation/undo behavior.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 integrations/gnome/text-editor/test_verify_patch.py /path/to/gnome-text-editor-50.1
```

## Callback probe follow-up (2026-09-28)

Added the two callback modes above, with scoped signal disconnection on every
completion exit and a one-shot guard set before insertion. The guard prevents
recursive test injection; it is not a document mutation barrier. Callbacks remain
restricted to the existing fixed synthetic document/private-session checks.

Fixed the page attachment call to explicitly cast EditorSourceView to its
GtkSourceView base type. This addresses a C pointer-type mismatch found by source
inspection, not a compiler-reported result.

Re-ran all three verifier regression checks under Python -O and the independent
forward/reverse restoration check successfully. These are packaging checks only.
Compilation, callback execution, document state, normal undo/redo, and targeted
reversal remain unverified. No dependency installation or live test was performed.

Signal contracts consulted:
[delete-range](https://docs.gtk.org/gtk4/signal.TextBuffer.delete-range.html),
[insert-text](https://docs.gtk.org/gtk4/signal.TextBuffer.insert-text.html), and
[begin-user-action](https://docs.gtk.org/gtk4/signal.TextBuffer.begin-user-action.html).
The build must verify behavior against the actual supported GTK version.

## In-application assertions (2026-09-29, source only)

The disabled development build now exposes `typomorph.check` with the same four
fixed case names as `typomorph.probe`. A future isolated driver must first arrange
the synthetic `ghbdtn` document through the real application input route, invoke
`typomorph.probe`, wait for completion, then invoke `typomorph.check` on that same
page. Neither action has a menu, shortcut or public D-Bus endpoint. No automatic
startup trigger, document seeding or desktop-wide control endpoint was added.

The checker requires one completed, unchecked probe at the same epoch/revision.
It refuses pending, stale, repeated and mismatched requests. It checks field
metadata and the original document/view binding before reading at most seven
characters. It prints only fixed status labels, never the document's contents.

- `normal` checks the replacement, then invokes ordinary buffer undo and redo and
  checks each exact synthetic result. It retains the owner through synchronous
  callbacks and blocks nested probe/check actions throughout. This is **not**
  targeted correction reversal and does not prove callback isolation.
- `callback-begin-edit` checks that the original word and competing character
  remain after refusal before deletion.
- `gap-edit` and `callback-delete-edit` check the fault state and retained competing
  character. They report `KNOWN_UNSAFE_PARTIAL_EDIT_OBSERVED`, never a safety pass.
- A failed assertion disables further probes for that owner. It does not attempt
  compensation or whole-document rollback.

`assess_probe.py` is the bounded output-assessment component, not an application
launcher. It requires the case and actual child exit status, consumes at most
64 KiB from stdin, and accepts only the exact ordered protocol markers. It returns
0 for the expected normal/refusal observation, 1 for failed/incomplete evidence,
and **2 for a reproduced unsafe partial edit**. A clean process exit is required;
unknown, duplicate or stale markers fail. A zero result is not production certification.

Validation performed: six synthetic assessment tests, three patch-verifier
regressions under Python -O, forward/reverse restoration and whitespace checks.
No C build, GTK callbacks, undo/redo, application startup or native typing ran.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
  -s integrations/gnome/text-editor -p test_assess_probe.py -v
```

GTK references: [ordinary undo](https://docs.gtk.org/gtk4/method.TextBuffer.undo.html)
and [ordinary redo](https://docs.gtk.org/gtk4/method.TextBuffer.redo.html).
Next runtime prerequisites are unchanged: approved dependencies, matching baseline
and patched builds, and a private real-page driver with process/timeout cleanup.

## Private runner and fixed-case driver

`run_native.py` creates fresh XDG directories, private D-Bus roles and a headless
compositor. Before any synthetic key it verifies the RemoteDesktop service PID
against its own compositor child. It does not inherit the host display/bus/input
settings or change HOME. Processes have bounded deadlines and cleanup. This is
process/session isolation, not a general filesystem sandbox.

Only the private development build with a recognized `TYPOMORPH_TEXT_EDITOR_CASE`
starts the internal driver. It waits for an eligible empty document, then for the
runner's actual `ghbdtn` keys; it does not seed the word or grab focus. It invokes
one probe/check sequence and quits. Timer references protect callback lifetime and
are removed on owner disposal. No menu, general IPC edit API or production trigger
is introduced. Three helper tests cover isolated environment and child cleanup;
they do not exercise GTK, process-group behavior, the compositor or the C driver.

## Runtime follow-up

The initial normal case refused because begin-user-action changed `can-undo`.
The patch now excludes only document undo-availability notifications from epoch
invalidation; actual document/mark changes still invalidate. Content-free markers
identify refusal stages. Both baseline and patched full builds pass with GCC;
four real fixed cases ran and were repeated after removing temporary verbose
notification diagnostics. The two partial-edit outcomes remain safety failures.
See the linked evidence for versions, patch fingerprint and test limitations.
