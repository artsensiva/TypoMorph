# Real Text Editor experiment results

Date: 2026-09-30. **Compiled and exercised in an isolated native Wayland session.**
This is an application-specific development experiment, not a released integration.

## Build and isolation

Official GNOME Text Editor 50.1 commit:
`d1bc58ff790267224be1536a86e1af236a9f7ea7`.
The baseline checkout is unmodified; all earlier pinned input hashes match.
Final development patch SHA-256:
`84f533bc0d0fd126de3c18e035cb515c2a48fe3bc7c1b50aec9d62c5e85d5c26`.

Both complete builds succeeded with GCC 15.2.0, GLib 2.88.0, GTK 4.22.4,
GtkSourceView 5.18.0, libadwaita 1.9.1 and libspelling 0.4.9. Each passed its two
upstream desktop/appstream validation tests. Those tests do not validate editing.
The baseline survived its five-second isolated startup check; no baseline document
comparison was performed. Build directories are
`/tmp/typomorph-editor-baseline-gcc-build` and
`/tmp/typomorph-editor-patched-gcc-build`.

The initial default-compiler configuration used a Zig `cc` wrapper. The successful
builds explicitly selected `/usr/bin/gcc`. Upstream also requires `itstool`, absent
from the original dependency plan. Ubuntu `itstool` and `python3-libxml2` packages
were downloaded and extracted only into `/tmp/typomorph-editor-build-tools/root`;
no additional system package installation or application installation was done.
PATH and PYTHONPATH pointed to that temporary tool tree while configuring/building.
No automatic Meson dependency downloads were allowed.

Every native case used a fresh private D-Bus session, fresh XDG directories and
headless GNOME Shell 50.1. The runner checked that RemoteDesktop belonged to its
compositor child before sending fixed synthetic `ghbdtn` keys. It did not set or
inherit the host display/bus. The patched editor read bounded synthetic content
in its own document, emitted fixed markers and quit. Private state was disposable.
The installed editor, user's active desktop, and TypoMorph production gate are
unchanged. This isolation is not a general filesystem sandbox.

## Results from the final patch

| Case | Observed result | Meaning |
| --- | --- | --- |
| `normal` | `ghbdtn` became `привет`; ordinary undo restored `ghbdtn`; redo restored `привет`; exit 0 | Fixed-case range replacement and ordinary history work in the actual app |
| `callback-begin-edit` | Synchronous begin-action callback appended `x`; correction refused before deletion; `ghbdtnx` retained; exit 0 | This pre-deletion conflict is preserved |
| `gap-edit` | Fixed competing edit after deletion left `x`; fault detected; exit 2 | Known unsafe partial edit, not a safety pass |
| `callback-delete-edit` | Actual post-delete GTK callback inserted `x`; fault detected, `x` retained; exit 2 | Callback-driven partial edit reproduced in the real app |

The final four cases were repeated after diagnostic cleanup with the same outcomes.
The two nonzero exits must not be relabeled as passing safety tests. Successful
recognition of an unsafe schedule is useful evidence, not acceptable product behavior.

## Defect found and fixed during runtime validation

The initial `normal` case always refused after beginning the undo group.
Content-free diagnostics identified `notify::can-undo` as the invalidation source.
GTK changes undo availability during begin/end-user-action without changing field
ownership or content. The owner now excludes only document `can-undo`/`can-redo`
notifications from epoch invalidation. Actual undo/redo changes still invalidate
through document changes/marks, and content, revision and field checks remain.
The successful normal case and conflicting begin-action case were rerun after this
fix. Other notification/selection/composition guards were not weakened.

The runner now prints only exact allowed markers, making refusals inspectable
without echoing arbitrary application output. Ten pure Python assessment/isolation/
cleanup tests pass, including marker redaction. Three patch integrity regressions
and forward/reverse exact restoration also pass. These are separate from native
application results.

## Release implications and next work

A grouped GtkTextBuffer delete/insert is **not an isolated replacement transaction**.
The normal case establishes neither general within-word detection nor safety under
other schedules. This experiment does not implement targeted reversal, lossless
concurrent edits, composition eligibility, multi-field/browser coverage, account
integration or OS layout switching. Unmodified Text Editor and Chrome remain unfixed.

Keep automatic production correction disabled. The next native task is to establish
a supported app/toolkit range replacement operation that preserves concurrent edits,
or explicitly document the missing primitive and obtain an architecture decision.
Do not silently ship a permanent editor/GTK fork or a global key-replay fallback.

Follow-up [GTK API probe](GTK_REPLACEMENT_API_RESULTS.md) evaluates public search
replacement and commit notifications. It reproduces additional safety failures
on fixed toolkit buffers; it is not another real-application test or a fix.
