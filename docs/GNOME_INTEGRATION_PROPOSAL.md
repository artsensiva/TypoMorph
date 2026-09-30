# GNOME Wayland integration proposal

Date: 2026-09-25. Update 2026-09-27: the native reproducer was approved and completed.
See NATIVE_WAYLAND_EXPERIMENT.md. Production architecture remains unapproved.
No installed component, production gate or approved release scope changed.

## Finding

The reviewed interfaces do not establish an end-to-end field-bound completion
for an external TypoMorph engine. An extension that checks focus and then calls
commit would still need to establish ordering through compositor and client.
The subsequent isolated native Wayland fixture also reproduced wrong-target
draft and delayed-character delivery. This is controlled fixture evidence, not a
claim that ordinary GNOME typing generally misroutes text. See
NATIVE_WAYLAND_EXPERIMENT.md for the exact route and limitations.

### Version-specific source evidence

[Mutter 50.1 input method](https://raw.githubusercontent.com/GNOME/mutter/50.1/clutter/clutter/clutter-input-method.c):
clutter_input_method_commit queues a Clutter IM event through
clutter_input_method_put_im_event. The shown event construction passes seat,
text and editing parameters, not an originating application-field reference.
The same focus object is ignored on repeated focus-in. This inspection covers
the construction path, not every event-dispatch implementation.

[Mutter 50.1 Wayland text input](https://raw.githubusercontent.com/GNOME/mutter/50.1/src/wayland/meta-wayland-text-input.c):
one input-focus object is created per text-input seat structure. Commit delivery
iterates its current focus_resource_list; surface/client routing is distinct from
a widget identifier. State commits can enable an already-focused object without
repeating focus-in. Reset/flush logic exists on transitions and must be included
in a real reproducer. Neither surface identity nor this object alone establishes
per-field generation. The inspected path does not carry an external AT-SPI field
reference.

[GTK 4.22.4 Wayland client](https://raw.githubusercontent.com/GNOME/gtk/4.22.4/gtk/gtkimcontextwayland.c):
text_input_commit stores pending text on global->current; commit_apply emits its
commit signal. text_input_done applies edits before the serial equality condition,
which controls subsequent state notification. Focus-out disables the context and
clears current; focus-in assigns it. Therefore a serial equality test must not be
assumed to reject stale edits. Native reset/disable behavior was subsequently measured; see the experiment results.

[GNOME Shell 50.1 input method](https://raw.githubusercontent.com/GNOME/gnome-shell/50.1/js/misc/inputMethod.js)
uses asynchronous IBus key processing and a shared context; its completion check
compares that context, not an application widget generation. Existing source
review and the actual shared-context observation are recorded in FIELD_BINDING.md.

The version-tagged clutter-input-focus.c could not be retrieved through either
source URL attempted. This is a partial source trace, not a complete proof of
all dispatch behavior. No claims about Chrome's implementation follow from GTK.

## Architecture recommendation

Do not implement an extension-only automatic replacement backend or distribute
patched system packages on the current evidence. The native reproducer is now
complete and failed five of nine fresh scenarios. The proposed next step is a
bounded client-owned transaction prototype; see
[ORIGINAL_TARGET_FIX_PROPOSAL.md](ORIGINAL_TARGET_FIX_PROPOSAL.md).
The bounded prototype was approved and completed; see
CLIENT_TRANSACTION_EXPERIMENT.md. No production route is certified yet.

A candidate long-term boundary places final eligibility validation and mutation
in the client/toolkit that owns the editable field, serialized with that field's
input lifecycle. Analysis may be local, but a returned suggestion is not permission
to apply. If the compositor/IBus path carries asynchronous results, it needs an
origin identifier preserved through transport and checked by the owner. These
are proposed requirements, not existing APIs or an approved product architecture.

| Component | Proposed responsibility | Existing evidence / change needed |
| --- | --- | --- |
| Field-owning client/toolkit | Field identity, revision, current safety, selection/composition, final apply | Owns widgets; a supported TypoMorph transaction hook is not established |
| Local analyzer | Bounded candidate computation; return suggestion with request identity | Existing classifier remains separate from this experiment |
| IBus/Shell bridge if used | Preserve request origin; never retarget stale results | Reviewed CommitText lacks origin; extension-only completion is insufficiently proven |
| Mutter/Wayland transport if used | Preserve origin through queued events and client delivery | No reviewed end-to-end field-token operation; may require coordinated protocol changes |
| Layout-only companion | Existing layout operations | Leave unchanged; it is not an eligibility authority |

### Required transaction contract

1. The field owner checks protected/unknown/selected/composing state before
   releasing any text to analysis. A generation changes on focus loss/return,
   destruction, input, selection/composition changes and any invalidating state.
2. A candidate references owner/session identity, field lifetime, focus generation,
   input revision, expected range and selection. Reconnection cannot reuse grants.
3. The owner validates and applies on its serialized edit path. Validation must
   not be separated from mutation by an asynchronous hop or unhandled reentrant
   editor callback. Reject before any deletion or layout side effect.
4. Original input is distinct from an optional correction. Stale suggestions can
   be discarded; consumed user input cannot. Preserve original text locally in
   the owning field or resolve its preedit exactly once before handoff. Do not
   deliver to the new target or resurrect a destroyed field to compensate.
5. Timeouts/disconnects leave original input intact and correction unavailable.
   No persistent input log, remote transmission, clipboard fallback, global grab,
   or global suppression is introduced.

Coordinating layout changes and safe undo remains an additional requirement;
text transaction success alone does not satisfy the full approved product.
An application-local integration also introduces deployment/coverage costs and
cannot silently substitute for the required native browser/desktop coverage.

## Completed experiment: original approved plan

The following records the original plan, not outstanding work. For final setup
and outcomes use NATIVE_WAYLAND_EXPERIMENT.md; its verified prerequisites supersede
the preflight uncertainties below.

Purpose: resolve whether native GNOME/Mutter ordering prevents or reproduces the
observed delayed-commit problem before changing architecture.

Local preflight: installed gnome-shell --help lists --headless, --no-x11,
--virtual-monitor and --wayland-display. This establishes available CLI options,
not successful startup. The grouped pkg-config development-library check did not
succeed; a compiled client cannot yet be assumed buildable.

Proposed repository changes:
- Add a runner under integrations/gnome/tests for a separate headless Shell,
  private session/IBus buses, temporary XDG settings, and a dedicated Wayland socket.
- Adapt the existing test-owned GTK client/fixture for actual GTK Wayland input
  (do not force GTK_IM_MODULE=ibus). Verify the route before counting results.
- Use fixed synthetic data only, with test input confined to the private seat.
  Do not use host uinput or connect to the active desktop display. Establish the
  isolated injection mechanism before running any input test.
- Measure two fields in one surface, two surfaces, A/B/A, outstanding key reply,
  preedit reset/focus loss, selection, protected transitions, disconnect and
  original-key fallback. Record actual commit destination and draft disposition.
- Update this proposal and PROJECT_STATE.md with a go/no-go result and remaining
  prerequisites; do not treat a synthetic client model as this milestone.

Isolation must be verified before compositor startup: separate D-Bus services,
no host display/session discovery, no installed extension loading, no host settings
writes, no production engine registration. Do not use --replace. Bound runtime,
clean up child processes, and refuse to fall back to the real desktop. If available
components cannot support this isolation, report the specific missing prerequisite
rather than install system packages or weaken the test silently.

Risks: headless Shell may need missing rendering/session services; synthetic input
may not exercise physical dispatch; a nested route is not final user-session
certification; protected-field content can travel through platform internals and
requires a separate collection-boundary audit. No completion date is asserted.

## Current decision

The reproducer approval and execution are complete. The earlier recommendation
to run it is superseded by ORIGINAL_TARGET_FIX_PROPOSAL.md. That experiment was approved and completed with 20 passing checks; see
CLIENT_TRANSACTION_EXPERIMENT.md. It is not a production fix or supported
deployment into existing applications.

No approved platforms, languages or privacy requirements are removed. If required
coverage needs upstream/app cooperation, the owner must decide the architecture;
this is not permission to ship an unsafe fallback.
