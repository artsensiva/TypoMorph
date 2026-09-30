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
