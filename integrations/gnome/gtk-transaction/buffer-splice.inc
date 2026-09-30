/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Appended inside gtktextbuffer.c. Explicitly experimental, headless and opt-in.
 */
guint
gtk_text_buffer_typomorph_revision (GtkTextBuffer *buffer)
{
  g_return_val_if_fail (GTK_IS_TEXT_BUFFER (buffer), 0);
  return _gtk_text_btree_get_chars_changed_stamp (get_btree (buffer));
}

/* GTK ownership is single-threaded. IDs are process-wide to reject a token
 * from another buffer; exhaustion refuses instead of reusing an ID. */
static guint64 typomorph_last_token;

void
gtk_text_buffer_typomorph_forget (GtkTextBuffer *buffer)
{
  g_return_if_fail (GTK_IS_TEXT_BUFFER (buffer));
  buffer->priv->typomorph_token = 0;
  buffer->priv->typomorph_track_edits = 0;
  memset (buffer->priv->typomorph_before, 0, sizeof buffer->priv->typomorph_before);
  memset (buffer->priv->typomorph_after, 0, sizeof buffer->priv->typomorph_after);
}

static void
typomorph_track_edit (GtkTextBuffer *buffer, int start, int end, int added)
{
  GtkTextBufferPrivate *priv = buffer->priv;
  if (priv->typomorph_token == 0)
    return;
  if (priv->typomorph_track_revision != gtk_text_buffer_typomorph_revision (buffer) ||
      ++priv->typomorph_track_edits > 256 ||
      (start < priv->typomorph_track_end && end > priv->typomorph_track_start) ||
      (added > 0 && start > priv->typomorph_track_start && start < priv->typomorph_track_end))
    {
      gtk_text_buffer_typomorph_forget (buffer);
      return;
    }
  if (end <= priv->typomorph_track_start)
    {
      priv->typomorph_track_start += added - (end - start);
      priv->typomorph_track_end += added - (end - start);
    }
}

static void
typomorph_track_committed (GtkTextBuffer *buffer)
{
  if (buffer->priv->typomorph_token != 0)
    buffer->priv->typomorph_track_revision = gtk_text_buffer_typomorph_revision (buffer);
}

guint64
gtk_text_buffer_typomorph_correction_token (GtkTextBuffer *buffer)
{
  g_return_val_if_fail (GTK_IS_TEXT_BUFFER (buffer), 0);
  if (buffer->priv->typomorph_token != 0 &&
      buffer->priv->typomorph_track_revision != gtk_text_buffer_typomorph_revision (buffer))
    gtk_text_buffer_typomorph_forget (buffer);
  return buffer->priv->typomorph_token;
}

static gboolean
typomorph_word (const char *text, int bytes)
{
  const char *cursor;
  if (text == NULL || bytes < 1 || bytes > 128 ||
      !g_utf8_validate (text, bytes, NULL) || g_utf8_strlen (text, bytes) > 32)
    return FALSE;
  for (cursor = text; cursor < text + bytes; cursor = g_utf8_next_char (cursor))
    if (!g_unichar_isalpha (g_utf8_get_char (cursor)))
      return FALSE;
  return TRUE;
}

