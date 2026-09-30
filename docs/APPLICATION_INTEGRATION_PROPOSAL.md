# Existing-application integration proposal

Date: 2026-09-27. Status: native-cooperation-first direction approved; API/patch specification
and isolated build plan prepared in NATIVE_OWNER_API_AND_PATCH.md and
NATIVE_TEXT_EDITOR_BUILD.md. Implementation approval pending.
No production architecture, package installation, upstream submission or scope
change is authorized by this document.

## Recommendation

Choose **native cooperation first** for desktop coverage: final safety checks,
revision validation and mutation must live in the actual field-owning client.
Treat a browser extension as a conditional, separately qualified adapter, not as
proof of native coverage. Do not ship private GTK/Mutter/Chrome forks or extend
the standalone Gtk.Text fixture indefinitely.

There is currently no demonstrated deployable path that safely corrects both
unmodified Text Editor and unmodified Chrome through our daemon alone. This is
an evidence limit, not proof that no such integration can ever be developed.
The 20 passing client-owned tests establish a restricted contract. They neither
repair the native delayed-key failure nor provide a hook into existing apps.

## Inspected targets and evidence limits

Installed package inventory: GNOME Text Editor 50.1, GTK 4.22.4, GtkSourceView
5.18.0, Google Chrome 153.0.8010.47. This inventory does not establish Chrome's
active Ozone backend or text-input protocol. No personal desktop content was read.

