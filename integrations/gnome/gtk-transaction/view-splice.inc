/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Internal GtkTextView registration and post-commit accessibility publication.
 * Registration does not authorize field ownership or production correction.
 */
void
_gtk_text_buffer_typomorph_view (GtkTextBuffer *buffer, GtkTextView *view, gboolean attach)
{
  if (attach)
    {
      g_assert (g_slist_find (buffer->priv->typomorph_views, view) == NULL);
      buffer->priv->typomorph_views = g_slist_prepend (buffer->priv->typomorph_views, view);
    }
  else
    buffer->priv->typomorph_views = g_slist_remove (buffer->priv->typomorph_views, view);
}

static GPtrArray *
typomorph_view_snapshot (GtkTextBuffer *buffer)
{
  GPtrArray *views = g_ptr_array_new_with_free_func (g_object_unref);
  for (GSList *l = buffer->priv->typomorph_views; l; l = l->next)
    g_ptr_array_add (views, g_object_ref (l->data));
  return views;
}

static void
typomorph_notify_edit (GtkTextBuffer *buffer, GtkAccessibleTextContentChange change,
                      int start, int end)
{
  g_autoptr(GPtrArray) views = typomorph_view_snapshot (buffer);
  for (guint i = 0; i < views->len; i++)
    {
      GtkTextView *view = g_ptr_array_index (views, i);
      if (gtk_text_view_get_buffer (view) == buffer)
        gtk_accessible_text_update_contents (GTK_ACCESSIBLE_TEXT (view), change, start, end);
    }
}

static GPtrArray *
typomorph_capture_contexts (GtkTextBuffer *buffer)
{
  /* Bound transaction metadata independently of whether each view has a layout. */
  guint count = 0;
  for (GSList *l = buffer->priv->typomorph_views; l; l = l->next)
    if (++count > 4)
      return NULL;
  g_autoptr(GPtrArray) views = typomorph_view_snapshot (buffer);
  g_autoptr(GPtrArray) contexts = g_ptr_array_new_with_free_func (g_object_unref);
  for (guint i = 0; i < views->len; i++)
    {
      GtkTextView *view = g_ptr_array_index (views, i);
      if (G_OBJECT_TYPE (view) != GTK_TYPE_TEXT_VIEW ||
          gtk_text_view_get_buffer (view) != buffer || !gtk_text_view_get_editable (view))
        return NULL;
      GtkATContext *context = gtk_accessible_get_at_context (GTK_ACCESSIBLE (view));
      if (context == NULL)
        return NULL;
      if (!GTK_IS_AT_SPI_CONTEXT (context) ||
          !gtk_at_spi_context_typomorph_ready (GTK_AT_SPI_CONTEXT (context)))
        {
          g_object_unref (context);
          return NULL;
        }
      g_ptr_array_add (contexts, context);
    }
  return g_steal_pointer (&contexts);
}

static void
typomorph_publish_contexts (GtkTextBuffer *buffer)
{
  GtkTextBufferPrivate *priv = buffer->priv;
  for (guint i = 0; i < priv->typomorph_contexts->len; i++)
    if (!gtk_at_spi_context_typomorph_publish (
          g_ptr_array_index (priv->typomorph_contexts, i), priv->typomorph_event_offset,
          priv->typomorph_event_before, strlen (priv->typomorph_event_before),
          priv->typomorph_event_after, strlen (priv->typomorph_event_after)))
      priv->typomorph_notification_failed = TRUE;
  g_clear_pointer (&priv->typomorph_contexts, g_ptr_array_unref);
  memset (priv->typomorph_event_before, 0, sizeof priv->typomorph_event_before);
  memset (priv->typomorph_event_after, 0, sizeof priv->typomorph_event_after);
}

gboolean
gtk_text_buffer_typomorph_notification_failed (GtkTextBuffer *buffer)
{
  g_return_val_if_fail (GTK_IS_TEXT_BUFFER (buffer), TRUE);
  return buffer->priv->typomorph_notification_failed;
}
