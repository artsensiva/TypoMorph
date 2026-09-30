# GTK replacement API feasibility

Date: 2026-09-30. Installed GTK 4.22.4 and GtkSourceView 5.18.0.
Evidence: official API/source inspection and executed synthetic toolkit tests.
This is not a fix for unmodified Text Editor or Chrome.

## Supported API evaluation

[GtkTextBuffer commit notifications](https://docs.gtk.org/gtk4/method.TextBuffer.add_commit_notify.html)
observe individual edits, with mutation forbidden inside the notification.
They can support revision/range tracking, but provide neither a combined
replacement operation nor permission to repair a failed edit.

[GtkSourceSearchContext.replace](https://gnome.pages.gitlab.gnome.org/gtksourceview/gtksourceview5/method.SearchContext.replace.html)
is a public single-call replacement API. The
[5.18.0 implementation](https://raw.githubusercontent.com/GNOME/gtksourceview/5.18.0/gtksourceview/gtksourcesearchcontext.c)
still groups a deletion and insertion. Its literal replacement path reports
success without checking whether signal handlers prevented either mutation.
Its documented contract does not promise isolation.

## Executed probe

`integrations/gnome/text-editor/probe_search_replace.c` creates a real
GtkSourceBuffer containing fixed synthetic `ghbdtn` and calls the installed
search replacement API with `привет`. No display, input capture or application
window is used. Commit notifications count actual edits without copying payloads.
The oracle reads at most 12 characters from its own synthetic buffer and prints
only fixed status markers. The probe accepts no arbitrary input text.

| Case | Observed buffer/result | Exit and interpretation |
| --- | --- | --- |
| `normal` | Replacement works; ordinary undo restores original; redo restores correction | 0: normal observation only |
| `veto-insert` | Insert signal stopped; original deleted; API returns true without error | 2: unsafe partial replacement |
| `veto-delete` | Delete signal stopped; original and replacement both remain; API returns true without error | 2: unsafe duplicate text |
| `callback-delete-edit` | Callback observes empty intermediate buffer and appends `x`; final result retains correction and `x` | 2: lack of isolation observed; this case alone does not lose competing text |

The callbacks exercise permitted signal interception/reentrancy. They are fault
injection, not a claim that ordinary Text Editor invokes these exact callbacks.
The post-delete callback revalidates its iterators. No handlers are suppressed
to manufacture success, and no full-buffer rollback is attempted.

Compiled with GCC, C11, `-Wall -Wextra -Werror`. All four cases produced the
expected observations without stderr under `G_DEBUG=fatal-warnings`, with a
fresh temporary home/config/cache/data directory, no display or D-Bus variables,
and a ten-second per-process timeout. Unsafe outcomes remain failures of the
product safety requirement, even though their reproductions are reliable.

Reproduction from the repository root with installed development dependencies:

```bash
probe_dir=$(mktemp -d /tmp/typomorph-buffer-probe-XXXXXX)
/usr/bin/gcc -std=c11 -Wall -Wextra -Werror \
  integrations/gnome/text-editor/probe_search_replace.c \
  -o "$probe_dir/probe" $(pkg-config --cflags --libs gtksourceview-5)
for case_name in normal veto-insert veto-delete callback-delete-edit; do
  env -i PATH=/usr/bin:/bin HOME="$probe_dir" \
    XDG_CONFIG_HOME="$probe_dir" XDG_CACHE_HOME="$probe_dir" \
    XDG_DATA_HOME="$probe_dir" GSETTINGS_BACKEND=memory \
    G_DEBUG=fatal-warnings LC_ALL=C.UTF-8 \
    timeout 10 "$probe_dir/probe" "$case_name"
  probe_status=$?
  printf '%s exit=%s\n' "$case_name" "$probe_status"
done
```

Expected exits in order: 0, 2, 2, 2. Any exit 1, timeout, warning or crash is
unexpected. The temporary directory contains the executable; no installation
occurs. The probe is separate from the real editor patch and native runner.

## Consequence and architecture boundary

Neither evaluated API supplies the needed replacement transaction. Post-edit
validation detects damage too late. Signal blocking, whole-buffer reset,
synthetic backspaces and unconditional undo cannot substitute for preserving input.

A cooperating app/toolkit needs an operation with these properties:

1. Validate owner, eligibility, revision and bounded range before mutation.
   Refusal leaves the document and undo history unchanged.
2. Commit range replacement, marks and undo record before delivering callbacks
   that can mutate the document. Observers never see a half-replaced word.
3. Preserve legitimate subsequent/reentrant edits in a defined order. Nested
   correction may refuse; ordinary edits must not be discarded.
4. Return a receipt only after commit. Support targeted reversal preserving
   later disjoint edits; refuse reversal across overlapping changes.
5. Preserve application signals, lifecycle, tags and undo behavior, or identify
   necessary application changes. Calling internal buffer methods while bypassing
   signals is not a proven implementation.

The existing app-only experiment cannot supply these guarantees through the
evaluated APIs. The recommended next architectural option is a separate temporary
toolkit transaction prototype to test feasibility and API compatibility. This
extends the earlier app-only patch boundary; the owner approved the isolated
prototype on 2026-09-30. It does not imply shipping a permanent GTK fork. Upstream
acceptance/distribution and other toolkits remain separate dependencies; no
upstream submission has been made.

Production correction remains disabled. An experimental toolkit change would
still not establish Chrome, Windows or macOS support.

The first approved [isolated GTK storage prototype](../integrations/gnome/gtk-transaction/README.md)
now includes ordinary undo/redo and bounded targeted reversal preserving disjoint
edits: 29 synthetic groups passed under sanitizers with
leak detection. It deliberately refuses views, subclasses and legacy edit observers.
A private Broadway view probe verifies correction/reversal refusal without editing; its earlier leak-enabled
run reports font-stack allocations, recorded in the prototype evidence. This demonstrates
a narrower storage/callback boundary, not a replacement for the missing complete
application transaction contract.
