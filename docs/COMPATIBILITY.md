# Compatibility and rollout matrix

Status: approved scope, with unverified details explicitly marked.
Date: 2026-09-24. No combination below has been certified by this documentation update.

## 1. Evidence labels

- **Required**: part of the first public release; not a claim of working code.
- **Owner-reported**: supplied by the owner; not independently reproduced.
- **Conditional**: needs feasibility evidence and a further product decision.
- **Deferred**: outside the first public release.
- **TBD**: must be resolved before claiming the affected support.

Language detection, a word list, a layout identifier, a successful build, and safe correction in a real application are different evidence levels. Only the last, with the required tests, supports a public compatibility claim.

## 2. Operating systems and hardware

| Target | CPU | Session | Requirement and evidence |
| --- | --- | --- | --- |
| Ubuntu LTS with GNOME | x86-64 | Wayland | Required. Owner uses Ubuntu 26.04.1 LTS / GNOME 50; BUG-001 is reported here |
| Ubuntu LTS with GNOME | x86-64 | X11 | Required. Exact LTS/GNOME version and available test session TBD; do not assume the owner's current version supplies this session |
| Windows 11 | x86-64 | Standard desktop | Required. Owner has a machine; release/build number TBD |
| macOS | Apple Silicon | Standard desktop | Required. A friend's test machine is planned, not yet confirmed; OS version TBD |
| macOS | Intel | Standard desktop | Required. A friend's test machine is planned, not yet confirmed; OS version TBD |
| Windows 10 | Any | Any | Deferred |
| Linux/Windows ARM | ARM | Any | Deferred |
| Other Linux distributions, KDE, other window managers | Any | Any | Deferred |

The owner's Linux reference machine is a Lenovo ThinkCentre M720q, Intel Core i5-8400T, 16 GiB RAM, Intel UHD Graphics 630, with kernel 7.0.0-31-generic. These are owner-reported facts, not a minimum-hardware specification.

Publish a finite list of tested versions. Do not promise every vendor-supported version, every Ubuntu LTS, or all future releases. Obtaining both Mac architectures remains a release dependency.

## 3. Language and layout scope

| Language | Required layout families | Exact IDs/evidence |
| --- | --- | --- |
| English | Standard US and UK on each target OS | TBD; current code has a partial US table only |
| Russian | Standard system Cyrillic/JCUKEN layouts; native Apple/PC variants where available on macOS | TBD; current RU table is incomplete |
| Ukrainian | Standard system layouts; separately verified native Apple/PC variants where available | TBD; current `ua` target identifier is not a complete Ukrainian keymap |
| German | Germany QWERTZ | TBD; no claim for Swiss or other regional variants |
| French | France traditional AZERTY, explicitly identified per OS | TBD; do not silently include all modern/alternative AZERTY variants |
| Spanish | Spain and Latin American system layouts | TBD; map each OS's actual options rather than assuming identical names |

Standard family names are product scope, not final technical mappings. For every variant record OS version, identifier, physical keyboard assumptions, modifiers, punctuation, diacritics, and test evidence before release.

Excluded now: US International, Dvorak, Colemak, French Belgian/Canadian variants, Swiss German variants, phonetic/transliteration layouts, and arbitrary custom layouts.

Only user-selected installed layouts participate. Detect new installed layouts and offer enrollment; never silently add them or install new OS layouts.

## 4. Conversion directions and quality

Cover all selected supported directed layout conversions where a safe, unambiguous result exists. Uncertainty leads to automatic abstention or a manual preview/choice. Correct text on a suitable layout remains unchanged even when its language differs from the layout label.

Track:
- language-detection evidence;
- physical/text mapping coverage;
- manual conversion;
- automatic correction;
- safe undo;
- modifier/dead-key/case/punctuation coverage;
- application safety and direct-selection access.

For each direction record false corrections and misses separately. Do not use one global accuracy number to hide a failing language or layout variant.

## 5. Browsers and application contexts

| Browser | Ubuntu GNOME | Windows 11 | macOS |
| --- | --- | --- | --- |
| Chrome | Required | Required | Required |
| Firefox | Required | Required | Required |
| Edge | Not a required release claim | Required | Not a required release claim |
| Safari | N/A | N/A | Required through the native application where safe |

Exact browser versions, packaging/backend details, editable controls, messaging editors, and protected fields must be recorded during testing. The additional required native editor, messenger, and document-application list is TBD. "Text Editor" in BUG-001 is not yet an exact package/version identification.

No browser-wide exclusion is triggered merely by an absent extension. Unknown/protected fields remain unavailable under the shared safety policy.

Direct selection access is mandatory for selection correction; no clipboard fallback. Terminals, IDEs, games, RDP/VNC, and VM windows are excluded by default. User overrides do not establish general support for those contexts or bypass protected/unknown-field suppression. Elevated applications are unsupported wherever ordinary-user permissions are insufficient.

## 6. Extension decision

Chrome/Edge and Firefox extensions are conditional, not committed release deliverables. First test native browser context and replacement. Record concrete gaps and return the inclusion decision to the owner.

If extensions are needed, resolve shared settings/entitlements, exclusive ownership of a correction, missing-app/not-running behavior, permissions, and delivery method before implementation. Existing Native Messaging code is not proof that the new lifecycle is settled. Browser-store distribution cannot be assumed while store publication is deferred.

Safari extension remains deferred, while native Safari compatibility remains in the required matrix.

## 7. Composition and future languages

IME correction is deferred. Active IME means suspension and buffer clearing. Safe coexistence with IMEs is required.

Dead keys used by the approved alphabetic layouts are not an excuse to exclude those languages. Do not interfere with an unfinished sequence; consider the completed character only when safe.

Next expansion priority: Greek, Turkish, Polish, and Portuguese. No additional language becomes mandatory merely because its dictionary is available. Later candidates include Italian, Czech, Bulgarian, Hebrew, and Arabic; IME-language support requires separate discovery and feasibility work.

## 8. UI languages and release evidence record

The UI is localized into the same six approved languages, independently of selected input layouts. Future input languages do not automatically expand the UI translation commitment.

Use the following evidence fields for every tested combination:

| Field | Required record |
| --- | --- |
| Build | Commit/artifact identity and release configuration |
| Environment | OS/build, architecture, session, permissions, physical keyboard |
| Application | Exact version, package/backend, field/control type |
| Layouts | Source/target IDs, selected set, modifiers |
| Results | Test IDs, expected/actual output, latency/accuracy data, safety outcome |
| Status | Passed, failed, blocked, or untested; tester and date |

Populate records during authorized validation. An empty matrix is not a pass.
