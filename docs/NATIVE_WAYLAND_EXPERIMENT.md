# Isolated native GNOME Wayland experiment

Updated: 2026-09-26. Approved experiment complete; production integration remains
blocked. This is synthetic test data in a separate compositor, not a test against
the owner's Text Editor or Chrome.

## Established route and isolation

Installed GNOME Shell/Mutter 50.1 runs headless on a virtual monitor with a private
Wayland socket. The runner uses temporary XDG config/data/cache/state/runtime
locations, in-memory GSettings and an explicit environment. A custom D-Bus
configuration has no service-activation directories; both session and system-bus
roles point to that private bus. No host session/system bus or host display is
used. No installed extension, input source, binary or system package is changed.
Headless Mutter uses the render node without mode setting; despite requesting
software rendering, the logs reported an i915 GBM renderer. This is not GPU or
filesystem sandbox isolation against malicious code.

The first startup with an inaccessible system-bus address created a socket but
failed Shell initialization. The runner was corrected to expose the private bus
for both roles and require a completed Shell startup marker as well as a real
client route check. Missing optional login/calendar/policy services still produce
warnings: this is not a full desktop session equivalence claim.

Input is sent through the private compositor's RemoteDesktop session. Before
creating the input session the client verifies that the D-Bus service PID matches
the child Shell PID. No clipboard, screencast or host uinput is used. The private
API is version-specific, not a proposed product input backend. Source:
[Mutter 50.1 RemoteDesktop interface](https://raw.githubusercontent.com/GNOME/mutter/50.1/data/dbus-interfaces/org.gnome.Mutter.RemoteDesktop.xml).

The real Gtk.Text client asserts that its IM context ID is `wayland`; it does not
force the GTK IBus module. An initial fixed echo must reach the field and increment
the private engine's echo counter. Thus the verified route is GTK Wayland ->
Shell/Mutter -> private IBus fixture, not the earlier Broadway client route.

## Test stimulus and scope

The separate lifecycle fixture injects fixed COMMIT-mode preedit `ghb`, or echoes
`x` after a 200 ms delay. It is not the TypoMorph classifier or eligibility engine.
A content-free KeyStarted signal is flushed before that delay; focus changes in
delayed tests wait for it. This replaces an assumption that a sent key had already
reached the engine. Only test-owned synthetic widget contents are asserted or
printed. No actual personal input is captured or saved.

## Results recovered and verified

The first combined run ended with 3 passes and 6 failures. It reused a compositor
and widgets, so later fixture/precondition failures could cascade. Each case was
then rerun with a fresh compositor, buses, fixture and client. Final case outcomes
are 4 passes and 5 failures (including a corrected protected-case assertion):

| Case | Result | Observation |
| --- | --- | --- |
| Shared context | Pass | Distinct fields share Shell's IBus context |
| Focus with unfinished draft | Fail | A empty; B received `ghb` |
| Delayed key, same surface | Fail | A empty; B received `x` |
| Rapid A/B/A | Pass | `x` remained on A in this tested schedule |
| Selection | Pass | Fixed echo changed `abc` to `axc` for the selected range |
| Protected transition | Fail | A empty; password field received old `ghb` plus ordinary fallback `x` |
| Reset | Fail | No committed draft; `ghb` remained preedit |
| Delayed key, two surfaces | Fail | Original field empty; new window received `x` |
| Fixture disconnect | Pass | Original widget retained the seeded draft in this schedule |

The initial protected-case assertion incorrectly required PASSWORD purpose to
reach an active engine. A focused-engine absence is also a valid refusal path:
State returned no active fixture and purpose 999 (the fixture's sentinel, not an
OS purpose). The corrected case permits that path, verifies Seed refusal and no
new fixture echo, and tests draft placement and normal fallback typing. It still
fails because the old draft reaches the protected field. This is not evidence
that the fixture analyzed protected input. Reset failure is unresolved disposition,
not demonstrated text loss.

A/B/A passing does not establish a generation protocol; endpoint content alone
cannot prove all intermediate state was safely handled. Four passing cases are
not a release compatibility certificate. Safety failures remain nonzero test
results, not marked expected failures or silently accepted.

## Artifacts and reproduction

- integrations/gnome/tests/run_wayland_probe.py: private services and bounded runner.
- integrations/gnome/tests/test_wayland_route.py: route verification and native cases.
- integrations/ibus/gtk_fixture.py: shared test-only stimulus and KeyStarted signal.

Run a single fresh scenario from the repository root:

```sh
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3 integrations/gnome/tests/run_wayland_probe.py --case delayed_same_surface
```

Allowed cases are the identifiers in CASES in the runner. With no arguments the
runner executes all cases together; independent `--case` runs are preferred for
attribution. Each failing case intentionally exits 1. The test process is bounded
at 40 seconds and the outer runner at 65 seconds; cleanup terminates private
children and the runner's process group. No compositor/test remains intentionally
running after a command.

## Conclusion and next step

No-go for connecting the current engine to normal application typing. The native
route reproduced wrong-target delivery with the fixed fixture, so selecting an
IBus engine or adding asynchronous focus observations does not establish safety.
A fix must preserve ownership through both delayed key completion and preedit
handoff; refusing a correction must not lose original input.

The next-step design is now prepared in
[ORIGINAL_TARGET_FIX_PROPOSAL.md](ORIGINAL_TARGET_FIX_PROPOSAL.md); implementation
was subsequently approved and completed: see CLIENT_TRANSACTION_EXPERIMENT.md.
It maps these schedules to a separate client-owned transaction experiment and
does not repair the failing native transport. The delayed same-surface failure
was reproduced again after adding the separate runner mode. Keep separate the engine stimulus, native
client/compositor behavior, and final deployment requirements. No upstream report,
system patch, forced environment setting or release-scope reduction is authorized
by this result. Protected/unknown input collection still needs its own audit.

## Stale text-input-v3 edit characterization (2026-10-29)

Question: when an input method's deletion and commit were computed from an
earlier view of a field, does a stock GTK4 client apply or reject them, and does
the `done` serial protect it? **Result: the client applies them, the serial does
not protect, and wrong-content and wrong-target mutation were reproduced.** This
is a finite synthetic fixture. It is not a safety result, says nothing about
Chrome, Firefox or other toolkits, and is not a decision on any workaround.

### Path and schedule

- Client: the system GTK 4.22.4 (`/usr/lib/x86_64-linux-gnu`, unpatched; the test
  prints the mapped library path) `Gtk.Text` with its `GtkIMContextWayland`,
  speaking `zwp_text_input_v3` version 1 to GNOME Shell 50.1 (Mutter) running
  headless, then the private IBus 1.5.34-rc2.
- Engine: the test-only `integrations/ibus/stale_edit_fixture.py`. It records the
  surrounding text the client reports, keeps one snapshot, and on request sends
  `delete_surrounding_text(-6, 6)` plus `commit_text("привет")` computed from that
  snapshot, whatever the field holds then. It handles no key events.
- Harness: `integrations/gnome/tests/test_stale_text_input.py`, run with
  `run_wayland_probe.py --stale-text-input [--case NAME]`. Field `a` = `xx ghbdtn`
  with the cursor at the end, field `b` = `keep`. Evidence is the client's own
  `WAYLAND_DEBUG` trace, kept inside the private session directory. Waits are
  condition-driven with deadlines (trace events, engine acknowledgement, socket
  byte counts), not sleeps used to create a race.
- Matched-serial schedules: change the field, publish the change the way GTK does
  (`gtk_im_context_reset()`), wait until the engine reports the new state, then
  send the stale request. In-flight schedules: let the request reach the client
  socket without reading it (`FIONREAD` on the socket identified by peer PID),
  change and publish the field, and only then dispatch events. The trace showed the
  client's next `commit` queued before the `done`, which carried an older serial.

### Observed (all 27 fresh-session runs agreed; 3 full-suite runs agreed)

| Case | Serial at `done` | Delete event reached client | Final `a` / `b` | Outcome |
| --- | --- | --- | --- | --- |
| control_unchanged | match | yes | `xx привет` / `keep` | exact replacement (control) |
| cursor_moved (cursor to 2) | match | **no** | `xxпривет ghbdtn` / `keep` | **corruption**: commit inserted mid-field, original word kept |
| prefix_inserted | match | yes | `QQxx привет` / `keep` | exact replacement |
| word_changed (`ghbdtx`) | match | yes | `xx привет` / `keep` | **corruption**: a different word replaced |
| other_field (focus to `b`) | match | **no** | `xx ghbdtn` / `keepпривет` | **corruption**: commit into the wrong field |
| selection_over_word | match | yes | `xx привет` / `keep` | exact replacement |
| inflight_cursor_moved | **mismatch** (9 vs client 10) | yes | `xxпривет ghbdtn` / `keep` | **corruption** |
| inflight_prefix_inserted | **mismatch** | yes | `QQxx привет` / `keep` | exact replacement |
| inflight_word_changed | **mismatch** | yes | `xx привет` / `keep` | **corruption** |

Four checks pass and five **fail**; the harness exits 1. The failures are retained
safety failures. The passing non-control cases are coincidences of this finite
schedule (the word still ended at the cursor), not safety.

### Why (source-read; the matching runtime outcomes are above)

- GTK 4.22.4 `text_input_done` applies the pending deletion then the commit
  regardless of the serial; the serial only gates a later `notify_im_change`.
  This matches the installed protocol text and the in-flight cases above.
- `gtk_text_delete_surrounding_cb` deletes at `current_pos + offset`, relative to
  the widget's current cursor, with no comparison against what the engine saw.
- When the engine's offset exceeded what Mutter's cached cursor allowed, no
  `delete_surrounding_text` reached the client but the `commit_string` did. That
  is a partial application: commit without deletion. The exact Mutter 50.1
  source explains the observation: `meta_wayland_text_input_focus_delete_surrounding`
  converts the requested character offset against its cached surrounding text,
  then returns at `g_return_if_fail (before >= start)` when the range begins
  before that cache. The commit path independently queues a null preedit and the
  commit string, so the delete rejection does not suppress the commit.
- GNOME Shell 50.1's `inputMethod.js` fallback is not the cause here. Its
  try/catch surrounds the call that queues the Clutter delete event; Mutter's
  range check occurs later while processing that event, so the exception cannot
  be caught there. If Shell's fallback ran for `offset=-6, nchars=6`, it would
  request a zero-length deletion, which does not match the trace showing no
  delete event at all.
- Unconfirmed: `text_input_delete_surrounding_text` applies a compositor-supplied
  byte length to the client's cached text with `cursor_pointer - before_length`,
  with no check against the cursor index, so a large length could read before the
  buffer. An AddressSanitizer run of the in-flight cursor case reported nothing,
  but the read would occur inside GLib, which is not instrumented, so that run
  cannot confirm or exclude it. The optional sanitizer mode was removed.

### Harness findings

GTK publishes a field's surrounding text only on focus-in, `gtk_im_context_reset()`
or an IM commit. A programmatic `set_text()` publishes nothing, so a first draft
had the engine seeing an empty field. The harness now publishes state the way
GTK does after an edit. Compositor `done` acknowledgements of the client's own
commits vary between two and three per matched case; outcomes did not vary.

### What this does not establish

Only `GtkText` in one toolkit version was exercised; `GtkTextView`/`EditorDocument`,
IME preedit, concurrent typing, a real analyzer or engine, other Mutter, Shell or
IBus versions, and any other client (including Chrome and Firefox) are untested.
The engine's stale request is an idealized stale reply. No protocol non-conformance
is claimed; the serial is not specified as a compare-and-swap. No workaround, GTK,
Mutter, Shell, daemon or gate change was made, and none is proposed here.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 integrations/gnome/tests/run_wayland_probe.py --stale-text-input
PYTHONDONTWRITEBYTECODE=1 python3 integrations/gnome/tests/run_wayland_probe.py --stale-text-input --case inflight_word_changed
```

The existing nine native cases are unchanged (four pass, five fail as above) and
the 20 client-transaction checks still pass.
