/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Cache invalidation happens during commit; callbacks run after history is ready.
 */
gboolean
gtk_text_layout_typomorph_ready (GtkTextLayout *layout)
{
  return G_OBJECT_TYPE (layout) == GTK_TYPE_TEXT_LAYOUT &&
         !GTK_TEXT_LAYOUT_GET_PRIVATE (layout)->typomorph_deferred &&
         layout->preedit_len == 0;
}

void
gtk_text_layout_typomorph_defer (GtkTextLayout *layout, gboolean deferred)
{
  GTK_TEXT_LAYOUT_GET_PRIVATE (layout)->typomorph_deferred = deferred;
}

void
gtk_text_layout_typomorph_publish (GtkTextLayout *layout)
{
  gtk_text_layout_invalidated (layout);
}

void
gtk_text_layout_typomorph_before_insert (GtkTextLayout *layout, GtkTextIter *iter)
{
  gtk_text_layout_before_buffer_insert_text (layout->buffer, iter, NULL, 0, layout);
}

void
gtk_text_layout_typomorph_before_delete (GtkTextLayout *layout, GtkTextIter *start, GtkTextIter *end)
{
  gtk_text_layout_before_buffer_delete_range (layout->buffer, start, end, layout);
}

void
gtk_text_layout_typomorph_after_edit (GtkTextLayout *layout)
{
  gtk_text_layout_update_cursor_line (layout);
}
