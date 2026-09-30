# Native GTK/IBus lifecycle experiment

Date: 2026-09-25. Approved isolated experiment; no production architecture chosen.

## Actual route and isolation

GTK 4.22.4 Gtk.Text widgets -> explicitly selected GTK IBus module -> private
IBus daemon -> dedicated lifecycle fixture. GTK uses the Broadway backend with
an HTTP Unix socket inside a temporary directory; no browser connects and no TCP
listener is requested. The display has its own runtime socket. Session bus and
XDG directories are private; desktop display/socket variables are removed.
No desktop engine is registered, installed extension modified, system package
installed, or production safety gate opened. This is process/environment isolation,
not a sandbox against malicious test code.

New files under integrations/ibus: run_gtk_tests.py (services and cleanup),
gtk_fixture.py (synthetic stimulus engine), test_gtk_lifecycle.py (real widgets).
The fixture is deliberately separate from the TypoMorph engine and eligibility
protocol. No idealized Begin/Confirm notifications are supplied. Gtk.Text drives
native focus transitions, including normal and password input-purpose metadata.
Seed explicitly injects fixed COMMIT-mode preedit `ghb`; a normal-field `x` key
causes a fixed delayed echo. Neither is production correction or a safety grant.
Keys are supplied through the widget controller's actual IMContext.filter_key,
not a physical device or the full window event dispatch path. Only test-owned
synthetic widget contents are read by assertions, held in RAM, and printed on
assertion failure. No personal application contents are involved.

## Results

- Synchronous mode (IBUS_ENABLE_SYNC_MODE=1, matching the reviewed GTK4 client
  default): all 11 checks passed.
- Forced asynchronous mode (IBUS_ENABLE_SYNC_MODE=0): 9 passed, 2 failed.
  Failures reproduced with longer waits and a fixture control round trip.
- Two widgets expose distinct native IBus contexts; returning to a widget reuses
  its context. This differs from the earlier real-session shared-context result
  and must not be generalized to GNOME's mediated Wayland route.
- Native focus loss preserves seeded preedit once in the original widget, including
  rapid A/B/A, transition to a password field, and fixture process disconnect.
- Native normal-field commit replaced only the selected synthetic range.
- Input immediately after a field change preserved the old draft and placed the
  new fixed echo in the new widget in both modes.
- Password input produced no fixture echo; GTK's own fallback inserted the test
  character. The first assertion incorrectly assumed empty text; it was corrected
  to measure the fixture's echo count separately from native fallback. This does
  not prove that no protected input ever reaches an input-method process.

### Outstanding reply: unsafe in forced async mode

The fixture pauses a key reply for 50 ms. The test sends a key for widget A,
switches focus to B without draining the GTK main context, then waits. In forced
async mode A stayed empty and B received `x`, although A received the original
key-filter call. The safety assertion remains failing, not marked expectedFailure
or weakened. This is a concrete wrong-target result for this isolated route and
schedule; it does not establish the same behavior in the user's desktop apps.
Synchronous mode serializes that call, so its passing result does not demonstrate
that it handled an outstanding reply across focus change.

### Reset: async draft remains preedit

The async reset check expected `ghb` to be committed once. Instead widget text
remained empty and get_preedit_string still returned the seeded draft. This is
unresolved disposition under the test's contract, not demonstrated text loss.
The stimulus engine intentionally has no reset policy, exposing the client-side
behavior; a production engine must define its own coordinated reset handling.

## Reproduction

```sh
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3 integrations/ibus/run_gtk_tests.py
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3 integrations/ibus/run_gtk_tests.py --async-client
```

The second command currently exits 1 for the two documented safety assertions.
A fresh bus/display/fixture is created per command. Commands require installed
GTK4 Python bindings, gtk4-broadwayd, IBus and dbus-run-session. No live user
participation is required. Earlier 25 protocol/routing checks are separate and
were not rerun: their implementation and runners were not changed.

## Source trace and remaining privacy boundary