The version-tagged [Text Editor document source](https://raw.githubusercontent.com/GNOME/gnome-text-editor/50.1/src/editor-document.c)
defines EditorDocument as a GtkSourceBuffer subclass and overrides insertion,
deletion and change handling. Therefore our Gtk.EntryBuffer swap cannot be
transplanted into its document without addressing its document lifecycle.
The inspected file is not a full application/plugin API audit. A supported
external TypoMorph transaction hook has not been established.

[GtkTextBuffer user actions](https://docs.gtk.org/gtk4/method.TextBuffer.begin_user_action.html)
group operations for undo. That is not a documented conditional-edit or rollback
guarantee. The online documentation is newer than installed GTK; confirm exact
APIs against installed headers/source during implementation.

[Chromium's current Wayland input context](https://raw.githubusercontent.com/chromium/chromium/main/ui/ozone/platform/wayland/host/wayland_input_method_context.cc)
contains separate v1/v3 protocol clients forwarding commit strings to the input
context. This is moving-main source, not verification of the installed Chrome
build. Fetching the matching 153.0.8010.47 file failed. GTK test outcomes must not
be reported as Chrome failures or compatibility results.

## Concrete integration boundaries

| Target/path | Available boundary | Required work / missing guarantee | Deployment consequence |
| --- | --- | --- | --- |
| Text Editor document | EditorDocument / GtkSourceBuffer in the application process | Track document/view lifetimes, edit revision, marks/selection, focus and composition; reject stale candidates; apply a bounded range through document editing and undo without callback corruption | Requires app/toolkit cooperation or a development source patch; no demonstrated drop-in external adapter |
| Text Editor search field | Separate field owner and lifecycle | No permission inherited from the document; new field must establish its own safety and revision | Document integration alone does not cover all fields |
| Native GNOME route | Shell IBus context, Mutter events, Wayland client | Preserve origin and generation through asynchronous transport; validate at actual field owner; separate optional correction from consumed original input | Potential coordinated changes across client/toolkit, compositor, Shell and IBus; not a Shell-extension-only fix |
| Native Chrome web field | Browser input context plus renderer/editor state | Bind result to document/frame and field lifetime/revision; reject navigation or focus races at the renderer owner; prove undo and composition handling | No supported TypoMorph external transaction API established; browser cooperation may be needed |
| Chrome extension on a permitted page | Content script can inspect and modify page DOM | Persistent element identity, document/frame generation, bounded snapshots, current safety, expected range and editor-specific mutation/undo semantics | Can use stock browser, but needs extension permission, packaging and supported-editor qualification |
| Browser UI / unpermitted or unsupported page | Not established by page adapter | Requires independently qualified native behavior | Extension success cannot certify these contexts |

### Proposed owner contract, not a new existing API

A client advertises a capability only after it can perform the full operation.
A request binds client session, document/frame, field lifetime, focus generation,
edit revision, exact range and expected text. The owner releases only a bounded
ordinary-field snapshot to the local analyzer; protected/unknown state refuses
before capture. Prefer UTF-8 at a defined IPC boundary and explicit conversions:
DOM offsets are UTF-16 code units, so Rust byte offsets cannot be reused blindly.

The reply is only a suggestion. The owner checks identity, current eligibility,
revision and selection again, then performs one owned editor action. No await or
focus-based global injection sits between validation and mutation. Refusal leaves
original input untouched. Disconnect, timeout, navigation and pause invalidate
pending work. There is no input replay on reconnection.

Undo must reverse the accepted correction without deleting subsequent typing.
Grouping an undo action alone does not prove this. Any text/layout coordination
must use the same ownership decision; do not switch the user's current layout
from a stale completion. These remain unsolved implementation requirements.

## Browser adapter: reusable parts and blockers

[Chrome content scripts](https://developer.chrome.com/docs/extensions/develop/concepts/content-scripts)
provide page DOM access with declared/programmatic permissions and an isolated
JavaScript world. Isolation does not grant ownership of page editor state.
[The HTML range replacement API](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#dom-textarea/input-setrangetext)
specifies range/selection changes for applicable inputs and textareas; it does not
establish the required editor undo contract. Rich editors and page callbacks need
separate qualification, not generic value assignment plus an input event.

Local legacy inventory:
- extensions/chrome/background.js reads a selection, awaits native messaging,
  then uses the current activeElement or selection for replacement. No original
  element/document revision is carried. Its reader lacks a protected-type guard.
- The Chrome manifest has on-demand activeTab/scripting access, not implemented
  continuous field ownership. Existing cloud/prompt actions contradict approved
  no-AI/no-transmission scope.
- crates/native-host runs independently of the daemon. It does not implement
  shared pause, entitlement, connection generation or exclusive correction ownership.
- Placeholder host IDs and Free/Pro UI remain in the legacy integration.

These are source findings, not a live exploit test. Do not enable this legacy
extension to test normal personal typing. No files were changed in that subsystem.

If an extension is later approved, replace that flow with an owner kept in the
content script and a validating service-worker/native-host bridge. Native messaging
is [available through extension pages/service workers](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging),
not directly in content scripts. Validate sender/document identity and bounds;
never log typed payloads. Use transient local IPC only.

Proposed missing/stopped-desktop behavior: no input analysis or correction until
an authenticated local desktop session authorizes it; keep ordinary typing and
show connection status. Do not silently launch a standalone analyzer. This is a
proposed lifecycle choice requiring owner approval, not implemented behavior.
Exclusive ownership must be enforceable on both native and extension paths;
if they cannot agree on the same field, do not run both corrections there.
Disabling all native browser fields merely because an extension is absent would
contradict D-11 and SPEC.md and is not proposed.

## Release-policy conflicts that need an owner decision

Chrome's documented [external-install methods](https://developer.chrome.com/docs/extensions/how-to/distribute/install-extensions)
require Chrome Web Store hosting for normal Windows/macOS external installs;
Linux supports additional methods. Developer-mode loading is a test workflow,
not our defined consumer delivery plan. A mandatory cross-platform Chrome
extension would therefore require revisiting deferred store publication or
agreeing another supported delivery policy. Do not silently change that decision.

A shipped modified Text Editor/browser or a maintained system-library fork would
also be a new distribution/maintenance commitment, not a routine daemon change.
The no-typed-text-persistence rule governs TypoMorph's collection; it must not be
misrepresented as disabling an editor's own document save/autosave behavior.
No editor setting was changed during this work.

## Decision options and bounded next work

**1. Native cooperation first (recommended).** Preserve the desktop-wide intent
and avoid making extensions mandatory before native Chrome measurement. Prepare
an implementation-ready owner API and a development patch plan for Text Editor's
actual document/range/undo path, plus the minimum transport changes needed to
reach that owner. Explicitly separate an application-specific patch from toolkit
coverage. Draft upstream questions locally; do not send them. Acceptance requires
original input preservation, stale refusal, undo with later typing, protected
search/field transitions and reentrant callback handling. Do not build another
whole-buffer Gtk.Text replacement and call it application integration.

The next deliverable is an API/patch specification and exact isolated-build setup,
not permission to install system packages or deploy a fork. Local preflight found
no meson/ninja on PATH and no pkg-config development metadata for gio-2.0, gtk4,
gtksourceview-5, libadwaita-1 or libspelling-1. Runtime libraries are insufficient.
The [Text Editor 50.1 build](https://raw.githubusercontent.com/GNOME/gnome-text-editor/50.1/meson.build)
lists those dependency families and a separate development application ID.
Do not promise a successful build until dependencies and isolation are verified.
External upstream acceptance and delivery timing remain unknown.

**2. Browser adapter first.** Qualify a new transaction owner in a disposable
Chrome profile on synthetic local pages; never enable the old extension on the
personal profile. First establish the native Chrome route and baseline, then
measure extension value on ordinary inputs/textarea, frames, navigation, typing
races, protected fields, page mutation and undo. This may deliver useful browser
capability sooner, but leaves Text Editor/native coverage unresolved and triggers
the lifecycle/distribution decisions above before production inclusion.

Both options are development priorities, not permission to reduce release scope.
No production readiness or implementation duration estimate is justified yet.
