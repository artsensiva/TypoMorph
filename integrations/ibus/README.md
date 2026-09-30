# Isolated IBus preedit experiment

Run `/usr/bin/python3 integrations/ibus/run_tests.py` from the repository root.

This is an experimental protocol fixture, not an installable input method. The direct runner above uses the installed IBus.Engine implementation on a new dbus-run-session bus. It does not run/register with an IBus daemon, open input devices, connect to a display, or install a component or input source. The runner removes display/session discovery variables, sets an unusable IBUS_ADDRESS and temporary XDG directories, and terminates the engine after tests. These are controlled test-process isolation measures, not a security sandbox against hostile code. No network or content persistence is implemented.

The synthetic client supplies every input value. Only the explicit fixture `ghbdtn` maps to `привет`; the production Rust classifier and layout switching are not integrated. The current lowercase word is displayed as preedit (maximum 32 characters), then committed with its delimiter. Long input flushes unchanged chunks. Keyboard shortcuts flush the draft unchanged and pass through. Unsupported input is passed through.

## Evidence

Sixteen tests exercise actual D-Bus Engine methods and signals:

- fixture correction and following-word ordering, with no DeleteSurroundingText;
- focus A/B/A without replaying the old draft;
- password, PIN, terminal, private/hidden and unknown purpose refusal;
- reset/disable invalidation;
- Backspace and shortcut ordering;
- missing/lost client capabilities;
- 80-character input without truncation;
- unchanged ContentType after focus leaves analysis disabled;
- late acknowledgement after A/B/A focus changes;
- different fields under one IBus context and one-shot confirmation;
- input arriving before confirmation;
- reset/disable/revocation invalidating pending confirmation;
- read-only, selected, composing or protected observations refusing acknowledgement.

Tests assert the requested COMMIT preedit mode. They do **not** assert that a real client commits a draft safely on focus loss. Key events are supplied sequentially without human delays; concurrent compositor/client routing is not modeled.

## Findings and blockers