[GTK 4.22.4 Gtk.Text](https://raw.githubusercontent.com/GNOME/gtk/4.22.4/gtk/gtktext.c)
drives focus/reset through its input context.
[IBus 1.5.34-rc2 GTK client](https://raw.githubusercontent.com/ibus/ibus/1.5.34-rc2/client/gtk2/ibusimcontext.c)
sets content type before focus-in, clears preedit on focus-out, and clears preedit
on reset only when its sync mode is nonzero. The GTK4 build defaults to sync mode
1 but permits an environment override. Focus-in calls the surrounding-text helper;
despite the broad comment at that call site, the reviewed helper checks capability
and needs_surrounding_text before retrieval. The fixture disables active surrounding
text requests. This is not proof that the IBus base class cannot cache unsolicited
text; protected/unknown-field privacy still requires a separate audit.

## Decision and next bounded task

No-go for deploying this fixture or accepting the forced async route as safe.
The synchronous GTK route is promising only within the tested limits. Forcing a
GTK module or environment variable is not an approved user-facing requirement,
nor proof of Chrome, GNOME Wayland, layout switching, safe undo, six-language
correction, full keyboard dispatch, or safe replacement of committed text.

Next: trace where the delayed async commit loses target ownership and determine
whether a supported client/engine mechanism can prevent it without global input
suppression or edits into a new field. Keep failing tests as the acceptance target.
Then compare that mechanism with the normal GNOME Wayland path. Do not patch
system GTK/IBus or the installed companion without an explicit implementation
proposal and approval.

## Delayed-commit ownership trace (2026-09-25)

Added a diagnostic Routing endpoint to the isolated stimulus fixture. It records
three booleans: whether a sample exists, whether the engine's cached focus still
matches its key-handler entry focus, and whether the daemon's CurrentInputContext
matches that origin immediately before emission. References remain in memory.
The query does not guard or alter the commit and is not a proposed safety check.

| Mode | Engine still original | Daemon still original | Actual widget text A / B |
| --- | --- | --- | --- |
| Sync | true | true | `x` / empty |
| Forced async | true | false | empty / `x` |

Both full 11-check suites were rerun after instrumentation: sync passes all;
async retains the same two safety failures. The extra query can affect timing,
but the wrong-target result was also reproduced before instrumentation. This
sample demonstrates disagreement, not an atomic routing trace or proof about
every schedule. No desktop capture or production changes occurred.

### Source-supported mechanism

[Engine CommitText](https://raw.githubusercontent.com/ibus/ibus/1.5.34-rc2/src/ibusengine.c)
serializes only text; its reviewed signature contains no originating context or
key-request identifier. FocusInId does not add that identity to later commits.

[Daemon inputcontext.c](https://raw.githubusercontent.com/ibus/ibus/1.5.34-rc2/bus/inputcontext.c)
connects engine-signal handlers with a context as callback data. Unsetting an
engine disconnects those handlers; assigning it installs handlers for its new
context. The commit callback delivers through that attached context. Key replies
retain their request context separately; text signals do not share that binding.
[ibusimpl.c main](https://raw.githubusercontent.com/ibus/ibus/main/bus/ibusimpl.c)
shows global-engine reassignment during focus changes. The tagged ibusimpl file
was unavailable, so this part of the source trace is corroboration rather than
an exact installed-build audit. Together with the measured drift and wrong widget,
these findings locate the missing original-target association at daemon/engine
commit routing, rather than establish a GTK widget-selection bug.

[GTK IBus client](https://raw.githubusercontent.com/ibus/ibus/1.5.34-rc2/client/gtk2/ibusimcontext.c)
forwards received commits to the corresponding GtkIMContext. Mode 1 synchronously
processes a key and requests post-processing; context creation enables that option.
Modes 0 and 2 are different paths, not interchangeable guarantees. Mode 2 was not
tested. Post-processing support does not itself establish arbitrary async target
transactions, and its D-Bus properties are marked unstable in the reviewed daemon.

### Prevention assessment

- Engine-local focus/generation checks: insufficient when notification is queued
  behind current processing, as demonstrated by the unchanged engine focus.
- Query daemon focus before commit: can detect this measured mismatch, but a
  separate query plus emission cannot exclude another transition in between.
  Dropping a consumed key would also violate input preservation.
- Return unhandled or ForwardKeyEvent after detecting a mismatch: original-target
  delivery is not established; this cannot be advertised as lossless recovery.
- GTK synchronous processing plus post-processing: existing client mechanism,
  passing the tested schedule; no engine API identified that enforces this for
  all clients. Do not require global environment changes or switch client modes
  silently. A client can also process focus outside the schedule tested here.
- Client/daemon change retaining origin ownership through completion: a plausible
  design requirement, not an implemented fix. It must distinguish original-key
  delivery from a stale correction and handle destroyed/protected targets.

[GNOME Shell 50.1](https://raw.githubusercontent.com/GNOME/gnome-shell/50.1/js/misc/inputMethod.js)
uses asynchronous processing and checks IBus context identity in the completion
callback. Its shared context does not identify individual application fields.
GTK's sync-mode setting therefore is not a demonstrated fix for that route.
This does not prove the Broadway failure occurs in Shell; compositor/client
routing remains a separate evidence boundary.

Next: inspect the GNOME Shell/Mutter commit target and client lifecycle to identify
where a target-bound completion could be implemented, or document that the reviewed
interfaces cannot support it. Produce a concrete design proposal before patching
any desktop component; retain the current failing async tests as acceptance cases.
