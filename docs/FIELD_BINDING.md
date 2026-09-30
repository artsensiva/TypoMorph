# GNOME field / IBus binding investigation

Status: read-only feasibility investigation, 2026-09-25. No production field authority is implemented.

## Identity boundary

AT-SPI exposes an accessible object reference for a field and a containing application reference. IBus exposes its current input-context path. The reviewed GNOME Shell 50.1 input method creates one `gnome-shell` IBus context and forwards its focus lifecycle. A path for that context is therefore not inherently a field identifier. Matching an application process, window or observation time would be supporting metadata, not proof of the exact edit target or a common event generation.

Mutter 50.1's Wayland implementation maintains compositor-side input-focus and surface/resource state. The inspected interface does not supply an AT-SPI field identifier for the external prototype to join to an IBus context. This is a finding about the reviewed path, not a claim that no possible integration exists.

Sources: [GNOME Shell 50.1 input method](https://raw.githubusercontent.com/GNOME/gnome-shell/50.1/js/misc/inputMethod.js), [Mutter 50.1 text input](https://raw.githubusercontent.com/GNOME/mutter/50.1/src/wayland/meta-wayland-text-input.c), [AT-SPI Accessible](https://raw.githubusercontent.com/GNOME/at-spi2-core/main/xml/Accessible.xml).

## Read-only diagnostic

Run `/usr/bin/python3 integrations/ibus/probe_field_binding.py --read-only` only during a coordinated session. It waits 45 seconds, obtains a baseline, then waits 120 seconds before a second snapshot. Suggested sequence: empty Text Editor document, then its empty Ctrl+F search field. Keep each field focused for the corresponding snapshot.

Only CurrentInputContext, accessibility bus discovery and Accessible.GetChildren/GetRole/GetState are queried. No Text, GetAttributes, Name, Description, clipboard, key capture, focus mutation or engine registration is used. IDs remain in memory and only comparison booleans are printed. Traversal is bounded by 256 objects, depth 32 and time limits; repeated metadata snapshots must agree. This probe establishes neither selection eligibility nor an atomic cross-protocol snapshot. Stable endpoints cannot rule out intermediate focus changes.

A changed field with unchanged IBus path disproves using that path alone as a unique field identity in the measured session. Different paths would still not prove authoritative binding. An unavailable or unchanged-field result is inconclusive about the intended transition.

## Required next evidence

A real observer needs a trustworthy field identity and an ordered focus generation, a demonstrable connection to the input method's target, content-free safety metadata, revocation on uncertainty/disconnection and lossless draft disposition. Separate polling streams must not be presented as a transactional binding. The installed layout-only companion currently exports none of this; it is unchanged by this investigation. Before extending it, establish whether observable GNOME events actually identify field changes inside one surface rather than only window changes.

## Real-session result

The owner-coordinated probe completed with `accessible_field_changed=True` and `ibus_context_unchanged=True`. Baseline and final field identities each passed repeated metadata reads, with stable IBus IDs bracketing those reads. The instructed transition was Text Editor document to its Ctrl+F field; application names and contents were deliberately not collected, so that attribution relies on the coordinated user actions.

This demonstrates a many-fields-to-one-context observation and rejects IBus path equality as a field binding. It does not establish a synchronized event stream, exact intermediate focus history, application-wide coverage or correction safety. No installed module/settings changed. Next investigate content-free field/focus event ordering in GNOME and AT-SPI before connecting any observer to the prototype's Confirm endpoint.

## Focus notification source review (2026-09-25)

GTK 4.22.4 emits `org.a11y.atspi.Event.Object.StateChanged` with detail
`focused` when the accessible platform focus state changes. Its implementation
uses `(siiva{sv})`: state name, enabled flag, zero, a constant string `"0"`, and
an empty property dictionary. This specific publisher path carries no field
contents. This is not a payload guarantee for arbitrary accessibility providers
or other event types. PropertyChange and TextChanged must not be subscribed to
for this diagnostic.

The reviewed IBus `main` bus interface declares RegistryChanged,
GlobalEngineChanged and GlobalShortcutKeyResponded, but no public field-focus
signal. Internal `focus-in`/`focus-out` GLib callbacks are not public D-Bus
subscriptions. This source review is not an installed-version compatibility
verification: the version-tagged source could not be retrieved. CurrentInputContext
can be sampled, but cannot supply a missing per-field generation; the preceding
live experiment already showed distinct fields sharing its value.

Consequently a GTK focus notification is a candidate invalidation hint, not an
authority to grant the prototype's Confirm permission. Receiving it before or
after a separate IBus query establishes only observer receipt order. It does not
establish ordering against key delivery, rule out a delayed focus notification,
or prove that an unfinished draft belongs to the newly observed field. No live
event subscription, input capture, installation or engine activation was performed
in this source-review stage.

A bounded diagnostic can next test whether the intended document/search transition
actually produces these notifications. Before a live run, validate narrow bus-side
filtering on a private bus: exact provider unique name, Object.StateChanged and
arg0=focused, with no requested property payloads. Resolve the registry registration
contract and verify the target GTK provider; do not broaden the subscription if
no events arrive. Keep IDs in RAM, output counts/booleans only, and label any IBus
samples as observations rather than synchronized focus events. Such a probe must
remain disconnected from Confirm. A production solution still requires a shared
target generation ordered with input processing and lossless draft resolution.

Sources: [GTK 4.22.4 focus event publisher](https://raw.githubusercontent.com/GNOME/gtk/4.22.4/gtk/a11y/gtkatspicontext.c),
[IBus bus implementation, main branch](https://raw.githubusercontent.com/ibus/ibus/main/bus/ibusimpl.c).

## Narrow observer implementation and first attempt (2026-09-25)

`integrations/ibus/probe_focus_events.py` implements a dedicated Gio connection
with a bus match restricted to the baseline provider's unique name,
Object.StateChanged and arg0=focused. Registry registration requests
`object:state-changed:focused`, no properties, and that provider only. The provider
must report GTK 4.22.4, whose publisher was reviewed. Payload shape/constants are
checked; unexpected payloads end observation without printing their contents.
This is a compatibility check, not authentication of an untrusted application.
IDs remain in RAM; only counters and booleans are output. A different focused
accessible is not automatically a verified editable field. No IBus input ordering
is inferred. Observation ends after 120 seconds or 128 matching events; normal
cleanup unsubscribes, deregisters and closes the dedicated connection.

Private-bus test passed: one expected focus event received; another sender,
another state and a TextChanged event excluded; unexpected payload rejected.
Run with a fresh `dbus-run-session` and set TYPOMORPH_FOCUS_TEST_ADDRESS to that
session's DBUS_SESSION_BUS_ADDRESS, then invoke `--private-test`. Test data are
synthetic constants only.

The first coordinated `--read-only` attempt ended with `baseline_unavailable`
before subscription or registry registration. No conclusion about event delivery
can be drawn. User timing/focus confirmation is pending. A retry can use
`--read-only --long-preparation` for 90 seconds before baseline (default 45),
followed by the same 120-second observation window.

Registry contract: [AT-SPI Registry XML](https://raw.githubusercontent.com/GNOME/at-spi2-core/main/xml/Registry.xml).

## Completed long-preparation live retry (2026-09-25)

The coordinated `--read-only --long-preparation` run reached `observer_ready`
after its 90-second baseline delay and completed with exit status 0:

```text
focus_gained_count=1
focus_lost_count=1
different_accessible_focused=True
unexpected_payload=False
binding_not_proven; no input ordering or eligibility conclusion
```

This establishes delivery of the narrowly selected GTK focus notifications and
a different accessible focus target in this session. The intended transition was
an empty Text Editor document to its empty Ctrl+F search field; attribution to
that action relies on the coordinated user setup, not collected names or text.
Counts do not record event sequence, and no key events or IBus event ordering
were measured. The new object's editable role, selection and safety were not
certified by this observer. No text was read, no input was modified, and no
installed component or production eligibility gate was changed. The process
finished through its cleanup path; deregistration success is not separately
reported by this probe.

Next bounded step: exercise delayed focus notification and new-field input in
the isolated protocol fixture, including A/B/A and an unfinished draft. Define
and test the required ordering contract before connecting these asynchronous
notifications to eligibility. Live event delivery alone does not establish it.

## Isolated delayed-notification schedules (2026-09-25)

Added three known-gap characterization tests to test_protocol.py. Withholding
field notifications while retaining one IBus context permits old eligibility to
consume subsequent input and emit a synthetic correction. An unreported A/B/A
round trip cannot be distinguished from continuous focus. Revoke clears the
engine's unfinished draft without sending a commit or preedit disposition update.
The tests do not model real application routing or prove actual text loss.

Validation: 16 direct protocol checks and 9 private-daemon checks passed. Three
passing checks explicitly reproduce unsafe schedules; these are evidence of a
blocker, not safety regression guarantees. No engine behavior, desktop input,
installed component or production gate was changed.

Required contract: field changes must invalidate permission before input for the
new target is processed; grants and input need the same authoritative generation;
old preedit must be resolved exactly once on its original target before ownership
changes. An asynchronous AT-SPI observer plus CurrentInputContext polling does not
establish this contract. Next: compare concrete integration options against these
requirements and obtain owner approval before an architectural change.
