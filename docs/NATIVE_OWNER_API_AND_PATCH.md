# Native owner API and Text Editor patch specification

Date: 2026-09-27. Native-cooperation-first direction approved.
Owner initially approved source-only preparation; on 2026-09-29 they explicitly
authorized the refreshed dependencies and isolated native build/tests. The builds and private runtime cases completed on 2026-09-30; see
NATIVE_TEXT_EDITOR_RESULTS.md. The delete/insert barrier is demonstrably insufficient.
The initial development diagnostic patch is in integrations/gnome/text-editor;
see its README for implemented scope. Full API and safe mutation remain unproven.
Build/runtime validation is recorded in NATIVE_TEXT_EDITOR_RESULTS.md. The normal
case works; post-delete callback schedules reproduce unsafe partial edits.
No stable public API, deployable integration or upstream agreement is claimed.

Follow-up (2026-09-30): the owner approved a separate temporary GTK transaction
prototype. Its storage primitive and ordinary undo/redo are built and tested;
views and subclasses still refuse. Bounded targeted correction reversal now passes
synthetic tests; field/focus ownership and cancellation wiring remain pending. See
[prototype evidence](../integrations/gnome/gtk-transaction/README.md). The full
owner API below and ordinary app integration remain unproven.

## First implementation boundary

Patch a separate GNOME Text Editor 50.1 development checkout. Preserve its real
EditorDocument and GtkSourceView, document callbacks and undo manager. Add a
private owner on the document/view pair; do not replace the document buffer.
Only synthetic unsaved documents in the private compositor are in scope.
Search fields are transition targets, not implicitly authorized document owners.
This is an application-specific experiment, not coverage of unmodified apps.

First use an in-process delayed fixed candidate, with no TypoMorph input engine
and no daemon connection. The purpose is to settle the actual editor mutation,
composition and undo boundary before adding IPC. Ordinary input stays on the
existing application path. No preedit, global key suppression, replay or clipboard
fallback belongs to this correction adapter.

## Private API contract v0 (proposed names)

All owner operations run on the application's UI thread. A worker must never
hold widget pointers or call GTK. These are design signatures, not installed APIs:

```c
Owner *owner_attach(EditorDocument *document, EditorSourceView *view);
CaptureResult owner_capture(Owner *owner, Request **request_out);
ApplyResult owner_complete(Owner *owner, const Candidate *candidate);
void owner_invalidate(Owner *owner, Invalidation reason);
void owner_detach(Owner *owner);
UndoResult owner_revert(Owner *owner, CorrectionId id);
```

CaptureResult is READY, INELIGIBLE, BUSY, UNSUPPORTED or TOO_LARGE.
ApplyResult is APPLIED, STALE, INELIGIBLE, EXPIRED, UNSUPPORTED, INVALID_PAYLOAD
or FAULT. Every result except APPLIED/FAULT guarantees no mutation. FAULT means
an invariant was violated; disable the experiment and record only the reason.
FAULT is not an acceptable production recovery strategy or passing safety test.

### Owner-held state and payloads

| Item | Definition |
| --- | --- |
| Session | Fresh identity on attach or reconnect; never reuse after detach |
| Owner identity | Opaque document lifetime plus view lifetime; not a memory address sent to a worker |
| Epoch | Monotonic generation invalidated on focus loss/return and eligibility changes |
| Revision | Document edit counter, changed by all edits including undo/redo and programmatic edits |
| Range | Owner-held start/end character offsets and insertion/selection marks at captured revision |
| Request | Session, request ID, owner identity, epoch, revision, deadline and bounded UTF-8 word |
| Candidate | Echoed identity/version fields and bounded replacement UTF-8; never a caller-selected widget/range |
| Receipt | Applied correction ID and revision; no full document text |

Prototype limit: one outstanding request per owner; original and replacement at
most 32 Unicode scalar values / 128 UTF-8 bytes each, valid UTF-8 and no embedded
NUL. One-second owner-local deadline. These are experimental bounds, not approved
product word limits. Retain only a bounded inverse record for the latest accepted
correction where safe undo is supported. Clear requests on invalidation, detach,
pause, disconnect and expiry. Do not persist payloads, document contents or logs.
UTF-8 byte lengths, GtkTextIter character offsets and grapheme boundaries are
separate: do not cast between them. Reconstruct iterators at the checked revision;
never retain GtkTextIter across edits or an asynchronous wait.

### Lifecycle and ordering

