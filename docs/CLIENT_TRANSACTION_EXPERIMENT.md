# Client-owned transaction experiment

Date: 2026-09-27. Owner-approved bounded prototype completed.
Production integration remains blocked; BUG-001 is not fixed.

## Implementation and route

`integrations/gnome/tests/client_transaction.py` owns a test Gtk.Text field and
its callbacks. Normal synthetic keyboard input inserts `ghbdtn` first; a separate
completion can then replace it with the fixed `привет` candidate. No classifier,
key suppression, IM preedit ownership or delayed echo engine is used.

`run_wayland_probe.py --client-transaction` uses the existing separate headless
Shell, private buses, temporary XDG state and PID-verified private RemoteDesktop
input. It does not start the echo fixture. The client asserts native `wayland`
IM context and absence of the fixture D-Bus owner, then verifies actual ordinary
key delivery. The exact ordinary platform key-processing internals are not traced
by this check. The owner's desktop, installed application and extension are unchanged.
Isolation limits remain those in NATIVE_WAYLAND_EXPERIMENT.md.

Each owner checks explicit ordinary-field metadata before any snapshot. One
outstanding snapshot is limited to 32 characters and a one-second default lifetime.
Requests use opaque local identities, an invalidation epoch, original contents,
caret and a monotonic deadline. Focus/edit/selection/safety/composition changes
invalidate requests. The timeout clears pending state; completion also checks the
deadline independently of timer dispatch. This is reference removal in transient
Python memory, not a guarantee of secure memory zeroization.

An accepted edit prepares a new Gtk.EntryBuffer before setting it on the widget.
The visible buffer is not modified by separate delete/insert operations. Nested
requests/completions are rejected while applying. This is a test-owned buffer
contract, not an atomic general GTK transaction API. Arbitrary external mutation
callbacks, shared buffers, undo history and editor-managed state are unsupported.
A post-edit invariant detects conflicting content/buffer mutation but cannot
roll back arbitrary callback effects. Therefore this is not safe to install into
an uncontrolled editor. Attach before composition; dispose explicitly before
removing the widget. These lifecycle requirements are controlled by the fixture.

## Validation

Final fresh private-compositor run: **20 passed, 0 failed**, exit 0.

- Delayed stable correction, exact-once completion and foreign-ticket rejection.
- Focus change, A/B/A, two windows, destroyed/recreated field.
- Additional ordered typing, Backspace, selection change and return.
- Protected/unknown exclusion and ordinary-to-password metadata transition.
  Snapshot/request counters verify refusal before analysis, without analyzer reads
  from the excluded field. Test assertions inspect only known synthetic content.
- Reset, composition-start signal, cancellation, disconnect/reconnect.
- Nested correction callback, bounded snapshots, dispatched timeout and deadline
  expiry before timer dispatch.

Composition uses an explicitly emitted lifecycle signal, not a real IME session.
Disconnect/reconnect are owner-state calls, not a killed analyzer subprocess.
Delayed completion is scheduled/held by test control; there is no worker or IPC.
Eligibility comes from the test-owned editor, not AT-SPI inference. Tests validate
one short whole-word range, not general document replacement. They do not prove
privacy of all platform internals or arbitrary application callbacks.

Because the runner changed, the unchanged native `delayed_same_surface` case was
rerun separately: **0 passed, 1 failed**, exit 1, original field empty and new field
received `x`. This remains an ordinary failing assertion, not an expected-failure
waiver. The new architecture experiment does not repair the old transport.
Syntax/whitespace checks passed for the three changed Python files; git diff
--check passed. No unrelated production tests were rerun for these test-only changes.

Reproduce:

```sh
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3 integrations/gnome/tests/run_wayland_probe.py --client-transaction
PYTHONDONTWRITEBYTECODE=1 /usr/bin/python3 integrations/gnome/tests/run_wayland_probe.py --case delayed_same_surface
```

Both runs finished and cleaned up their private services. The second command
intentionally remains nonzero while the native transport defect is unresolved.

## Deployment prerequisites and next decision

Unmodified Text Editor/Chrome do not acquire this owner merely because the test
passes. A deployable integration needs a supported client/toolkit hook with:

- Field lifetime and safety information before text collection.
- Ordered edit, selection, focus, composition and destruction notifications.
- Revision-checked edits within the actual editor's transaction and undo model.
- Original input independent of analyzer availability, plus text/layout coordination.

The prototype establishes none of those hooks in existing applications. No
production automatic correction, system patch, upstream submission or release
scope reduction is approved. Safe undo, layout coordination, multilingual
classification and Windows/macOS implementation remain outstanding.

Follow-up proposal completed: see APPLICATION_INTEGRATION_PROPOSAL.md for
unmodified applications, comparing required native-stack changes with client
adapters. Identify exact missing hooks and deployment dependencies before asking
the owner to approve production architecture. Avoid extending the standalone
fixture as a substitute for resolving that deployment boundary.
