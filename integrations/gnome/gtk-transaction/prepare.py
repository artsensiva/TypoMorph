#!/usr/bin/env python3
"""Patch an explicitly supplied disposable GTK 4.22.4 release source tree."""
import argparse
import hashlib
import re
from pathlib import Path

HASHES = {
    'gtktextbufferprivate.h': '0680741e55c73fc88e8ddccaa2c9afe740e0e30669bcea93968d963c4ffddfcf',
    'gtktextview.c': 'df7491b3753bb72e4c6c6118827f42b23c6d547a1f77c5c6129820154d299e55',
    'a11y/gtkatspicontextprivate.h': 'b02a33f07358a6fb38fcd8b9360842d4e8d1d282dc9b532df95a4cd50bd568a2',
    'a11y/gtkatspicontext.c': '769c0bd70bf311e7596c859020e3c0e45c64cabfad6d81424acedddc4fd38284',
    'gtktextlayout.c': 'afe689322884dbb7a9179796c387ec84dacef88ea6e2ba18d6b54efdb0ae7dd0',
    'gtktextlayoutprivate.h': '450fdc31a2c6b36ec5cdbab85deb5e421beb1cc9ae1f818eb1d9ddcfbdcdccd1',
    'gtktexthistory.c': 'dfeb72fe19f5872ea4aae33f853a96715802cccc343a75b8de4838d9db04137f',
    'gtktexthistoryprivate.h': 'bf1e45cda7afa5cbc1e9893f6dfed121881121a1ac82dd84bd6d736ef0aeb26c',
    'gtktextbuffer.c': '98a82aa59d503cbf6b5ebbd13422f5e6ae1b85646bb73366c21f1665ddac549e',
    'gtktextbuffer.h': '87396ced80cb3c22fb5da7dc38cfe9a672b1e11411cf821ae5864edd3587caf3',
    'gtktextbtree.c': '4f9f0c7d830c9e339c64d8dea58349d6c846e7431d9fa6b1a34625c83ff8faa5',
    'gtktextbtreeprivate.h': '3d130800fdfec394a193178ec3197558ad0612ffa85d9df5b0ca428f95ca6790',
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    args = parser.parse_args()
    source = args.source.resolve()
    if not source.is_relative_to('/tmp'):
        parser.error('Only a disposable source directory under /tmp is accepted')
    files = {name: (source / 'gtk' / name).read_bytes() for name in HASHES}
    if any(hashlib.sha256(files[name]).hexdigest() != digest for name, digest in HASHES.items()):
        parser.error('Source identity mismatch or already patched; no files changed')
    text = {name: data.decode() for name, data in files.items()}
    here = Path(__file__).resolve().parent
    text['gtktextbuffer.c'] = text['gtktextbuffer.c'].replace(
        '#include "gtkprivate.h"', '#include "gtkprivate.h"\n#include "gtktextview.h"\n#include "gtkaccessibleprivate.h"\n#include "a11y/gtkatspicontextprivate.h"')
    text['gtktextbuffer.c'] = text['gtktextbuffer.c'].replace(
        'static void gtk_text_buffer_real_undo',
        'static void typomorph_notify_edit (GtkTextBuffer *, GtkAccessibleTextContentChange, int, int);\nstatic void gtk_text_buffer_real_undo', 1)

    text['gtktextbuffer.c'] = text['gtktextbuffer.c'].replace(
        '  guint in_commit_notify : 1;',
        '  guint in_commit_notify : 1;\n  guint typomorph_splicing : 1;\n  guint typomorph_quiet : 1;\n  guint typomorph_publishing : 1;\n  guint typomorph_replaying : 1;\n  guint typomorph_notify_revision;\n  guint64 typomorph_token;\n  guint typomorph_mutating;\n  GPtrArray *typomorph_layouts;\n  GSList *typomorph_views;\n  GPtrArray *typomorph_contexts;\n  gboolean typomorph_notification_failed;\n  guint typomorph_event_offset;\n  char typomorph_event_before[129], typomorph_event_after[129];\n  guint typomorph_track_revision, typomorph_track_edits;\n  int typomorph_track_start, typomorph_track_end;\n  char typomorph_before[129], typomorph_after[129];')
    text['gtktextbuffer.c'] = text['gtktextbuffer.c'].replace(
        'static void gtk_text_buffer_real_undo',
        'static void typomorph_track_edit (GtkTextBuffer *buffer, int start, int end, int added);\nstatic void typomorph_track_committed (GtkTextBuffer *buffer);\nstatic gboolean typomorph_history_step (GtkTextBuffer *buffer, gboolean redo);\nstatic void gtk_text_buffer_real_undo', 1)
    for operation, redo in [('undo', 'FALSE'), ('redo', 'TRUE')]:
        old = f'gtk_text_buffer_real_{operation} (GtkTextBuffer *buffer)\n{{'
        text['gtktextbuffer.c'] = text['gtktextbuffer.c'].replace(
            old, old + f'\n  if (buffer->priv->typomorph_splicing || typomorph_history_step (buffer, {redo}))\n    return;')
    text['gtktextbuffer.c'] = text['gtktextbuffer.c'].replace(
        'gtk_text_buffer_real_changed (GtkTextBuffer *buffer)\n{\n  gtk_text_buffer_set_modified (buffer, TRUE);',
        'gtk_text_buffer_real_changed (GtkTextBuffer *buffer)\n{\n  if (!buffer->priv->typomorph_replaying ||\n      gtk_text_buffer_typomorph_revision (buffer) != buffer->priv->typomorph_notify_revision)\n    gtk_text_buffer_set_modified (buffer, TRUE);')
    state_start = text['gtktextbuffer.c'].index('static void\ngtk_text_buffer_history_change_state (gpointer funcs_data,')
    state_body = text['gtktextbuffer.c'].index('  GtkTextBuffer *buffer = funcs_data;', state_start) + len('  GtkTextBuffer *buffer = funcs_data;')
    text['gtktextbuffer.c'] = text['gtktextbuffer.c'][:state_body] + """
  if (buffer->priv->typomorph_quiet)
    return;
  if (buffer->priv->typomorph_publishing)
    {
      gboolean undo_changed = buffer->priv->can_undo != can_undo;
      gboolean redo_changed = buffer->priv->can_redo != can_redo;
      gboolean modified_changed = buffer->priv->modified != is_modified;
      buffer->priv->can_undo = can_undo;
      buffer->priv->can_redo = can_redo;
      buffer->priv->modified = is_modified;
      if (undo_changed)
        g_object_notify_by_pspec (G_OBJECT (buffer), text_buffer_props[PROP_CAN_UNDO]);
      if (redo_changed)
        g_object_notify_by_pspec (G_OBJECT (buffer), text_buffer_props[PROP_CAN_REDO]);
      if (modified_changed)
        g_signal_emit (buffer, signals[MODIFIED_CHANGED], 0);
      return;
    }
""" + text['gtktextbuffer.c'][state_body:]
    history = text['gtktexthistory.c']
    history = history.replace('  ACTION_KIND_INSERT              = 7,', '  ACTION_KIND_INSERT              = 7,\n  ACTION_KIND_TYPOMORPH_REPLACE    = 8,')
    history = history.replace('  union {', '  union {\n    struct { guint offset; char *before, *after; } replace;', 1)
    history = history.replace('    case ACTION_KIND_INSERT: return "Insert";', '    case ACTION_KIND_INSERT: return "Insert";\n    case ACTION_KIND_TYPOMORPH_REPLACE: return "Replacement (redacted)";')
    history = history.replace('  if (action->kind == ACTION_KIND_INSERT)\n    istring_clear', '  if (action->kind == ACTION_KIND_TYPOMORPH_REPLACE)\n    {\n      g_free (action->u.replace.before);\n      g_free (action->u.replace.after);\n    }\n  else if (action->kind == ACTION_KIND_INSERT)\n    istring_clear', 1)
    chain = history.index('action_chain (Action   *action,')
    guard = history.index('  if (action->kind == ACTION_KIND_GROUP)', chain)
    history = history[:guard] + '  if (action->kind == ACTION_KIND_TYPOMORPH_REPLACE || other->kind == ACTION_KIND_TYPOMORPH_REPLACE)\n    return FALSE;\n\n' + history[guard:]
    for function, body in [
        ('gtk_text_history_printf_action (Action', 'break;'),
        ('action_chain (Action', 'return FALSE;'),
        ('gtk_text_history_apply (GtkTextHistory', 'g_assert_not_reached ();'),
        ('gtk_text_history_reverse (GtkTextHistory', 'g_assert_not_reached ();'),
    ]:
        start = history.index(function)
        switch = history.index('  switch (action->kind)\n    {', start) + len('  switch (action->kind)\n    {')
        history = history[:switch] + '\n    case ACTION_KIND_TYPOMORPH_REPLACE: ' + body + '\n' + history[switch:]
    text['gtktexthistory.c'] = history + '\n' + (here / 'history-splice.inc').read_text()
    history_api = """
gboolean gtk_text_history_typomorph_ready (GtkTextHistory *self);
void gtk_text_history_typomorph_record (GtkTextHistory *self, guint offset,
                                      const char *before, const char *after);
void gtk_text_history_typomorph_publish (GtkTextHistory *self);
int gtk_text_history_typomorph_step (GtkTextHistory *self, gboolean redo,
                                    gboolean (*replace) (gpointer, guint, const char *, const char *),
                                    gpointer data);
"""
    text['gtktexthistoryprivate.h'] = text['gtktexthistoryprivate.h'].replace('G_END_DECLS', history_api + '\nG_END_DECLS')
    # Ordinary history notifications may run before the associated storage edit.
    # Refuse experimental transactions for the entire default mutation handler.
    for function, first_work in [
        ('gtk_text_buffer_real_insert_text', '  gtk_text_history_text_inserted'),
        ('gtk_text_buffer_real_delete_range', '  if (gtk_text_history_get_enabled'),
        ('gtk_text_buffer_real_insert_paintable', '  _gtk_text_btree_insert_paintable'),
        ('gtk_text_buffer_real_insert_anchor', '  _gtk_text_btree_insert_child_anchor'),
    ]:
        start = text['gtktextbuffer.c'].index('static void\n' + function + ' (')
        end = text['gtktextbuffer.c'].index('\n}', start)
        body = text['gtktextbuffer.c'][start:end]
        body = body.replace(first_work, '  buffer->priv->typomorph_mutating++;\n' + first_work, 1)
        text['gtktextbuffer.c'] = text['gtktextbuffer.c'][:start] + body + '\n  buffer->priv->typomorph_mutating--;' + text['gtktextbuffer.c'][end:]
    # Track only mutations that actually reach storage, after any veto callbacks.
    for call, preflight in [
        ('_gtk_text_btree_insert (iter, text, len);',
         'typomorph_track_edit (buffer, gtk_text_iter_get_offset (iter), gtk_text_iter_get_offset (iter), g_utf8_strlen (text, len));'),
        ('_gtk_text_btree_delete (start, end);',
         'typomorph_track_edit (buffer, MIN (gtk_text_iter_get_offset (start), gtk_text_iter_get_offset (end)), MAX (gtk_text_iter_get_offset (start), gtk_text_iter_get_offset (end)), 0);'),
    ]:
        assert text['gtktextbuffer.c'].count(call) == 2
        text['gtktextbuffer.c'] = text['gtktextbuffer.c'].replace(
            call, preflight + '\n      ' + ('_gtk_text_btree_typomorph_before_insert (get_btree (buffer), iter);' if 'insert' in call else '_gtk_text_btree_typomorph_before_delete (get_btree (buffer), start, end);') + '\n      ' + call + '\n      _gtk_text_btree_typomorph_after_edit (get_btree (buffer));\n      typomorph_track_committed (buffer);')
    view = text['gtktextview.c']
    for handler in ['insert_text_handler', 'delete_range_handler']:
        name = 'gtk_text_view_' + handler
        view, count = re.subn(
            r'      g_signal_(?:connect(?:_after)?|handlers_disconnect_by_func) '
            r'\(priv->buffer,\s*(?:"(?:insert-text|delete-range)",\s*)?'
            r'(?:G_CALLBACK \(' + name + r'\)|' + name + r'),\s*text_view\);\n', '', view)
        assert count == 2, name
        view, count = re.subn(r'static void\s+' + name + r'\s+\([^;{]+\);\n', '', view)
        assert count == 1, name
        view, count = re.subn(r'static void\n' + name + r' \([^;{]+\)\n\{.*?\n\}\n', '', view, flags=re.S)
        assert count == 1, name
    view = view.replace('  if (old_buffer != NULL)\n    {',
                        '  if (old_buffer != NULL)\n    {\n      _gtk_text_buffer_typomorph_view (old_buffer, text_view, FALSE);', 1)
    view = view.replace('      g_object_ref (buffer);',
                        '      g_object_ref (buffer);\n      _gtk_text_buffer_typomorph_view (buffer, text_view, TRUE);', 1)
    text['gtktextview.c'] = view
    text['gtktextbufferprivate.h'] += '\nvoid _gtk_text_buffer_typomorph_view (GtkTextBuffer *, GtkTextView *, gboolean);\n'
    # Ordinary accessibility notifications retain remove-before/insert-after semantics.
    start = text['gtktextbuffer.c'].index('static void\ngtk_text_buffer_real_insert_text (')
    end = text['gtktextbuffer.c'].index('\n}', start)
    body = text['gtktextbuffer.c'][start:end].replace(
        '  g_signal_emit (buffer, signals[CHANGED], 0);',
        '  typomorph_notify_edit (buffer, GTK_ACCESSIBLE_TEXT_CONTENT_CHANGE_INSERT, gtk_text_iter_get_offset (iter) - g_utf8_strlen (text, len), gtk_text_iter_get_offset (iter));\n  g_signal_emit (buffer, signals[CHANGED], 0);')
    text['gtktextbuffer.c'] = text['gtktextbuffer.c'][:start] + body + text['gtktextbuffer.c'][end:]
    start = text['gtktextbuffer.c'].index('static void\ngtk_text_buffer_real_delete_range (')
    end = text['gtktextbuffer.c'].index('\n}', start)
    body = text['gtktextbuffer.c'][start:end].replace(
        '  buffer->priv->typomorph_mutating++;',
        '  buffer->priv->typomorph_mutating++;\n  typomorph_notify_edit (buffer, GTK_ACCESSIBLE_TEXT_CONTENT_CHANGE_REMOVE, gtk_text_iter_get_offset (start), gtk_text_iter_get_offset (end));')
    text['gtktextbuffer.c'] = text['gtktextbuffer.c'][:start] + body + text['gtktextbuffer.c'][end:]
    text['gtktextlayout.c'] = text['gtktextlayout.c'].replace(
        'struct _GtkTextLayoutPrivate\n{',
        'struct _GtkTextLayoutPrivate\n{\n  gboolean typomorph_deferred;')
    text['gtktextlayout.c'] = text['gtktextlayout.c'].replace(
        'gtk_text_layout_invalidated (GtkTextLayout *layout)\n{',
        'gtk_text_layout_invalidated (GtkTextLayout *layout)\n{\n  if (GTK_TEXT_LAYOUT_GET_PRIVATE (layout)->typomorph_deferred)\n    return;')
    # Internal cache/cursor maintenance moves to explicit storage hooks. Never
    # whitelist or disconnect application-provided legacy handlers at runtime.
    for handler in ['before_buffer_insert_text', 'after_buffer_insert_text',
                    'before_buffer_delete_range', 'after_buffer_delete_range']:
        pattern = (r'      g_signal_(?:connect(?:_after)?|handlers_disconnect_by_func) '
                   r'\(layout->buffer,\s*(?:"(?:insert-text|delete-range)",\s*)?'
                   r'G_CALLBACK \(gtk_text_layout_' + handler + r'\),\s*layout\);\n')
        text['gtktextlayout.c'], count = re.subn(pattern, '', text['gtktextlayout.c'])
        assert count == 2, handler
    for handler in ['after_buffer_insert_text', 'after_buffer_delete_range']:
        name = 'gtk_text_layout_' + handler
        text['gtktextlayout.c'], count = re.subn(
            r'static void\s+' + name + r'\s+\([^;{]+\);\n', '', text['gtktextlayout.c'])
        assert count == 1, name
        text['gtktextlayout.c'], count = re.subn(
            r'static void\n' + name + r' \([^;{]+\)\n\{.*?\n\}\n', '',
            text['gtktextlayout.c'], flags=re.S)
        assert count == 1, name
    text['gtktextlayout.c'] += '\n' + (here / 'layout-splice.inc').read_text()
    text['gtktextlayoutprivate.h'] += """
/* Disposable transaction prototype; private layout batching. */
gboolean gtk_text_layout_typomorph_ready (GtkTextLayout *layout);
void gtk_text_layout_typomorph_defer (GtkTextLayout *layout, gboolean deferred);
void gtk_text_layout_typomorph_publish (GtkTextLayout *layout);
void gtk_text_layout_typomorph_before_insert (GtkTextLayout *layout, GtkTextIter *iter);
void gtk_text_layout_typomorph_before_delete (GtkTextLayout *layout, GtkTextIter *start, GtkTextIter *end);
void gtk_text_layout_typomorph_after_edit (GtkTextLayout *layout);
"""
    text['gtktextbuffer.c'] += '\n' + (here / 'view-splice.inc').read_text() + '\n' + (here / 'buffer-splice.inc').read_text()
    text['gtktextbtree.c'] += '\n' + (here / 'btree-splice.inc').read_text()
    public = '''
/* Experimental TypoMorph API: disposable headless prototype only. */
GDK_AVAILABLE_IN_ALL
guint gtk_text_buffer_typomorph_revision (GtkTextBuffer *buffer);
GDK_AVAILABLE_IN_ALL
guint64 gtk_text_buffer_typomorph_correction_token (GtkTextBuffer *buffer);
GDK_AVAILABLE_IN_ALL
gboolean gtk_text_buffer_typomorph_revert (GtkTextBuffer *buffer, guint64 token);
GDK_AVAILABLE_IN_ALL
void gtk_text_buffer_typomorph_forget (GtkTextBuffer *buffer);
GDK_AVAILABLE_IN_ALL
gboolean gtk_text_buffer_typomorph_splice (GtkTextBuffer *buffer,
                                          guint expected_revision,
                                          int offset,
                                          const char *original,
                                          int original_bytes,
                                          const char *replacement,
                                          int replacement_bytes);

'''
    public += '\ngboolean gtk_text_buffer_typomorph_notification_failed (GtkTextBuffer *buffer);\n'
    private = '''
gboolean _gtk_text_btree_typomorph_layouts_supported (GtkTextBTree *tree);
gboolean _gtk_text_btree_typomorph_splice (GtkTextIter *start,
                                        GtkTextIter *end,
                                        const char *replacement,
                                        GPtrArray **layouts);
void _gtk_text_btree_typomorph_before_insert (GtkTextBTree *tree, GtkTextIter *iter);
void _gtk_text_btree_typomorph_before_delete (GtkTextBTree *tree, GtkTextIter *start, GtkTextIter *end);
void _gtk_text_btree_typomorph_after_edit (GtkTextBTree *tree);
void _gtk_text_btree_typomorph_thaw (GPtrArray *layouts);
void _gtk_text_btree_typomorph_publish (GPtrArray *layouts, GtkTextBuffer *buffer);

'''
    text['gtktextbuffer.h'] = text['gtktextbuffer.h'].replace('G_END_DECLS', public + 'G_END_DECLS')
    text['gtktextbtreeprivate.h'] = text['gtktextbtreeprivate.h'].replace('G_END_DECLS', private + 'G_END_DECLS')
    text['a11y/gtkatspicontext.c'] += '\n' + (here / 'a11y-splice.inc').read_text()
    text['a11y/gtkatspicontextprivate.h'] = text['a11y/gtkatspicontextprivate.h'].replace(
        'G_END_DECLS', 'gboolean gtk_at_spi_context_typomorph_ready (GtkAtSpiContext *self);\ngboolean gtk_at_spi_context_typomorph_publish (GtkAtSpiContext *self, guint offset,\n                                              const char *before, int before_bytes,\n                                              const char *after, int after_bytes);\n\nG_END_DECLS')
    for name, content in text.items():
        (source / 'gtk' / name).write_text(content)
    print('Pinned disposable GTK source patched; installed GTK unchanged.')


if __name__ == '__main__':
    main()
