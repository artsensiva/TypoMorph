/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Fixed synthetic view-attachment refusal test on a private Broadway display.
 */
#include <gtk/gtk.h>

static void
count_change (GtkTextBuffer *buffer, guint *count)
{
  (void) buffer;
  (*count)++;
}

int
main (void)
{
  GtkTextBuffer *buffer;
  GtkWidget *view;
  GtkTextIter start, end;
  GdkRectangle location;
  char *text;
  guint count = 0, revision;
  const char *runtime = g_getenv ("XDG_RUNTIME_DIR");
  if (runtime == NULL || !g_str_has_prefix (runtime, "/tmp/typomorph-view-") ||
      g_strcmp0 (g_getenv ("GDK_BACKEND"), "broadway") != 0 ||
      g_strcmp0 (g_getenv ("BROADWAY_DISPLAY"), ":91") != 0 ||
      g_getenv ("DISPLAY") != NULL || g_getenv ("WAYLAND_DISPLAY") != NULL)
    return 1;
  if (!gtk_init_check ())
    return 1;
  buffer = gtk_text_buffer_new (NULL);
  gtk_text_buffer_set_text (buffer, "ghbdtn", 6);
  view = g_object_ref_sink (gtk_text_view_new_with_buffer (buffer));
  gtk_text_buffer_get_start_iter (buffer, &start);
  gtk_text_view_get_iter_location (GTK_TEXT_VIEW (view), &start, &location);
  revision = gtk_text_buffer_typomorph_revision (buffer);
  g_signal_connect (buffer, "changed", G_CALLBACK (count_change), &count);
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 0, "ghbdtn", 6, "привет", 12));
  g_assert_cmpuint (count, ==, 0);
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  gtk_text_buffer_get_bounds (buffer, &start, &end);
  g_assert_cmpint (gtk_text_buffer_get_char_count (buffer), ==, 6);
  text = gtk_text_buffer_get_text (buffer, &start, &end, TRUE);
  g_assert_true (g_str_equal (text, "ghbdtn"));
  g_free (text);
  g_object_unref (view);
  g_assert_true (gtk_text_buffer_typomorph_splice (buffer, revision, 0, "ghbdtn", 6, "привет", 12));
  g_assert_cmpuint (count, ==, 1);
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  g_assert_cmpuint (token, !=, 0);
  view = g_object_ref_sink (gtk_text_view_new_with_buffer (buffer));
  gtk_text_buffer_get_start_iter (buffer, &start);
  gtk_text_view_get_iter_location (GTK_TEXT_VIEW (view), &start, &location);
  revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_false (gtk_text_buffer_typomorph_revert (buffer, token));
  g_assert_cmpuint (count, ==, 1);
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  g_object_unref (view);
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, token));
  g_assert_cmpuint (count, ==, 2);
  gtk_text_buffer_get_bounds (buffer, &start, &end);
  text = gtk_text_buffer_get_text (buffer, &start, &end, TRUE);
  g_assert_true (g_str_equal (text, "ghbdtn"));
  g_free (text);
  g_object_unref (buffer);
  g_print ("view-probe: ATTACHED_VIEW_SPLICE_AND_REVERT_REFUSED_WITHOUT_EDIT\n");
  return 0;
}
