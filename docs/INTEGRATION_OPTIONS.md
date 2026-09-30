# Linux integration options

Status: recommended experiment approved and completed; see GTK_IBUS_EXPERIMENT.md.
Date: 2026-09-25. No architecture or release-scope change approved here.

## Decision boundary

The current uinput path lacks target-bound editing. The isolated IBus fixture
also demonstrates stale permission when a field change is not reported before
input. A working adapter needs field ownership ordered with input, fresh safety
metadata before analysis, and exactly-once disposition of old preedit on its
original target. A shared generation is one possible design, not proof supplied
by an asynchronous observer. A platform lifecycle with equivalent guarantees
could also satisfy the requirement if demonstrated.

## Comparison

| Option | Identity and ordering | Draft preservation | Coverage and cost | Assessment |
| --- | --- | --- | --- | --- |
| More AT-SPI polling plus uinput | Separate checks cannot bind global key injection to an edit target | Partial edits remain possible | Broad apparent reach, unchanged fundamental gap | Reject as the safety solution |
| Standalone IBus engine plus asynchronous AT-SPI permission | Current fixture cannot detect an unreported field transition | Server handling passes synthetic tests; real clients unresolved | Reuses current prototype, but no proven eligibility authority | Do not connect current observer to Confirm |
| Native toolkit/input-method lifecycle, first GTK/IBus | GTK exposes widget-associated contexts and focus/reset lifecycle; actual client route still needs verification | Must test real widget preedit, not just modeled clients | Promising bounded experiment; GTK results cannot certify Chrome or GNOME-mediated Wayland routing | Recommended next feasibility experiment |
| GNOME Shell/Mutter input-path integration | Closer to dispatch, but reviewed Shell context is shared and no verified accessible-field join exists | Compositor/client lifecycle still needs proof | Version coupling, desktop stability risk; installed layout-only extension is insufficient | Reserve pending native-client evidence; no patch approved |
| Application-local adapters, including browser extension | In-process target/revision checks may serialize edits within a supported editor | Editor-specific undo/composition behavior needs testing | Narrow coverage and substantial per-editor work; cannot replace desktop-wide scope | Conditional alternative, requires owner scope/architecture decision |

These are engineering assessments, not measured implementation estimates.
A manual command using the same unsafe global injection is not automatically safe.
An application-local adapter must still avoid collecting protected-field contents,
handle reentrancy and editor-managed state, and fail before partial mutation.

## Source findings and limits

[GTK IMContext](https://docs.gtk.org/gtk4/class.IMContext.html) provides focus,
reset, key filtering and commit/preedit interfaces.
[set_client_widget](https://docs.gtk.org/gtk4/method.IMContext.set_client_widget.html)
associates a client widget. This does not prove a remote IBus engine receives a
unique widget identity or that every application drives the lifecycle correctly.
Current online GTK documentation includes APIs newer than installed 4.22.4;
notably get_client_widget is marked unstable since 4.24 and must not be assumed
available. Version-specific implementation review is required.

[GNOME Shell 50.1](https://raw.githubusercontent.com/GNOME/gnome-shell/50.1/js/misc/inputMethod.js)
creates a shared IBus context, forwards key processing asynchronously, and uses
client-side preedit commit. Its callback checks context identity, which alone
is not a per-field generation. This is not evidence that Shell generally corrupts
text; it explains why our external observer has not established the needed join.

[Wayland text-input-v3](https://raw.githubusercontent.com/wayland-mirror/wayland-protocols/main/unstable/text-input/text-input-unstable-v3.xml)
groups edits through done. Its serial-mismatch rule still applies edits; it is
not a compare-and-reject transaction. This source is a moving protocol branch,
not proof of installed protocol versions.

[Input Events](https://w3c.github.io/input-events/) defines beforeinput/input
semantics with composition and cancelability distinctions. It supports exploring
editor-local integration, not a claim of universal browser-extension coverage.

## Approved bounded experiment (original plan)

1. Trace version-matched GTK/IBus client code to identify the actual lifecycle
   route and whether safety metadata arrives before text reaches our engine.
2. Prepare a standalone test application with two ordinary widgets and one
   protected widget, a private IBus/session bus, and synthetic input only.
   Exercise actual widget focus/reset/preedit behavior; do not manually feed
   idealized acknowledgements and report them as native eligibility evidence.
3. Test A/B/A, typing during transition, outstanding replies, reset, selection,
   protected focus, engine disconnect and unfinished drafts. Assert exact text
   placement, no duplication/loss, and no protected-field analysis.
4. Record the precise display/input route. A forced GTK IBus module or X11 test
   does not certify the normal GNOME Wayland route, Text Editor, or Chrome.
   Required follow-up coverage must remain explicit.

Local prerequisites: GTK 4.22.4 Python bindings available; neither Xvfb nor Weston
found on PATH. An isolated display route is not yet available from these checks.
Do not silently fall back to the user's desktop or install system packages.
Resolve the test environment first and report any additional setup required.

Proposed changes: new isolated harness/runner under integrations/ibus, version-
matched source notes, relevant tests and recovery documentation. No desktop engine
registration, installed extension changes, safety-gate removal, or release-scope
change. Main risks: proving only a forced route, accidentally idealizing lifecycle,
and unsolicited surrounding-text exposure. Stop before live integration if native
field safety or lossless draft handling cannot be established. The next milestone
is an evidence-based go/no-go result for this route, not more observer polling.

Outcome: isolated Broadway was available. Synchronous checks passed; forced async
mode exposed wrong-target commit and unresolved reset preedit. These findings do
not approve a production architecture; see GTK_IBUS_EXPERIMENT.md.
