/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Experimental post-commit AT-SPI payload publication, not a mutation API.
 * Never reread widget contents to reconstruct text that has already been removed.
 */
gboolean
gtk_at_spi_context_typomorph_ready (GtkAtSpiContext *self)
{
  return GTK_IS_AT_SPI_CONTEXT (self) &&
         gtk_at_context_is_realized (GTK_AT_CONTEXT (self)) &&
         self->connection != NULL &&
         !g_dbus_connection_is_closed (self->connection) &&
         self->context_path != NULL && self->n_registered_objects > 0 &&
         GTK_IS_ACCESSIBLE_TEXT (gtk_at_context_get_accessible (GTK_AT_CONTEXT (self)));
}

static gboolean
typomorph_a11y_word (const char *text, int bytes)
{
  if (text == NULL || bytes < 1 || bytes > 128 ||
      !g_utf8_validate (text, bytes, NULL) || g_utf8_strlen (text, bytes) > 32)
    return FALSE;
  for (const char *p = text; p < text + bytes; p = g_utf8_next_char (p))
    if (!g_unichar_isalpha (g_utf8_get_char (p)))
      return FALSE;
  return TRUE;
}

gboolean
gtk_at_spi_context_typomorph_publish (GtkAtSpiContext *self, guint offset,
                                    const char *before, int before_bytes,
                                    const char *after, int after_bytes)
{
  if (!gtk_at_spi_context_typomorph_ready (self) ||
      !typomorph_a11y_word (before, before_bytes) ||
      !typomorph_a11y_word (after, after_bytes))
    return FALSE;
  int characters = g_utf8_strlen (before, before_bytes);
  if (g_utf8_strlen (after, after_bytes) != characters ||
      offset > G_MAXINT - characters)
    return FALSE;
  if (!gtk_at_spi_root_has_event_listeners (self->root))
    return TRUE;

  /* Own both payloads before queuing either event. No widget lookup or main-loop
   * dispatch occurs between them. D-Bus delivery itself is not transactional;
   * failure must never cause the caller to roll back committed application text. */
  g_autofree char *removed = g_strndup (before, before_bytes);
  g_autofree char *inserted = g_strndup (after, after_bytes);
  g_autoptr(GVariant) deletion = g_variant_ref_sink (g_variant_new ("(siiva{sv})",
    "delete", (int) offset, characters, g_variant_new_string (removed), NULL));
  g_autoptr(GVariant) insertion = g_variant_ref_sink (g_variant_new ("(siiva{sv})",
    "insert", (int) offset, characters, g_variant_new_string (inserted), NULL));
  if (!g_dbus_connection_emit_signal (self->connection, NULL, self->context_path,
        "org.a11y.atspi.Event.Object", "TextChanged", deletion, NULL))
    return FALSE;
  return g_dbus_connection_emit_signal (self->connection, NULL, self->context_path,
    "org.a11y.atspi.Event.Object", "TextChanged", insertion, NULL);
}