static gboolean
typomorph_splice_impl (GtkTextBuffer *buffer,
                                guint expected_revision,
                                int offset,
                                const char *original,
                                int original_bytes,
                                const char *replacement,
                                int replacement_bytes,
                                gboolean replay,
                                gboolean remember)
{
  GtkTextIter start, end;
  g_autofree char *actual = NULL;
  g_autofree char *owned_replacement = NULL;
  int characters;

  g_return_val_if_fail (GTK_IS_TEXT_BUFFER (buffer), FALSE);
  /* Unsupported integrations refuse before reading text or changing history.
   * Existing legacy veto handlers must not silently be ignored. */
  if (G_OBJECT_TYPE (buffer) != GTK_TYPE_TEXT_BUFFER ||
      buffer->priv->typomorph_mutating != 0 ||
      (buffer->priv->typomorph_splicing && !replay) || buffer->priv->in_commit_notify ||
      buffer->priv->user_action_count != 0 ||
      _gtk_text_btree_typomorph_has_views (get_btree (buffer)) ||
      (!replay && !gtk_text_history_typomorph_ready (buffer->priv->history)) ||
      (buffer->priv->commit_funcs != NULL && buffer->priv->commit_funcs->len != 0) ||
      gtk_text_tag_table_get_size (get_table (buffer)) != 0 ||
      g_signal_has_handler_pending (buffer, signals[INSERT_TEXT], 0, TRUE) ||
      g_signal_has_handler_pending (buffer, signals[DELETE_RANGE], 0, TRUE) ||
      gtk_text_buffer_typomorph_revision (buffer) != expected_revision ||
      typomorph_last_token == G_MAXUINT64 ||
      !typomorph_word (original, original_bytes) ||
      !typomorph_word (replacement, replacement_bytes))
    return FALSE;

  characters = g_utf8_strlen (original, original_bytes);
  if (g_utf8_strlen (replacement, replacement_bytes) != characters ||
      offset < 0 || offset > gtk_text_buffer_get_char_count (buffer) - characters)
    return FALSE;
  gtk_text_buffer_get_iter_at_offset (buffer, &start, offset);
  gtk_text_buffer_get_iter_at_offset (buffer, &end, offset + characters);
  actual = gtk_text_buffer_get_text (buffer, &start, &end, TRUE);
  if (strlen (actual) != original_bytes || memcmp (actual, original, original_bytes) != 0)
    return FALSE;
  owned_replacement = g_strndup (replacement, replacement_bytes);
  buffer->priv->typomorph_splicing = TRUE;
  if (!_gtk_text_btree_typomorph_splice (&start, &end, owned_replacement))
    {
      if (!replay)
        buffer->priv->typomorph_splicing = FALSE;
      return FALSE;
    }

  /* A changed/modified callback may now make an ordinary edit. It sees the
   * complete committed word. Keep the object alive and refuse nested splices
   * until delivery completes, without suppressing legitimate ordinary edits. */
  if (buffer->priv->log_attr_cache != NULL)
    clear_log_attr_cache (buffer->priv->log_attr_cache);
  gtk_text_buffer_typomorph_forget (buffer);
  if (remember)
    {
      buffer->priv->typomorph_token = ++typomorph_last_token;
      buffer->priv->typomorph_track_start = offset;
      buffer->priv->typomorph_track_end = offset + characters;
      buffer->priv->typomorph_track_revision = gtk_text_buffer_typomorph_revision (buffer);
      memcpy (buffer->priv->typomorph_before, actual, original_bytes + 1);
      memcpy (buffer->priv->typomorph_after, owned_replacement, replacement_bytes + 1);
    }
  if (replay)
    return TRUE;
  g_object_ref (buffer);
  buffer->priv->typomorph_quiet = TRUE;
  gtk_text_history_typomorph_record (buffer->priv->history, offset, actual, owned_replacement);
  buffer->priv->typomorph_quiet = FALSE;
  buffer->priv->typomorph_publishing = TRUE;
  gtk_text_history_typomorph_publish (buffer->priv->history);
  buffer->priv->typomorph_publishing = FALSE;
  g_signal_emit (buffer, signals[CHANGED], 0);
  buffer->priv->typomorph_splicing = FALSE;
  g_object_unref (buffer);
  return TRUE;
}


gboolean
gtk_text_buffer_typomorph_splice (GtkTextBuffer *buffer, guint expected_revision,
                                int offset, const char *original, int original_bytes,
                                const char *replacement, int replacement_bytes)
{
  return typomorph_splice_impl (buffer, expected_revision, offset, original,
                                original_bytes, replacement, replacement_bytes, FALSE, TRUE);
}

static gboolean
typomorph_history_replace (gpointer data, guint offset,
                          const char *original, const char *replacement)
{
  GtkTextBuffer *buffer = data;
  return typomorph_splice_impl (buffer, gtk_text_buffer_typomorph_revision (buffer),
                                offset, original, strlen (original), replacement,
                                strlen (replacement), TRUE, FALSE);
}

static gboolean
typomorph_history_step (GtkTextBuffer *buffer, gboolean redo)
{
  int result;
  g_object_ref (buffer);
  buffer->priv->typomorph_splicing = TRUE;
  buffer->priv->typomorph_quiet = TRUE;
  result = gtk_text_history_typomorph_step (buffer->priv->history, redo,
                                           typomorph_history_replace, buffer);
  buffer->priv->typomorph_quiet = FALSE;
  if (result == 1)
    {
      buffer->priv->typomorph_notify_revision = gtk_text_buffer_typomorph_revision (buffer);
      buffer->priv->typomorph_replaying = TRUE;
      buffer->priv->typomorph_publishing = TRUE;
      gtk_text_history_typomorph_publish (buffer->priv->history);
      buffer->priv->typomorph_publishing = FALSE;
      g_signal_emit (buffer, signals[CHANGED], 0);
      buffer->priv->typomorph_replaying = FALSE;
    }
  buffer->priv->typomorph_splicing = FALSE;
  g_object_unref (buffer);
  return result != 0;
}

/* Reversal is a new chronological history entry, not deletion of older history.
 * Copies prevent record invalidation from aliasing this operation's arguments. */
gboolean
gtk_text_buffer_typomorph_revert (GtkTextBuffer *buffer, guint64 token)
{
  char before[129], after[129];
  g_return_val_if_fail (GTK_IS_TEXT_BUFFER (buffer), FALSE);
  if (token == 0 || gtk_text_buffer_typomorph_correction_token (buffer) != token)
    return FALSE;
  memcpy (before, buffer->priv->typomorph_before, sizeof before);
  memcpy (after, buffer->priv->typomorph_after, sizeof after);
  return typomorph_splice_impl (buffer, buffer->priv->typomorph_track_revision,
                                buffer->priv->typomorph_track_start,
                                after, strlen (after), before, strlen (before), FALSE, FALSE);
}
