# Isolated GTK storage transaction prototype

Owner approved this experiment on 2026-09-30. **Not a production adapter, GTK
distribution, or fix for unmodified applications.** No installed library is
changed. The earlier editor and search-replacement failure fixtures remain intact.

## Source and artifacts

Official release archive:
[GTK 4.22.4](https://download.gnome.org/sources/gtk/4.22/gtk-4.22.4.tar.xz).
SHA-256: `51bd9f60c7d23a665a556c7364c21fb2e4e282566b3e7e092455e8f910330893`.

- `prepare.py` checks twelve exact release-source file hashes before patching a
  disposable source tree under `/tmp`. It refuses an already modified tree.
- `btree-splice.inc` supplies a new internal segment replacement operation.
- `buffer-splice.inc` supplies an explicitly experimental buffer entry point.
- `layout-splice.inc` moves GTK's own layout-cache maintenance into internal edit
  hooks and defers transaction invalidation notifications until history is ready.
- `history-splice.inc` adds a standalone chronological undo/redo action.
- `a11y-splice.inc` adds bounded post-commit AT-SPI event payload publication;
  it does not edit text or authorize a transaction.
- `view-splice.inc` registers attached GtkTextView instances with the buffer,
  captures their ready AT-SPI contexts before commit, and publishes the snapshot
  payloads after commit. GtkTextView's own insert/delete accessibility handlers
  move into the buffer's default edit handlers, keeping remove-before and
  insert-after semantics for ordinary edits.
- `test-history.inc` adds history, savepoint and notification tests.
- `test-revert.inc` adds targeted reversal and mutation-tracking regressions.
- `test-view.c` and `run-view.py` check attached-view refusal on private Broadway
  and D-Bus Unix sockets, without connecting to the desktop.
- `test-a11y-view.c` drives the splice through an attached GtkTextView with a
  real AT-SPI listener on a private headless Wayland session (`run-a11y.py`);
  it realizes the view context explicitly.
- `test-a11y-natural.c` measures, one fresh-process scenario at a time, whether
  that context becomes ready without explicit realization, and characterizes the
  listener-table startup gap.
- `test-splice.c` exercises the real patched GTK library using synthetic buffers.
  It does not launch an application, connect to a display, capture input or open
  personal documents. Fixture strings are fixed; no arbitrary text is accepted.

The C additions use LGPL-2.1-or-later, compatible with GTK's LGPL-2.0-or-later
source. The archive and compiled artifacts remain outside the repository.

## What this experiment changes

The new operation validates a bounded expected word and text revision, prepares
all replacement character segments, then substitutes their links inside the
existing B-tree without delivering callbacks between those substitutions.
It preserves character counts, zero-width mark segments and neighboring text.
It does not swap buffers, replace the whole document or roll back a snapshot.

When history is enabled, the operation records one replacement action, rather
than separate delete/insert actions. Undo/redo uses the same storage operation; a
refused reversal keeps its history entry. History queues become coherent before
notifications, so ordinary callback edits remain undoable.

Only after text storage is coherent does it emit `changed`. Ordinary edits made
by those callbacks continue through GTK and are preserved. A nested call to the
experimental operation refuses until notification delivery finishes. A temporary
reference protects the buffer if a callback releases its last external owner.

This is a new, deliberately restricted notification contract. Existing apps cannot
be migrated merely by substituting this call for delete/insert.

## Targeted reversal contract

The buffer retains one fixed-size record (two 129-byte arrays plus scalar
metadata) for the most recent correction. A process-wide, non-reused 64-bit token
binds the request to that buffer; exhaustion refuses. This is an internal
single-threaded toolkit token, not authentication or a cross-process protocol.

Ordinary insert/delete handlers track actual storage edits. Disjoint edits shift
the retained range as necessary. Insertion exactly at the start moves the range;
insertion exactly at the end stays outside it. Overlap invalidates the record,
including when later undo restores identical text. Tracking stops after 256 edits;
unaccounted text revision changes also invalidate it. The explicit `forget` hook
clears both retained strings. A new correction supersedes the previous record;
ordinary correction undo/redo clears it. Failed validation does not mutate text.

Reversal validates the token, tracked revision, expected word and existing safety
gates, then records the inverse as a new chronological history entry. It leaves
later disjoint typing intact; normal undo can undo this reversal, later typing,
and the original correction in order. It does not erase older history or reenable
the old token. Notifications during a pending ordinary mutation cannot start a
transaction. Normal callback edits after transaction commit remain supported.

Field/focus authorization and calling `forget` on ownership loss remain the
adapter's responsibility and are not implemented by this synthetic experiment.

## Explicit limits

- Exact `GtkTextBuffer` type only; no GtkSourceBuffer/EditorDocument subclasses.
  The real `EditorDocument` was tested on 2026-10-29 and does not satisfy the
  required hooks; see the Text Editor README's subclass boundary section.
- Up to four exact GtkTextLayout instances, with no active preedit, are supported
  on buffers of at most 256 real lines. Exact, editable GtkTextView instances
  (at most four) are accepted only when every attached view has a ready AT-SPI
  context; otherwise the call refuses (for example on Broadway, with
  `GTK_A11Y=none`, or with an unrealized context). Tags, commit observers, and
  external legacy insert/delete handlers (including blocked handlers) refuse.
- Ordinary chronological undo/redo is supported experimentally. Active user-action
  groups, irreversible actions and history replay refuse new corrections.
  Targeted reversal is supported for the latest retained correction only, within
  the same restricted buffer. It is not yet bound to application focus/eligibility.
  Replacement strings live in the application's in-memory history until GTK frees
  the entries; there is no disk persistence. A product-specific retention policy
  remains necessary before production integration.
- Alphabetic, valid UTF-8 words only: 1–32 Unicode scalar values, at most 128
  bytes, identical original/replacement scalar counts. This is not a grapheme
  algorithm or evidence for complete product layout support.
- One line, at most 256 characters including its terminator and 512 segments.
  At most 32 prepared segments; copying adjacent text within a touched segment
  is bounded by the line limit. All preflight refusals leave text unchanged.
- The revision uses GTK's existing finite-width text-change stamp. This is not
  the proposed long-lived owner/session/eligibility protocol; rollover, focus,
  IME, protected-field authorization and pending asynchronous work are not solved.
- GTK's process-fatal allocation policy remains. No recoverable out-of-memory
  transaction guarantee is claimed.
- Calls use GTK's single-threaded ownership model. This is callback isolation,
  not a thread-safe operation for concurrent access from multiple threads.

Tests cover the storage, callback and synthetic view-notification boundary only.
They do not establish GtkTextView use in a real application, input-method
integration, ordinary editor correction, browser support or an
upstream-compatible public API.

## Reproduction

Extract the verified archive into a fresh disposable source tree. From TypoMorph:

```bash
python3 integrations/gnome/gtk-transaction/prepare.py /tmp/typomorph-gtk-transaction/gtk-4.22.4
CC=/usr/bin/gcc CXX=/usr/bin/g++ meson setup \
  /tmp/typomorph-gtk-transaction/build \
  /tmp/typomorph-gtk-transaction/gtk-4.22.4 \
  --wrap-mode=nodownload --buildtype=debugoptimized \
  -Dx11-backend=false -Dwayland-backend=true -Dbroadway-backend=true \
  -Dvulkan=disabled -Dmedia-gstreamer=disabled -Dprint-cups=disabled \
  -Dintrospection=disabled -Dbuild-demos=false -Dbuild-examples=false \
  -Dbuild-tests=false -Dbuild-testsuite=true -Ddocumentation=false
ninja -C /tmp/typomorph-gtk-transaction/build -j 4 gtk/libgtk-4.so.1.2200.4 \
  gdk/broadway/gtk4-broadwayd testsuite/gtk/textbuffer
```

The Broadway daemon serves the view-refusal probe; the upstream buffer test
target provides the static-archive link command reused by the layout and
accessibility runners.

Wayland must be compiled in to make GTK's standard Linux AT-SPI backend available.
Layout/view regression tests select Broadway; the accessibility fixture uses a
private headless GNOME Wayland compositor. Neither connects to the desktop.
This builds the library with a restricted backend configuration; it is not the
installed Ubuntu GTK configuration. Meson must find its development dependencies;
automatic fallback downloads are disabled. No `meson install` is part of this test.

On the reference machine nine additional development packages were downloaded
with APT and extracted only under `/tmp/typomorph-gtk-transaction/deps/root`:
libtiff-dev, libjpeg-dev, libjpeg-turbo8-dev, libwebp-dev, libzstd-dev,
liblerc-dev, libdeflate-dev, libsharpyuv-dev, and libdrm-dev. Their temporary
pkg-config prefix/include/library paths were relocated, and required temporary
development symlinks pointed to existing system runtime libraries. Configuration
used `PKG_CONFIG_PATH` pointing to that temporary pkg-config directory. These
are build prerequisites, not additions to the user's system installation.

For a sanitizer build, configure `-Db_sanitize=address,undefined` and rebuild the
same library target. Compile the test with the matching sanitizer flags:

```bash
/usr/bin/gcc -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -I/tmp/typomorph-gtk-transaction/build \
  -I/tmp/typomorph-gtk-transaction/gtk-4.22.4 \
  $(pkg-config --cflags gtk4) \
  integrations/gnome/gtk-transaction/test-splice.c \
  /tmp/typomorph-gtk-transaction/build/gtk/libgtk-4.so.1.2200.4 \
  $(pkg-config --libs gtk4) \
  -Wl,-rpath,/tmp/typomorph-gtk-transaction/build/gtk \
  -o /tmp/typomorph-gtk-transaction/test-splice
```

Verify `ldd` resolves GTK to the temporary build, then run with a fresh temporary
HOME/XDG directory, no display or D-Bus environment, `G_DEBUG=fatal-warnings`,
`ASAN_OPTIONS=detect_leaks=1:halt_on_error=1`, and
`UBSAN_OPTIONS=halt_on_error=1`. Use a 30-second timeout. The test explicitly
enables GTK's text-tree integrity checks. Exit 0 means its listed synthetic
checks passed, not that application integration is ready.

### Private view refusal probe

Build `gdk/broadway/gtk4-broadwayd` with Ninja. Compile `test-view.c` using the
same compiler/linker command above, replacing the input/output test filenames.
Then run:

```bash
python3 integrations/gnome/gtk-transaction/run-view.py \
  --server /tmp/typomorph-gtk-transaction/build/gdk/broadway/gtk4-broadwayd \
  --probe /tmp/typomorph-gtk-transaction/test-view
```

The runner creates private temporary HOME/XDG directories and Unix sockets,
scrubs inherited desktop/bus variables, bounds execution and stops child process
groups. No TCP server or real desktop connection is used. By default this view
probe disables process-exit leak checks because of the recorded font-stack
allocations; `--check-leaks` reenables them. The buffer/history suite keeps leak
checks enabled.

### Sanitizer build caveat on the reference machine

GCC 15.2.0 with sanitizers raised `-Warray-bounds` diagnostics in unchanged
upstream `gtk/gtkcssselector.c` and `gtk/timsort/gtktimsort.c`, which GTK promotes
to errors. For this temporary build only, `-Wno-error=array-bounds` was appended
to the `ARGS` entries for those two objects in generated `build.ninja`:
`gtk/libgtk.a.p/gtkcssselector.c.o` and
`gtk/libgtk.a.p/timsort_gtktimsort.c.o`. Warnings remained visible. No new
transaction source or test received that exception. Meson regeneration removes
this generated-file adjustment. A manual compile alone is insufficient because
Ninja will rebuild the object using its recorded command.

## Recorded results, 2026-09-30

The normal library build succeeded and the initial ten test groups passed.
The final library and test were built with AddressSanitizer/UndefinedBehaviorSanitizer;
all **29 groups passed**, exit 0, with leak detection, fatal GLib warnings and GTK
text-tree integrity checking enabled. Stderr was empty. Dynamic dependency
inspection confirmed the test loaded `/tmp/typomorph-gtk-transaction/build/gtk/libgtk-4.so.1`
and the sanitizer runtimes. System dependencies were not sanitizer-instrumented.

| Coverage | Result |
| --- | --- |
| Bounded word replacement with untouched surrounding lines | Passed |
| Cyrillic, RTL and four-byte Unicode replacements and explicit reverse operations | Passed; reverse operations are not undo |
| Stale revision, mismatched original, invalid UTF-8/NUL, unequal lengths and invalid offsets | Refused without editing |
| All lengths 1–32, both mark gravities at every word offset, and caret position | Preserved |
| Change callback appends text; nested transaction attempted | Append preserved; nested call refused |
| Change callback deletes part of committed correction | Subsequent deletion preserved |
| Modified-state and property-notification callbacks append text | Preserved; callbacks observe committed word |
| Callback releases last external buffer owner | Operation completes; buffer then finalizes |
| Existing commit observer, legacy insert/delete veto handlers including blocked handlers | Refused without editing |
| Irreversible action, active user action, tags, oversized line and excessive segments | Refused without editing |
| Ordinary undo/redo, savepoints, earlier and later typing | Passed |
| Refused undo/redo with legacy veto handler attached afterward | Text and history entry preserved |
| Change and history-availability callbacks edit during reversal | Edits preserved and separately undoable |
| Targeted reversal after later typing; chronological undo/redo of the entire sequence | Passed |
| Disjoint Unicode insertion/deletion before and after the correction | Range tracked; later edits preserved |
| Overlap followed by identical text restoration; stale or foreign buffer token | Refused without editing |
| Explicit cancellation, superseding correction, ordinary correction undo/redo | Old token invalidated |
| Savepoint, callback edit during targeted reversal, refusal followed by safe retry | Passed |
| Reversal requested from a notification before ordinary insertion reaches storage | Refused; pending insertion completes |
| 256 recognized edits, exceeded tracking budget, untracked child-anchor insertion | Boundary succeeds; excess/unknown invalidates |

A separate private Broadway probe created a real GtkTextView and its layout.
Attached-view replacement and targeted reversal both refused without text/revision
changes; after destroying the view each operation succeeded. Its first leak-enabled run reached all
assertions but exited 1: LeakSanitizer reported 261,335 bytes in 6,119 allocations
with Fontconfig/Pango stacks. A separate run with leak detection disabled exited 0
with no runtime diagnostics. ASan/UBSan and fatal GLib warnings remained enabled.
This is refusal evidence, not a leak-clean rendering result or working view
integration. Subclass rejection is not independently exercised. There is no
real-app or composition integration result, performance guarantee, or general
memory-safety proof.

Reapplying `prepare.py` to fresh pinned source reproduced the ten patched source
files byte-for-byte on 2026-10-01; the current twelve-file patch reproduced on
2026-10-28. Repeat and modified-source attempts refuse without changing any input
file. `verify-prepare.py` automates these checks. Prototype input hashes as of
2026-10-28 (after the natural-readiness test):

```text
a11y-splice.inc    4ee98144961bbc8d6a308bcab50dbe11a29229bf43c2366417cb5f10f1c922d7
btree-splice.inc   856440e4292f8c9ab0bd32b50d2f02c8eed105b1590948a2081a3276cea9cf8b
buffer-splice.inc  a2d8b30091309fc94d0d6fa7bc006b85d1bfe61272c9663a7318a8b9aad5e6a4
history-splice.inc de4b156f68c4508430373353208150ad5b19d45a1884175ed24faf1d081374fa
layout-splice.inc  f4e345348c7725a36f6e6ce5ea39cb2f3b97d3834fcd52ceceb63e385535fb3e
view-splice.inc    7ac87afc6195537bda3490131015f729a6d9e0b6e3c851e7a1b00ec14d052c3a
prepare.py         156dbdabdc06822334b1472987e7e3dd5b6a87d2c9b4bed8c0716bfac8e14047
run-a11y.py        8bede216a61be31fab7d798d23918c29811733e95afd08d0093376f531da1850
run-layout.py      9e70e80ade48eb73b33e75eb3e391e4c78f14c4e2c43de6dece8aca2650e95cb
run-view.py        7c5b9660c2b567e6f46935fb4073ecb44af0b480b690abe92f81c2cbd59c9e3f
test-a11y-natural.c 36ecd4c358b3cd51f807f4787616368440d700d63a7db2c40c41173585416f27
test-a11y-view.c   06cc4a3994ded68672151473a0474ded6f24b4e758baaf625966a49af626137c
test-a11y.c        e9fba1223748a1ea6def0f6bead52ea877472689fb9909d5f758bec3215d2ba3
test-history.inc   008d81e96579b9813982503974e036c917c568f34221e231b5439530810108f0
test-layout.c      38381bf4423c8ca365ef1319f5a2bae421511a09641b13637e0e934e8e94c9cc
test-revert.inc    f54da3a4baf229cffa32c48255164c92d44e5b8d4d3c8c3f91e3df7eeabe1873
test-splice.c      8737c0a15c40f170947f929e0cb0a4f9c1a00e8d66c35dd1caa3e7ec02005afc
test-view.c        40ca0f3748c594ce96aadb5d1cd42d65e4c15f6384680e4fa8797a1c4533c54a
verify-prepare.py  206e4c8800e8444d334bc51c695a619dcbf3e58c1fb9e20d6426822f2594c2dd
```

## Internal layout transaction milestone, 2026-10-01

The earlier cache-only draft correctly refused because GtkTextLayout registered
four legacy edit observers. The current patch explicitly migrates only GTK's own
four observers into internal before/after storage hooks. It does not whitelist,
disconnect or block application handlers at runtime. Ordinary typing continues
through the same cache invalidation and cursor-line maintenance functions.

Transactions snapshot up to four layouts with strong references, invalidate their
caches while notifications are deferred, and publish only after storage and
history are coherent. All layouts are thawed before callbacks. A callback may
edit text or detach another layout; snapshot lifetime and buffer rechecks preserve
those edits and avoid publishing to a detached layout. Retired Pango displays use
GTK's existing idle-purge queue, not synchronous destruction during commit.

Six fixed-input `test-layout.c` groups pass with ASan/UBSan and fatal GLib warnings:

- Two warm Pango caches observe correction, targeted reversal and ordinary undo/redo.
- Callback edits survive and remain undoable; detaching a second layout is safe.
- Bidirectional paragraph caches update on correction and undo.
- Excessive layout count, more than 256 lines and active preedit refuse without
  mutation; the exact 256-line boundary permits correction.
- External legacy observers, including blocked observers, still refuse.
- Ordinary multiline insertion/deletion refreshes caches before a later correction.

All 29 buffer/history groups also pass again with leak detection enabled. Twenty
upstream GTK textbuffer tests pass against the modified build on private Broadway,
including normal editing, tags, marks, clipboard, anchors and undo. The existing
GtkTextView refusal probe still passes. Layout/view/upstream display tests disable
process-exit leak checks for the previously recorded font-stack limitation; they
retain ASan/UBSan. There is no rendered application or accessibility result yet.

Because private layout symbols are not exported from the shared library, the
runner links the disposable GTK static archive using its own test dependency
list. It runs with isolated HOME/XDG paths and no display or session bus:

```bash
python3 integrations/gnome/gtk-transaction/run-layout.py \
  --build /tmp/typomorph-gtk-transaction/build \
  --source /tmp/typomorph-gtk-transaction/gtk-4.22.4
```

## Accessibility snapshot notification milestone, 2026-10-01

`a11y-splice.inc` adds an internal AT-SPI-only primitive that queues deletion and
insertion events from explicitly supplied original/replacement bytes. It never
rereads the widget for removed text. Its bounds match the storage experiment:
1–32 alphabetic scalars, at most 128 UTF-8 bytes per payload, equal scalar count,
with signed AT-SPI offset overflow refused. An unrealized/unregistered or closed
context refuses. Other accessibility backends have no new support.

This primitive only publishes notifications. It neither validates field ownership
nor performs a storage transaction. A caller must capture the original payload
before mutation, validate eligibility and context identity, and publish immediately
after commit before other callbacks can edit. (GtkTextView wiring was added
later; see the attached-view milestone below.)

The two D-Bus messages are queued in order without UI main-loop dispatch between
them. This is not atomic delivery: connection loss can prevent either message
from reaching listeners. Notification failure must never trigger rollback of
already committed application text. A production recovery contract remains open.

`run-a11y.py` builds `test-a11y.c`, starts a private headless GNOME Wayland
compositor, private IBus/session/accessibility services and the real
`/usr/libexec/at-spi2-registryd`. Broadway is unsuitable for this fixture because
GTK only constructs its AT-SPI context for Wayland/X11 displays. It passes only
fixed synthetic text through the actual GTK backend. It scrubs inherited desktop
variables, uses Unix sockets, bounds waits and cleans child processes. It requires
`at-spi2-core`, `gnome-shell`, `ibus`, and the Wayland build dependencies in addition to the earlier GTK
prerequisites. The probe is not an ordinary application correction test or a
screen-reader usability test. Font-stack leak checking remains disabled; ASan,
UBSan and fatal GLib warnings remain enabled for the probe. Infrastructure services
use their ordinary warning policy; system dependencies are not instrumented.

All four fixed-input scenario groups passed against the actual GTK AT-SPI backend
and real registry/listener on the private Wayland session:

- Removed `ghbdtn` and inserted `привет` arrive in order at character offset 2,
  with character length 6, while the buffer already contains the replacement.
- The exact 128-byte / 32-scalar supplementary-Unicode boundary passes.
- Invalid UTF-8, embedded NUL, null/oversized/unequal payloads and offset overflow
  refuse without producing events. An ordinary edit from the deletion listener
  appears after the already queued replacement insertion, preserving its payload.
- Unrealized contexts refuse publication.

The fixture deliberately supplies fixed payloads without using the storage splice;
this isolates protocol behavior. It does not prove field authorization, atomic
storage-to-notification integration, disconnect recovery or screen-reader behavior.
The 29 storage/history, six layout and attached-view refusal checks also pass after
the rebuild. The unchanged upstream buffer suite is checked separately.

```bash
python3 integrations/gnome/gtk-transaction/run-a11y.py \
  --build /tmp/typomorph-gtk-transaction/build \
  --source /tmp/typomorph-gtk-transaction/gtk-4.22.4
python3 integrations/gnome/gtk-transaction/verify-prepare.py \
  --archive /tmp/typomorph-gtk-transaction/gtk-4.22.4.tar.xz \
  --compare-source /tmp/typomorph-gtk-transaction/gtk-4.22.4
```

## Attached GtkTextView AT-SPI integration, 2026-10-28

`test-a11y-view.c` closes the gap left by the direct-publisher fixture. It
creates a real GtkTextBuffer containing fixed text `xxghbdtn`, attaches a
GtkTextView in a presented, focused GtkWindow on the private headless GNOME
Wayland session, waits until the view is realized and mapped, registers a real
AT-SPI `object:text-changed` listener with the private registry, and then calls
only the prototype API (`gtk_text_buffer_typomorph_splice`, undo/redo and
`gtk_text_buffer_typomorph_revert`). It never calls the AT-SPI publisher directly.

```bash
python3 integrations/gnome/gtk-transaction/run-a11y.py \
  --build /tmp/typomorph-gtk-transaction/build \
  --source /tmp/typomorph-gtk-transaction/gtk-4.22.4 \
  --test test-a11y-view.c
```

All six groups passed (exit 0) in four consecutive runs, with ASan/UBSan and
fatal GLib warnings; font-stack leak checking is disabled as before:

- Splice `ghbdtn` → `привет` at character offset 2: the listener receives exactly
  `delete(2, 6, "ghbdtn")` then `insert(2, 6, "привет")`. When the first event
  arrives, the buffer already holds `xxпривет` at the committed revision. The
  removed payload is the pre-commit snapshot, not reread widget text.
- Undo and redo replay publish the inverse and repeated payload pairs in order.
- A `changed` handler that appends `x` after commit produces
  `insert(8, 1, "x")` strictly after the replacement pair, showing publication
  precedes application callbacks. The callback observes the full replacement.
- An ordinary deletion through the migrated buffer path reports
  `delete(8, 1, "x")` with the removed text.
- A second attached GtkTextView whose AT-SPI context is unrealized makes the
  splice refuse: text, revision and `changed` count are unchanged; no events.
- After the primary view's context is unrealized, splice, undo of a retained
  replacement and targeted reversal all refuse with text and revision unchanged;
  the undo entry and correction token are retained and no event is emitted.
  `notification_failed` stays false throughout.

Observed limitation: presenting and focusing the window did not realize the
view's AT-SPI context in this headless session. This fixture therefore realizes
the view context explicitly with GTK's internal `gtk_at_context_realize()` and
prints `VIEW_CONTEXT_REQUIRED_EXPLICIT_REALIZE`, as `test-a11y.c` also does. It
establishes the notification path, not natural readiness; see the next section.
(An earlier draft of this fixture reported that a client `GetChildren` query
failed with `Object does not exist`. That was a defect in the draft probe, which
used the wrong discovery path, not evidence about GTK. The probe was removed.)
Any view without a ready context makes correction refuse.

The 29 buffer/history groups (leak checks enabled, empty stderr), six layout
groups, the four direct AT-SPI groups, the Broadway view-refusal probe and the
20 unchanged upstream textbuffer tests (private Broadway, `GTK_A11Y=none`) also
passed against the same rebuilt library. On Broadway the view no longer refuses
because of legacy handlers; it refuses because no AT-SPI context is available.
`verify-prepare.py` passed: twelve files reproducible; repeat and modified-source
attempts refuse without changes.

Not covered: two or more simultaneously ready views, D-Bus disconnect or
`notification_failed` recovery, screen-reader behavior, input methods with
active composition, GtkSourceBuffer subclasses, field/focus ownership, or any
real application. Ordinary-edit notification now fires from the buffer's
default handlers; for insertion this precedes `changed`, whereas upstream's
`connect_after` handler ran after it. Only the two ordinary events above were
checked against a listener. This is synthetic prototype evidence only.

## Natural AT-SPI readiness of an attached GtkTextView, 2026-10-28

Question: does an attached GtkTextView's AT-SPI context become ready without
`gtk_at_context_realize()` in the private Wayland/AT-SPI session?
`test-a11y-natural.c` never calls that function. It runs **one scenario per fresh
process** (`--scenario`), each in a newly started private session. "Cold" below
means exactly that: the first and only GTK window of the process, with no earlier
window, view or AT-SPI client traversal in it. An earlier draft ran the
late-added scenario first in the same process, which made the primary
measurement non-cold; that was corrected.

```bash
python3 integrations/gnome/gtk-transaction/run-a11y.py \
  --build /tmp/typomorph-gtk-transaction/build \
  --source /tmp/typomorph-gtk-transaction/gtk-4.22.4 \
  --test test-a11y-natural.c --scenario primary      # or late-added, listener-gap
```

All text is fixed and synthetic (`xxghbdtn`). Exit codes: 0 pinned outcome
observed and verified; 2 usage or boundary violation; 3 readiness not
established (refusal verified: splice returns false, text and revision unchanged,
no failure flag, no event); 4 ready but GTK's listener table never populated;
5 primary: passive readiness reached (the pinned claim changed); 7 listener-gap:
the view was ready but the listener table was already known (gap not observed).
A deadline that expires before the view is ever ready exits 3, not 7. A failed
event or text assertion aborts, also non-zero.

| Scenario | Exit 0 only if | Enforced |
| --- | --- | --- |
| `primary` | passive wait does **not** make the view ready; an unready splice refuses unchanged; a client walk (Registry → application → window `GetChildren`) lists the view and makes it ready; ordered `delete(2,6,"ghbdtn")`/`insert(2,6,"привет")` arrive with the buffer already `xxпривет` at the first event | Yes |
| `late-added` | a view added to an already presented, mapped window becomes ready with no client walk, and the same events verify | Yes |
| `listener-gap` | the ready-without-listeners state is observed, the splice commits with no event and no failure flag, and a later splice delivers events (a characterization, see below) | Yes, as a characterization |

The rare `listener-gap` exits are covered by two harness self-tests, run with
`--scenario`: `listener-gap-timeout` (1 ms deadline, expects exit 3) and
`listener-gap-settled` (waits for the listener table before sampling, expects
exit 7). They are not product evidence. An earlier version removed the deadline
timer unconditionally after it had already fired; GLib then raised a critical
(`Source ID … was not found`), which aborted the run with exit 250 under
`G_DEBUG=fatal-warnings` instead of the documented code. The timer is now removed
only if it has not fired; 3 of 3 runs of each self-test give the expected exit.

Failure propagation was checked with negative controls in a scratch copy: a wrong
first-event text (late-added and primary), a wrong insert offset (late-added), a
forced not-ready (late-added) and a wrong gap expectation (listener-gap) each
exited 250, while the unmutated copy exited 0.

GTK 4.22.4 mechanism, from the source read and the test: a window realizes its
own AT context when presented; a child context is not realized with its widget.
It is realized when an AT client calls `GetChildren`/`GetChildAtIndex` on the
parent, or when a child is added to a parent whose context is already realized
(`gtk_accessible_update_children`). Other triggers were not enumerated. A
context has no object path until realized.

Results (6 runs per scenario, all exit 0, plus the earlier cold runs):

- `primary`: window context realized at present; the view's context stays
  unrealized after presentation, mapping and a 3 s passive wait
  (`PASSIVE_READINESS NOT_REACHED`, 6/6). After the client walk it is ready
  (6/6) and the splice events verify.
- `late-added`: ready without a walk, 6/6, but only **3.15–3.42 s after the
  window was presented**, not immediately. A previous 3 s budget would have been
  marginal; the test now allows 8 s and prints the elapsed time. Registration
  completion (`Embed` round trip) takes about 3 s here; why was not investigated.
- Conclusion: for a view present at first presentation, readiness requires an AT
  client traversal. A view added later becomes ready when GTK finishes root
  registration, without a walk. A view reached by neither stays unready and
  correction refuses by design. Whether a real session performs the traversal was
  not tested; the test plays the client.

### Listener-table startup gap (reproduced, characterized, not fixed)

GTK sets its "can use listeners" flag but fills the listener table from an
asynchronous registry reply. While the table is empty, `has_event_listeners` is
false and the publisher returns success **without emitting**.

`listener-gap` detects this by state, not by sleeping: it presents the window and
adds the view before any main-loop dispatch, dispatches one source at a time, and
splices in the same step in which the view is first ready while the root reports
no listener. If the state is not seen it exits 7, which is a non-reproduction and
not a pass. Observed in 6/6 runs (plus 5/5 during development):

- The root is assigned within about 10–16 µs of the first dispatch with the
  flag true (GTK's "chatty" default). About 2.87–3.07 s later the flag turns
  false and the view becomes ready **in the same dispatch step**. The table was
  learned 1.4–5.0 ms after that.
- In that interval the splice returned true and committed (`xxпривет`, revision
  changed); `notification_failed` stayed false; no AT-SPI event was delivered.
  The reverse splice afterwards delivered the expected two events.

This asserts the prototype's current behavior; it does not endorse it. Events in
this interval are silently lost and nothing records the loss. Why the flag change
and readiness coincide, and why registration takes about 3 s, were not
established. The earlier note of a transient "listeners = 0" in a long-running
process is consistent with this same interval (it appeared as the old 3 s passive
wait ended near registration completion) but that link is inferred, and the cold
`primary` scenario did not show it. The other scenarios wait for the table
(`LISTENERS_KNOWN_AFTER_WAIT_MS`, 0–1 ms) before splicing, so they do not cover
this hazard; only `listener-gap` does, and only for a view that is ready at the
instant the root registers.

Not established: real assistive technology or desktop session, Orca behavior,
multiple views, applications keeping views reachable, and any recovery contract
for suppressed notifications. Synthetic prototype evidence only.

Regression results on the same library (no patch input changed; pinned-source
reproduction passed, 12 files): explicit-realize fixture 6 groups, direct
publisher 4 groups, 29 buffer/history groups (empty stderr), 6 layout groups,
Broadway refusal probe, and 20 upstream textbuffer tests all passed.

### Defining evidence for an explicit "unknown" refusal (not implemented)

No GTK change was made for the listener gap. GTK's `has_event_listeners` conflates
two different states: "listener tracking enabled but the registry reply has not
arrived" (unknown) and "table populated, no listener" (nobody to tell). A refusal
or a flag in the unknown state would need this evidence before any code change:

- A way to distinguish the states, which requires GTK-internal access (a private
  accessor or a patch to the root), since neither is observable through public API.
- The existing deterministic reproduction (`listener-gap`) turned into a test that
  expects a pre-commit refusal in the unknown state, with text, revision, history
  and callbacks unchanged, and expects a splice to remain allowed when the table is
  populated and empty, so users without an assistive technology are not blocked.
- A policy choice, not a technical one: refuse before commit (no lost text, brief
  refusals of about 1.4–5 ms in the cold window), or commit and record that
  notification delivery was not possible. Committed text must never be rolled back.

Text-transaction authorization (owner, revision, field safety, subclass hooks) and
accessibility event delivery are separate conditions and must stay separate: a
splice must not be authorized merely because events could be sent, and a missing
event must not silently stand in for an authorization failure.

## Next implementation boundary

Retain the original failing real-editor tests. A single attached GtkTextView
now publishes snapshot AT-SPI events for correction and history replay in the
synthetic fixture above, once its context is ready, and context readiness
depends on an AT client traversal or a late-added view (see the natural-readiness
section). Next, decide how a correction path may legitimately learn that a view is
ready or must refuse, decide how to treat the reproduced startup listener gap
(events silently lost), then cover multiple ready
views, observer edits across views, connection loss and `notification_failed`
recovery, and a real session, then GtkSourceBuffer subclass semantics.
The correction-record cancellation hook must also be bound to actual field/focus
ownership before application use.
Do not remove refusal guards merely to make a real editor accept the primitive.
The isolated prototype approval does not authorize installing a GTK fork, changing
the production safety gate or claiming unmodified applications are supported.
