/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Appended inside gtktextbtree.c by the isolated prototype patch builder.
 * Same-character-count, short-line experiment. No production ABI is proposed.
 */
gboolean
_gtk_text_btree_typomorph_has_views (GtkTextBTree *tree)
{
  return tree->views != NULL;
}

gboolean
_gtk_text_btree_typomorph_splice (GtkTextIter *start,
                                GtkTextIter *end,
                                const char  *replacement)
{
  GtkTextBTree *tree = _gtk_text_iter_get_btree (start);
  GtkTextLine *line = _gtk_text_iter_get_text_line (start);
  GtkTextLineSegment *segment, **link;
  struct { GtkTextLineSegment *old_segment, *new_segment; } plan[32];
  guint count = 0, steps = 0, index = 0;
  int position = 0;
  int first = gtk_text_iter_get_line_offset (start);
  int last = gtk_text_iter_get_line_offset (end);
  int absolute_first = gtk_text_iter_get_offset (start);

  /* Views/subclasses need a new notification contract. Do not silently bypass
   * their existing delete/insert callbacks in this first storage experiment. */
  if (_gtk_text_btree_typomorph_has_views (tree) || _gtk_text_iter_get_text_line (end) != line ||
      gtk_text_iter_get_chars_in_line (start) > 256 || last <= first ||
      last - first > 32 || g_utf8_strlen (replacement, -1) != last - first)
    return FALSE;

  /* Build every replacement segment before changing a link. Character counts
   * are preserved, including offsets of zero-width mark segments inside words.
   * The short-line limit bounds copies of neighboring text in the same segment. */
  for (segment = line->segments; segment != NULL; segment = segment->next)
    {
      int finish = position + segment->char_count;
      if (++steps > 512)
        goto refuse;
      if (finish > first && position < last && segment->char_count > 0)
        {
          int left = MAX (first, position);
          int right = MIN (last, finish);
          const char *prefix_end, *suffix, *part, *part_end;
          g_autofree char *combined = NULL;
          gsize prefix_bytes, part_bytes, suffix_bytes;
          if (segment->type != &gtk_text_char_type || count == G_N_ELEMENTS (plan))
            goto refuse;
          prefix_end = g_utf8_offset_to_pointer (segment->body.chars, left - position);
          suffix = g_utf8_offset_to_pointer (segment->body.chars, right - position);
          part = g_utf8_offset_to_pointer (replacement, left - first);
          part_end = g_utf8_offset_to_pointer (replacement, right - first);
          prefix_bytes = prefix_end - segment->body.chars;
          part_bytes = part_end - part;
          suffix_bytes = segment->byte_count - (suffix - segment->body.chars);
          combined = g_malloc (prefix_bytes + part_bytes + suffix_bytes + 1);
          memcpy (combined, segment->body.chars, prefix_bytes);
          memcpy (combined + prefix_bytes, part, part_bytes);
          memcpy (combined + prefix_bytes + part_bytes, suffix, suffix_bytes);
          combined[prefix_bytes + part_bytes + suffix_bytes] = 0;
          plan[count].old_segment = segment;
          plan[count].new_segment = _gtk_char_segment_new (
            combined, prefix_bytes + part_bytes + suffix_bytes);
          count++;
        }
      position = finish;
    }
  if (count == 0 || position < last)
    goto refuse;

  /* Commit contains no signal, layout, history or user callback. No buffer or
   * B-tree is swapped. Allocation failure follows GTK's process-fatal policy. */
  for (link = &line->segments; *link != NULL && index < count; link = &(*link)->next)
    if (*link == plan[index].old_segment)
      {
        GtkTextLineSegment *old_segment = *link;
        plan[index].new_segment->next = old_segment->next;
        *link = plan[index].new_segment;
        g_free (old_segment);
        index++;
      }
  g_assert (index == count);
  chars_changed (tree);
  segments_changed (tree);
  _gtk_text_btree_get_iter_at_char (tree, start, absolute_first);
  _gtk_text_btree_get_iter_at_char (tree, end, absolute_first + last - first);
  gtk_text_btree_resolve_bidi (start, end);
  if (GTK_DEBUG_CHECK (TEXT))
    _gtk_text_btree_check (tree);
  return TRUE;

refuse:
  for (index = 0; index < count; index++)
    g_free (plan[index].new_segment);
  return FALSE;
}
