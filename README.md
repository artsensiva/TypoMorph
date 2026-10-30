# TypoMorph

**Not release-ready. Do not install or run this code for real typing.**

TypoMorph is a local keyboard-layout correction utility in development for
multilingual writing. This `main` branch still holds the older 0.2.x prototype
source. Current development, documentation and evidence are on
[`diagnostic/device-echo-investigation`](https://github.com/artsensiva/TypoMorph/tree/diagnostic/device-echo-investigation);
the links below are pinned to its checkpoint
[`0bc4f72`](https://github.com/artsensiva/TypoMorph/tree/0bc4f72ac751c9e4e5e093d1818e89cda9a2e15d).

## Current status

- **Automatic correction is disabled in the current development build.** Its
  live `run` path and uinput replacement refuse before capture or mutation,
  because safe text replacement across focus changes and concurrent input is
  not established ([project state](https://github.com/artsensiva/TypoMorph/blob/0bc4f72ac751c9e4e5e093d1818e89cda9a2e15d/docs/PROJECT_STATE.md)).
- **The source on this branch predates that change.** Its `typomorph run`
  command can emit replacement keystrokes through uinput without that refusal.
  It is kept for history, not as the current product.
- **[BUG-001](https://github.com/artsensiva/TypoMorph/blob/0bc4f72ac751c9e4e5e093d1818e89cda9a2e15d/docs/BUGS.md) is open:**
  the owner reports that correction never occurs on Ubuntu 26.04.1 / GNOME 50 /
  Wayland. It is independently unreproduced.
- **The 0.2.x prototype releases are not the planned product.** Do not use the
  GitHub releases v0.2.0–v0.2.2 or this branch's `landing/install.sh`, which
  installs the latest of them and enables its user service. The installed 0.2.2
  prototype can print input characters and buffers to stderr.
- **Prompt improvement and the old licensing model are not part of the
  product.** The `improve-prompt` command, the `prompt-cloud` crate, the
  Free/Pro split and Lemon Squeezy licensing on this branch are legacy. AI
  prompt improvement is excluded (decision D-02) and has been removed from the
  current development build. The approved commercial model is described in
  [LICENSING.md](https://github.com/artsensiva/TypoMorph/blob/0bc4f72ac751c9e4e5e093d1818e89cda9a2e15d/docs/LICENSING.md);
  it is not implemented.

## Approved product scope

These are approved requirements, not working features
([specification](https://github.com/artsensiva/TypoMorph/blob/0bc4f72ac751c9e4e5e093d1818e89cda9a2e15d/docs/SPEC.md),
[decision log](https://github.com/artsensiva/TypoMorph/blob/0bc4f72ac751c9e4e5e093d1818e89cda9a2e15d/docs/DECISIONS.md)).

- **First public release (unchanged):** six input and interface languages
  (English, Russian, Ukrainian, German, French, Spanish); Ubuntu LTS with GNOME
  on Wayland and X11, Windows 11, and macOS; input processed only in transient
  memory, with no typed-text transmission or logging.
- **Linux beta (decision D-38):** Linux is the first beta target. Its
  input/layout languages are the six above plus Greek, Turkish, Polish,
  Portuguese and Italian. The five additions do not extend UI localization.
  Windows and macOS remain longer-term targets. No beta, audience, access model
  or date is committed, and the beta's application coverage is undecided.

## Feasibility evidence

The current work is investigating whether text can be replaced safely in
ordinary applications. All results so far are **synthetic feasibility evidence
from fixed test text in private, disposable sessions, not working application
support**:

- An isolated GTK 4.22.4 transaction prototype passes synthetic storage,
  history, layout and accessibility tests
  ([evidence](https://github.com/artsensiva/TypoMorph/blob/0bc4f72ac751c9e4e5e093d1818e89cda9a2e15d/integrations/gnome/gtk-transaction/README.md)).
- A patched GNOME Text Editor 50.1 fixture still reproduces two unsafe partial
  edits. The prototype refuses on the editor's real document; which guard
  causes that refusal is not isolated
  ([evidence](https://github.com/artsensiva/TypoMorph/blob/0bc4f72ac751c9e4e5e093d1818e89cda9a2e15d/integrations/gnome/text-editor/README.md)).
- On the stock GNOME Wayland input-method route, five of nine fixed stale-edit
  schedules corrupted a field, including an edit delivered to the wrong field
  ([evidence](https://github.com/artsensiva/TypoMorph/blob/0bc4f72ac751c9e4e5e093d1818e89cda9a2e15d/docs/NATIVE_WAYLAND_EXPERIMENT.md)).

None of this establishes working correction in Chrome, Firefox, Text Editor,
Windows or macOS.

## License

The repository retains its existing proprietary license notice in
[LICENSE](LICENSE). It is not an open-source license or a customer agreement.
