# Isolated GTK storage transaction prototype

Owner approved this experiment on 2026-09-30. **Not a production adapter, GTK
distribution, or fix for unmodified applications.** No installed library is
changed. The earlier editor and search-replacement failure fixtures remain intact.

## Source and artifacts

Official release archive:
[GTK 4.22.4](https://download.gnome.org/sources/gtk/4.22/gtk-4.22.4.tar.xz).
SHA-256: `51bd9f60c7d23a665a556c7364c21fb2e4e282566b3e7e092455e8f910330893`.

- `prepare.py` checks six exact release-source file hashes before patching a
  disposable source tree under `/tmp`. It refuses an already modified tree.
- `btree-splice.inc` supplies a new internal segment replacement operation.
- `buffer-splice.inc` supplies an explicitly experimental buffer entry point.
- `history-splice.inc` adds a standalone chronological undo/redo action.
- `test-history.inc` adds history, savepoint and notification tests.
- `test-revert.inc` adds targeted reversal and mutation-tracking regressions.
- `test-view.c` and `run-view.py` check attached-view refusal on private Broadway
  and D-Bus Unix sockets, without connecting to the desktop.
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
- No attached views, tags, commit observers, or connected legacy insert/delete
  handlers, including blocked handlers. These configurations refuse.
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

Tests cover the storage and callback boundary only. They do not establish a
working GTK view, input-method integration, ordinary editor correction, browser
support or an upstream-compatible public API.

## Reproduction

Extract the verified archive into a fresh disposable source tree. From TypoMorph:

```bash
python3 integrations/gnome/gtk-transaction/prepare.py /tmp/typomorph-gtk-transaction/gtk-4.22.4
CC=/usr/bin/gcc CXX=/usr/bin/g++ meson setup \
  /tmp/typomorph-gtk-transaction/build \
  /tmp/typomorph-gtk-transaction/gtk-4.22.4 \
  --wrap-mode=nodownload --buildtype=debugoptimized \
  -Dx11-backend=false -Dwayland-backend=false -Dbroadway-backend=true \
  -Dvulkan=disabled -Dmedia-gstreamer=disabled -Dprint-cups=disabled \
  -Dintrospection=disabled -Dbuild-demos=false -Dbuild-examples=false \
  -Dbuild-tests=false -Dbuild-testsuite=true -Ddocumentation=false
ninja -C /tmp/typomorph-gtk-transaction/build -j 4 gtk/libgtk-4.so.1.2200.4
```

This builds the library with a minimal backend configuration; it is not the
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

Reapplying `prepare.py` to fresh pinned source reproduced the six patched source
files byte-for-byte. A second application refused and left all six unchanged.
Final prototype input hashes:

```text
btree-splice.inc   baf69852005f4633903c5776d4d6a04f879b0e98887bf8627f4e4cce77167837
buffer-splice.inc  c74530baed830f9fdda7124a6c7adcd5e8b6cadeb3b00f4988c8dbbde18883ee
history-splice.inc de4b156f68c4508430373353208150ad5b19d45a1884175ed24faf1d081374fa
prepare.py         6295973c04c820821ca84821612e0ee96d452b2c532b490e2b18f93c08a15480
run-layout.py      a140cbdbfb6cc438dd72f782fd677b6756d4fe9e3ca6d2e113b621cbf4c1fdd4
run-view.py        7c5b9660c2b567e6f46935fb4073ecb44af0b480b690abe92f81c2cbd59c9e3f
test-history.inc   008d81e96579b9813982503974e036c917c568f34221e231b5439530810108f0
test-layout.c      a78f3efa985030ffc28601614c77b81e72fabe7b6b97e659858f10c09374e146
test-revert.inc    f54da3a4baf229cffa32c48255164c92d44e5b8d4d3c8c3f91e3df7eeabe1873
test-splice.c      8737c0a15c40f170947f929e0cb0a4f9c1a00e8d66c35dd1caa3e7ec02005afc
test-view.c        40ca0f3748c594ce96aadb5d1cd42d65e4c15f6384680e4fa8797a1c4533c54a
```

## Layout observer investigation, 2026-10-01

A temporary cache-invalidation batching draft was compiled but did not pass the
first positive layout test: replacement correctly refused. Inspection showed
that GtkTextLayout itself installs four legacy insert/delete handlers for cache
and cursor-line maintenance, even without GtkTextView. GtkTextView adds separate
accessibility handlers. Cache invalidation alone therefore does not satisfy the
required observer contract. The draft was removed; the original six-file patch
and its recorded hashes were restored. No refusal gate was bypassed.

`test-layout.c` now contains two explicit **known-gap/refusal** tests. They attach
real internal GtkTextLayout objects, warm their Pango caches, verify that legacy
observers are registered, and assert that correction/reversal leaves the text,
revision and cached displays unchanged. Detaching the layouts permits the same
operation. Both tests pass with ASan/UBSan and fatal GLib warnings; font-stack
process-exit leak checking is disabled, as for the view probe. This is not working
view integration or visual rendering evidence.

Because private layout symbols are not exported from the shared library, the
runner links the disposable GTK static archive using its own test dependency
list. It runs with isolated HOME/XDG paths and no display or session bus:

```bash
python3 integrations/gnome/gtk-transaction/run-layout.py \
  --build /tmp/typomorph-gtk-transaction/build \
  --source /tmp/typomorph-gtk-transaction/gtk-4.22.4
```

## Next implementation boundary

Retain the original failing real-editor tests. Next, establish view/subclass
notification semantics before permitting those configurations through the new API.
This requires a compatible layout observer contract and correct GtkTextView
accessibility notifications; invalidating cached layouts alone is insufficient.
The correction-record cancellation hook must also be bound to actual field/focus
ownership before application use.
Do not remove refusal guards merely to make a real editor accept the primitive.
The isolated prototype approval does not authorize installing a GTK fork, changing
the production safety gate or claiming unmodified applications are supported.
