# Isolated Text Editor development build plan

Prepared 2026-09-27; authorization updated 2026-09-29. The approved dependencies
are now installed. On 2026-09-30 both baseline and patched editors compiled and
ran in the private compositor. See [actual results](NATIVE_TEXT_EDITOR_RESULTS.md)
for the successful fixed case and two reproduced unsafe partial edits. Earlier
planning and historical progress entries below are retained as context.

## Concrete prerequisite plan

Read-only apt simulation succeeded using current local indexes:
**0 upgraded, 92 newly installed, 0 removed, 43 not upgraded**.
Full resolver output: [NATIVE_BUILD_PACKAGE_PLAN.txt](NATIVE_BUILD_PACKAGE_PLAN.txt).
This includes transitive development libraries and tools, not just the named
packages. It is a snapshot, not a guarantee of later repository availability.
No apt update, download, installation, removal or service change was performed.

Proposed command after approval:

```sh
sudo apt-get --no-install-recommends install build-essential meson ninja-build pkg-config gettext libglib2.0-dev libgtk-4-dev libgtksourceview-5-dev libadwaita-1-dev libspelling-1-dev libeditorconfig-dev
```

Repeat the simulation immediately before installation. Stop for review if it
introduces upgrades/removals or a materially different plan. Do not run a system
upgrade or bypass package verification. A credentials prompt may require the owner;
never request passwords in chat. Disk/download totals were not supplied by this
simulation and must be checked before the actual install.

## Source and build isolation

1. Retrieve official GNOME Text Editor 50.1 source into a temporary checkout;
   record the resolved source commit or archive digest. Keep original source and
   the development patch separate. Do not assume an unofficial mirror's moving
   branch matches the installed binary or contains Ubuntu packaging changes.
2. Build the unpatched baseline first with the normal dependencies and
   -Ddevelopment=true. The verified upstream option chooses
   org.gnome.TextEditor.Devel. Meson/Ninja build commands do not install the app.
   Preserve the baseline for side-by-side evidence, not concurrent ambiguous
   application activation on the same session bus.
3. Add the disabled-by-default experiment option from NATIVE_OWNER_API_AND_PATCH.md.
   Compile a separate patched build with development=true and that option enabled.
   Do not pass the new option to unpatched source: it does not exist there yet.
4. Use workspace/temp build and staging prefixes only. If schemas/resources need
   staging, use a private DESTDIR/prefix and run post-install tooling only against
   that staging tree. Never run sudo meson install or replace /usr/bin files.
   Verify the actual executable/resources/schema lookup before first launch.
5. Check actual dependency versions and fail clearly on missing dependencies.
   Additional packages, downloaded build fallbacks or altered system settings are
   not silently authorized by this plan.

## Runtime isolation and diagnostics

Adapt the existing run_wayland_probe.py isolation into a separate application
runner. Use fresh XDG config/data/cache/state/runtime paths, private session and
system-bus roles without activation directories, memory GSettings and the private
headless compositor. Verify the input service PID before injecting synthetic keys.
Use an absolute development executable path, fresh private app session identity
and an empty synthetic document. No host display, desktop bus, personal recent-file
state, installed extension or host uinput. Never use --replace.

Do not launch the development binary directly into the active desktop as a shortcut.
Do not set HOME to redirect application behavior. Inspect startup before input and
bound execution/cleanup as with the existing harness. The prior headless setup
is process/session isolation, not a general filesystem/security sandbox.
Text Editor can save drafts; all test data is synthetic and any app-created state
must remain in disposable test directories. The TypoMorph adapter itself must not
persist typed content. No personal files are opened.

A test-only local control endpoint may report content-free state and perform
fixed synthetic assertions inside the private app. It must be compiled only with
the experiment and refuse the host session. It is not a production editing API.
Do not build a general text-dump endpoint for testing. Preserve ordinary app
callbacks, undo, spelling and document behavior rather than disabling them to
obtain a pass. Record any unavoidable test configuration difference explicitly.

## Approval scope and completion criteria

Recommended next step: approve installation of the simulated 92-package plan and
implementation of the isolated patch/runner described above. This changes the
development machine's installed build dependencies, but not the installed Text
Editor, GNOME integration or TypoMorph production gate.

Completion means an unpatched baseline and patched real-application result with
range/undo/reentrancy evidence, or a precise documented blocker. Safe mutation is
not assumed achievable. If the existing toolkit cannot supply it, return the
missing primitive for a separate architecture decision; do not widen into a full
GTK/Mutter/browser fork. No commit, publication or upstream submission is included.

## Source-only harness progress, 2026-09-29

The patch now contains fixed-case `typomorph.check` assertions for document state,
callback refusal/faults and ordinary undo/redo. The bounded `assess_probe.py`
component validates exact output sequences plus actual child exit status, and
returns nonzero for known unsafe outcomes. Its six tests use synthetic transcripts;
no actual application result is claimed. See the patch README for the protocol.
The private launcher/control driver, baseline/patched builds and runtime evidence
remain outstanding. This update does not change the dependency-installation scope.

## Prepared checkout and runner (2026-09-30)

Official upstream tag 50.1 resolves to commit
`d1bc58ff790267224be1536a86e1af236a9f7ea7` (annotated tag object
`4eefff81543ad341e3bbe896a71c4fef89a5428d`). All earlier pinned file hashes match.
Temporary trees: `/tmp/typomorph-editor-baseline-50.1` and
`/tmp/typomorph-editor-patched-50.1`. The latter contains the current patch.

After dependencies are present, build without installing either app:

```sh
meson setup /tmp/typomorph-editor-baseline-build /tmp/typomorph-editor-baseline-50.1 -Ddevelopment=true --buildtype=debug --wrap-mode=nodownload
meson compile -C /tmp/typomorph-editor-baseline-build
meson setup /tmp/typomorph-editor-patched-build /tmp/typomorph-editor-patched-50.1 -Ddevelopment=true -Dtypomorph_experiment=true --buildtype=debug --wrap-mode=nodownload
meson compile -C /tmp/typomorph-editor-patched-build
```

Then use `integrations/gnome/text-editor/run_native.py --binary ABSOLUTE_BINARY
--schemas ABSOLUTE_BUILD_DATA_DIRECTORY --case CASE`. `baseline` only checks
five-second process survival, not window/input correctness. Patched cases are
`normal`, `callback-begin-edit`, `gap-edit`, and `callback-delete-edit`.
The last two must remain explicit unsafe results, not passing safety evidence.
The runner must finish cleanly; record exact case outcomes before making claims.
The successful builds used `-gcc-build` directory suffixes, explicit GCC and
temporary itstool/Python dependencies; see NATIVE_TEXT_EDITOR_RESULTS.md for the
executed configuration and evidence. The older command examples above omit these
environment requirements.