DETACHED -> INELIGIBLE -> READY -> WAITING -> APPLYING -> READY.
WAITING can be canceled to READY/INELIGIBLE; detach is terminal for that session.
Before reading any range, require known ordinary editable document/view, active
window and exact focused view, no selection/composition, no load/shutdown/busy
state, and supported mutation/undo capability. Unknown metadata refuses capture.

Observe document edits, selection/insert-mark movement, view/window focus,
editability, buffer changes, composition, loading, shutdown and disposal.
A focus round trip changes epoch even if all text matches again. A shared document
in two views has a shared revision but separate owner/focus identities. Work must
not cross views on completion. Capture occurs only after the original edit has
completed, never inside a partially executed insertion callback.

Preedit handling must use verified lifecycle information. GtkTextView exposes
[preedit-changed](https://docs.gtk.org/gtk4/signal.TextView.preedit-changed.html),
which carries text, not a content-free start/end token. Do not copy or analyze
that payload. Subscribe only on the explicitly ordinary test document view; use
it to invalidate/suspend. Verify installed behavior and initial composition state
before granting READY. If end/reset cannot be established reliably, stay suspended;
text emptiness heuristics alone are not a proven complete lifecycle protocol.

### Completion and mutation barrier

1. Reject bad identity, duplicate, expired or oversized responses before text
   access. Resolve the original live owner, not the currently focused field.
2. On the UI thread recheck epoch/revision, safety, focus, selection and expected
   bounded range. Preallocate replacement and receipt before entering mutation.
3. Require an owned mutation barrier covering the actual document callbacks,
   marks, undo and view lifecycle. Do not run a nested main loop or await a reply.
4. Replace only the approved range through the existing document edit path.
   Publish APPLIED only after the edit and its receipt are coherent.
5. Never compensate for failure by deleting elsewhere, replaying text or rolling
   back a full document snapshot. Original input belongs to the editor already.

The barrier in step 3 is the central unimplemented requirement. A boolean that
blocks only TypoMorph callbacks cannot block arbitrary document mutations.
[GTK insert-text](https://docs.gtk.org/gtk4/signal.TextBuffer.insert-text.html)
invokes connected handlers before the default insertion handler. A delete/insert
pair can therefore expose intermediate state or encounter reentrant edits.
[begin_user_action](https://docs.gtk.org/gtk4/method.TextBuffer.begin_user_action.html)
groups undo; it does not promise isolation. freeze_notify does not freeze all edit
signals. Do not suppress or discard other legitimate editor edits to make tests pass.

The development patch may exercise a grouped range replacement with injected
callbacks to determine the exact failure boundary. It must NOT advertise a safe
replace capability until those tests pass with a justified barrier. If application
hooks cannot provide it, stop automatic application and specify the necessary
GTK buffer primitive; do not silently weaken this contract or substitute buffer swap.

### Undo and layout

Test normal document undo/redo separately from TypoMorph correction reversal.
After correction followed by later typing, ordinary Ctrl+Z is allowed to undo the
later typing first; it must not be presented as targeted correction reversal.
For owner_revert, validate a retained correction range/revision and preserve later
disjoint typing. Overlapping edits, lost ownership or insufficient tracking must
refuse safely. Failure to support the approved safe-undo behavior remains a release
gap, not permission to omit it. No cross-field inverse edits.

Do not switch OS layouts in this first experiment. Text acceptance and a subsequent
asynchronous layout change are not an atomic pair. A later layout coordinator
needs its own same-owner/seat validation; the existing layout bridge alone does
not provide it. Successful text tests do not satisfy within-word switching scope.

## Verified upstream patch map

Version-tagged source read on 2026-09-27, including direct HTTPS reads where the
web tool could not retrieve the file. Line positions are advisory; pin source
commit/archive hash before patching. Newly proposed symbols are identified below.

| File in GNOME Text Editor 50.1 | Verified anchor | Proposed patch |
| --- | --- | --- |
| src/editor-page.c | editor_page_set_document, constructed, dispose | Attach after real view/document binding; detach before document shutdown/template disposal |
| src/editor-page.c | _editor_page_set_search_visible | Invalidate document owner before search focus; never pass document permission into search |
| src/editor-document.c | editor_document_insert_text, delete_range, changed; _editor_document_shutdown | Revision/lifecycle invalidation; actual range/undo experiment; preserve existing handlers |
| src/editor-source-view.c | constructed, dispose; key controller | Bind focus/editability/composition observation; no interception of ordinary original keys |
| src/editor-document-private.h | Existing private document declarations | Declare only necessary private test integration entry points |
| src/editor-typomorph-owner.c/.h | New proposed files | Owner state, bounded capture, validation, receipt/revert and content-free diagnostics |
| src/meson.build and meson_options.txt | Source list; development option | New explicit disabled-by-default typomorph_experiment option; compile only for development tests |

Sources: [page](https://raw.githubusercontent.com/GNOME/gnome-text-editor/50.1/src/editor-page.c),
[document](https://raw.githubusercontent.com/GNOME/gnome-text-editor/50.1/src/editor-document.c),
[view](https://raw.githubusercontent.com/GNOME/gnome-text-editor/50.1/src/editor-source-view.c),
[private header](https://raw.githubusercontent.com/GNOME/gnome-text-editor/50.1/src/editor-document-private.h),
[source build](https://raw.githubusercontent.com/GNOME/gnome-text-editor/50.1/src/meson.build),
[options](https://raw.githubusercontent.com/GNOME/gnome-text-editor/50.1/meson_options.txt).

TypoMorph repository artifacts for the next approved implementation:
- integrations/gnome/text-editor/: recorded source identity, development patch and
  build/run instructions; external checkout/build output kept in a temporary path.
- integrations/gnome/tests/: a separate real-application runner/assertion harness;
  retain existing native failing cases and client prototype tests unchanged.
- Evidence report and PROJECT_STATE.md update, including failures and prerequisites.
No changes to the production daemon, installed extension or package are included.

## Transport beyond the application patch

The first in-process experiment requires no Shell, Mutter, IBus or Wayland change.
For a cooperating app, a later bounded local request/reply channel can connect its
owner to the analyzer directly, avoiding CommitText for correction entirely. The
owner remains the only mutation authority. Define peer/session authorization,
message size/version checks, pause/disconnect generations and request limits before
adding that IPC. Same-user credentials alone are not proof of application identity.
Do not create an unauthenticated desktop-wide edit service.

If the native input-method transport is chosen instead, the current CommitText
path cannot simply carry an AT-SPI ID as trusted authority. A coordinated design
must carry a client-issued lifetime/revision token and result type through the
client, Wayland/Mutter, Shell and IBus bridge, then let the client validate/apply.
Original consumed-key completion and optional correction must be separate message
classes. Exact protocol changes remain conditional on upstream collaboration;
this document does not invent an already available cross-stack transaction API.
Chrome needs a renderer/editor owner of its own. Patching Text Editor or GTK does
not implement Chrome support. No permanent system forks are selected for shipping.

## Acceptance and stop conditions

Use real synthetic typing into the patched application's document, with explicit
comparison to unpatched baseline. Verify:
- Original word visible before candidate; exact range change once; normal save/
  modified state and document identity preserved.
- Focus to search, another tab/window, A/B/A, document destruction and load/reload
  all reject stale results without moving original text.
- Concurrent typing, Backspace, selection, programmatic edits, UTF-8 boundaries,
  expiry/disconnect and response replay preserve original edits.
- Normal undo/redo history and targeted reversal with later disjoint typing are
  distinct, measured behaviors; overlap refuses safely.
- Composition suspends before capture; search/unknown or protected targets receive
  no document snapshot/permission. Test only synthetic controlled content.
- Injected reentrancy before deletion, between delete/insert, during mark changes
  and during undo cannot produce partial replacement or swallow another edit.

If callback isolation or reliable composition eligibility fails, report a failed
capability and stop before enabling automatic application. Do not transform a
known-gap assertion into a safety pass. No production gate removal follows from
this application-specific experiment.

## Local upstream questions (draft; not sent)

1. Is there a supported owner-side conditional range replacement with an explicit
   revision and coherent undo, or is a new GtkTextBuffer primitive needed?
2. How should a cooperative adapter obtain content-free composition lifecycle and
   field eligibility before capture, including initial attachment?
3. Can a toolkit expose this capability without application-specific patches, and
   how would it preserve app callbacks, tags, marks and undo semantics?
4. For browser/toolkit interoperability, who owns the versioned origin token and
   acknowledgment? A surface or shared IBus context alone is insufficient here.

## Next approval

See NATIVE_TEXT_EDITOR_BUILD.md and NATIVE_BUILD_PACKAGE_PLAN.txt. Approve the
listed dependency installation and isolated application patch experiment together,
or choose source/patch work only without installing dependencies. No sending of
upstream messages, installed app replacement or production deployment is included.
