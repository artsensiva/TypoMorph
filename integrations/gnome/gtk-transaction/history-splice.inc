/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Standalone replacement entries; never chain into existing action groups.
 */
gboolean
gtk_text_history_typomorph_ready (GtkTextHistory *self)
{
  return !self->applying && !self->irreversible && !self->in_user;
}

void
gtk_text_history_typomorph_record (GtkTextHistory *self, guint offset,
                                   const char *before, const char *after)
{
  Action *action;
  if (!self->enabled)
    return;
  g_assert (gtk_text_history_typomorph_ready (self));
  action = action_new (ACTION_KIND_TYPOMORPH_REPLACE);
  action->u.replace.offset = offset;
  action->u.replace.before = g_strdup (before);
  action->u.replace.after = g_strdup (after);
  action->is_modified_set = !self->is_modified;
  action->is_modified = TRUE;
  self->is_modified = TRUE;
  gtk_text_history_push (self, action);
}

void
gtk_text_history_typomorph_publish (GtkTextHistory *self)
{
  if (self->enabled)
    gtk_text_history_update_state (self);
}

/* 0: ordinary history item; -1: replacement refused; 1: replacement committed.
 * The supplied callback performs storage mutation only, without notifications.
 * The buffer publishes notifications after this function finishes updating both
 * queues and leaves applying state, so callback edits are tracked normally.
 */
int
gtk_text_history_typomorph_step (GtkTextHistory *self, gboolean redo,
                                 gboolean (*replace) (gpointer, guint, const char *, const char *),
                                 gpointer data)
{
  GQueue *from = redo ? &self->redo_queue : &self->undo_queue;
  GQueue *to = redo ? &self->undo_queue : &self->redo_queue;
  Action *barrier = NULL;
  GList *link = redo ? from->head : from->tail;
  Action *action;
  gboolean applied;
  if (!self->enabled || !gtk_text_history_typomorph_ready (self) || link == NULL)
    return 0;
  action = link->data;
  if (action->kind == ACTION_KIND_BARRIER)
    {
      barrier = action;
      link = redo ? link->next : link->prev;
      if (link == NULL)
        return 0;
      action = link->data;
    }
  if (action->kind != ACTION_KIND_TYPOMORPH_REPLACE)
    return 0;
  self->applying = TRUE;
  applied = replace (data, action->u.replace.offset,
                     redo ? action->u.replace.before : action->u.replace.after,
                     redo ? action->u.replace.after : action->u.replace.before);
  if (!applied)
    {
      self->applying = FALSE;
      return -1;
    }
  if (barrier != NULL)
    move_barrier (from, barrier, to, !redo);
  g_queue_unlink (from, &action->link);
  if (redo)
    g_queue_push_tail_link (to, &action->link);
  else
    g_queue_push_head_link (to, &action->link);
  if (action->is_modified_set)
    self->is_modified = redo ? action->is_modified : !action->is_modified;
  gtk_text_history_update_state (self);
  self->applying = FALSE;
  return 1;
}