ContentType is an Engine D-Bus property, not a SetContentType method. In installed IBus 1.5.34-rc2, setting an unchanged value does not invoke the content-type callback. The old artificial password-to-normal handshake has been removed. Positive cases now use explicit synthetic field acknowledgements; cached FREE_FORM alone never grants permission. A separate test demonstrates refusal without acknowledgement. See the [IBus implementation](https://raw.githubusercontent.com/ibus/ibus/1.5.34-rc2/src/ibusengine.c).

On focus/reset/disable the engine clears its private draft and makes no delayed commit. This prevents replay into a new context but does not prove preservation of the original word: real client COMMIT-mode handling must be tested. Losing input would be a release blocker. Selection behavior, unsolicited surrounding-text delivery/base-library caching, shared real GNOME contexts, pending asynchronous callbacks, native Wayland/XWayland clients, source changes, manual correction and undo are unverified.

The prototype leaves unsupported/protected synthetic key events unhandled and does not put them into its own draft. The base library can receive/cache surrounding text if another process sends it; disabling active requests alone is not a demonstrated privacy boundary. No such content is supplied by this runner. Do not install or use this fixture on personal input.

The private-daemon suite below covers synthetic routing. Keep the production automatic-replacement safety gate closed.

## Private daemon routing suite

Run `/usr/bin/python3 integrations/ibus/run_daemon_tests.py` for the next integration layer. It starts a real IBus daemon on a unique temporary socket inside a new session bus, registers the fixture only there and supplies two synthetic input contexts. It does not use `--replace`, XIM, a desktop display or the owner's IBus socket. The daemon activates its IBus portal on that private session bus; the bus and test processes are torn down on exit. Display variables are removed and XDG paths are temporary. The synthetic clients observe serialized text signals through Gio to avoid GI floating-reference warnings from InputContext signal wrappers.

Nine daemon-routing checks pass:

1. Fixture correction and following input are routed to the client.
2. Server-side COMMIT policy preserves a draft in the original context across A/B/A focus transitions.
3. A **modeled** client-side COMMIT policy preserves the draft once without a duplicate server commit.
4. Password input produces no preedit/correction.
5. An ordinary draft cannot leak into the next password context.
6. Normal-to-normal focus with unchanged content type leaves correction suspended and ordinary input passing through.
7. Server-side reset preserves the original draft once and suspends analysis pending fresh eligibility.
8. A fresh acknowledgement resumes correction in the new context even with unchanged normal content type.
9. An old acknowledgement cannot reactivate a context after focus leaves and returns.

The daemon uses global-engine mode; tests select the engine on the private bus accordingly. The same engine is routed between synthetic contexts. Positive correction cases use the synthetic observer acknowledgement. The negative normal-to-normal case deliberately omits it.

These results establish behavior of the installed daemon with synthetic contexts, not the actual GTK/Chrome or shared GNOME Shell client. In particular the client-side commit test implements the policy in test code; it is not evidence of application behavior. Actual field identity, selection, fresh eligibility for identical content types, crash recovery, unsolicited surrounding-text delivery and desktop timing remain unresolved. No production safety gate was opened.

## Field-generation acknowledgement

`eligibility.py` holds only context/field identifiers, a generation counter and permission state. Each Begin creates a pending observation; Confirm must match context, field and generation exactly and can be accepted only once. Every focus transition (including return to the same field), reset, disable, capability loss or explicit revocation invalidates pending permission. Input passed through while waiting also invalidates it, so a late response cannot enable processing of a partially observed word.

The confirmed observation must be editable, unselected, not composing and ordinary non-sensitive text. Known protected IBus content metadata vetoes the observation. ContentType callbacks never grant permission themselves. Field transitions within a shared IBus context must be reported explicitly; the gate does not invent field-change events.

`test_control.py` is a synthetic observer with Begin/Confirm/Revoke D-Bus methods, exported only by the isolated runners. It is deliberately **not** an authenticated desktop service or a real AT-SPI/GNOME observer. Random initialization reduces accidental reuse across fixture instances; it is not an authentication credential. A malicious or stale real observer is outside the proof. No observation payload contains typed text.

Begin/revoke currently clears the engine draft without emitting into a potentially changed target. Tests request fresh acknowledgements only after client/daemon draft resolution. Preservation during arbitrary observer revocation is not established and blocks live use. Production needs trusted observer lifecycle/disconnection handling, field-to-IBus binding, correctly ordered focus/cursor/selection invalidation, and a safe draft-disposition contract. Polling alone is insufficient for focus-out-and-back.

Next: implement and verify a content-free real observer/field binding in read-only mode before enabling any live correction. The 25 passing synthetic checks include three known-gap characterization tests; a passing result does not establish production safety.


### Metadata-only focus notification probe

`probe_focus_events.py --read-only` waits 45 seconds for a coordinated empty
Text Editor document, verifies the reviewed GTK provider, then observes only its
focused-state events for up to 120 seconds. `--long-preparation` increases the
initial delay to 90 seconds. No text interfaces, key capture or eligibility grants.
See `docs/FIELD_BINDING.md` for isolated filtering evidence and live-test limits.

### Delayed-notification characterization

Three direct protocol tests intentionally reproduce known gaps: stale permission
can emit a commit before notification; an unreported A/B/A transition is invisible;
revocation clears the engine draft without requesting client preedit disposition.
These are deterministic withheld-message schedules, not measured desktop races.
The direct client does not prove where a real application would route the commit
or whether it loses the draft. No production behavior was changed to make these
checks pass. A shared field generation ordered with each input operation and
old-target draft resolution remain prerequisites.


### Real GTK lifecycle harness

Run `run_gtk_tests.py` with `/usr/bin/python3` for a private Broadway display,
private IBus daemon and real Gtk.Text widgets. Eleven synchronous-mode checks pass.
`--async-client` deliberately tests the alternate client mode and currently fails
two safety assertions (wrong-target delayed commit and unresolved reset preedit).
This separate stimulus engine neither uses nor validates TypoMorph eligibility.
See `docs/GTK_IBUS_EXPERIMENT.md` for scope, sources and reproduction limits.


The shared lifecycle fixture also supports the explicitly guarded private native
Wayland runner under integrations/gnome/tests. KeyStarted is metadata-only and
marks the synthetic key handler before its test delay. See
`docs/NATIVE_WAYLAND_EXPERIMENT.md`; native failing cases remain intentional
nonzero safety-test results, not certified application compatibility.
