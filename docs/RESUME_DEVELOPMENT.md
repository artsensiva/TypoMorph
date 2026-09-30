# Resume TypoMorph development

This is a recovery prompt, **not an active automation or scheduled task**.
Updated: 2026-10-01.

## Continuation prompt

> Continue TypoMorph in /home/artem/TypoMorph. Read AGENTS.md and
> docs/PROJECT_STATE.md, inspect the relevant working diff, and resume the
> approved work without restarting product discovery. Preserve all existing
> uncommitted changes. The isolated GTK transaction prototype now has ordinary
> undo/redo and bounded targeted reversal preserving later disjoint edits, with
> 29 buffer/history groups passing under ASan/UBSan and leak
> detection. An attached-view refusal probe passes behavioral checks; its
> leak-enabled run reports font-stack allocations. Read
> integrations/gnome/gtk-transaction/README.md for exact evidence and limits.
> A cache-only layout draft was rejected: GtkTextLayout itself registers legacy
> edit observers. Two additional layout refusal tests preserve this finding.
> Next: compatible layout observers, GtkTextView accessibility notifications, and binding correction-record
> cancellation to field/focus ownership. Retain the original failing real-editor
> fixtures and production refusal gates. Do not capture personal typing, install
> a toolkit fork, commit, push or publish without the required authorization.
> Keep PROJECT_STATE.md synchronized with verified progress and blockers.

## Current artifacts

- Source fragments, pinned-source preparer, tests and private view runner:
  `integrations/gnome/gtk-transaction/`.
- Disposable GTK 4.22.4 sources/build: `/tmp/typomorph-gtk-transaction/`.
  They may disappear after reboot; reproduction instructions are in the README.
- Original real-editor failure evidence: `docs/NATIVE_TEXT_EDITOR_RESULTS.md`.
- No installed app was replaced and ordinary application correction remains broken.

## Automatic restart status

The current IDE session exposes no supported usage-reset scheduling control.
Nothing will automatically restart this session merely because the limit resets.
OpenAI documents local scheduled tasks in the desktop app, with the computer and
app running; that is distinct from a verified usage-reset trigger. No such task
was configured here. See [official scheduled-task documentation](https://learn.chatgpt.com/docs/automations?surface=app).

After access returns, send the continuation prompt above (or “Continue” in this
thread). Do not use tight retry loops or alternate credentials to work around a
usage limit.
