/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Fixed synthetic layout/cache tests; link against the disposable GTK archive.
 * No GtkTextView accessibility, application, display, or input integration.
 */
#include <gtk/gtk.h>
#include <pango/pangocairo.h>
#include "gtk/gtktextlayoutprivate.h"
#include "gtk/gtktextiterprivate.h"

static GtkTextBuffer *
fixture (const char *text)
{
  GtkTextBuffer *buffer = gtk_text_buffer_new (NULL);
  gtk_text_buffer_set_text (buffer, text, -1);
  gtk_text_buffer_set_modified (buffer, FALSE);
  return buffer;
}

static GtkTextLayout *
layout_new (GtkTextBuffer *buffer)
{
  GtkTextLayout *layout = gtk_text_layout_new ();
  PangoContext *ltr = pango_font_map_create_context (pango_cairo_font_map_get_default ());
  PangoContext *rtl = pango_font_map_create_context (pango_cairo_font_map_get_default ());
  GtkTextAttributes *style = gtk_text_attributes_new ();
  style->font = pango_font_description_from_string ("Sans 12");
  pango_context_set_base_dir (ltr, PANGO_DIRECTION_LTR);
  pango_context_set_base_dir (rtl, PANGO_DIRECTION_RTL);
  gtk_text_layout_set_default_style (layout, style);
  gtk_text_attributes_unref (style);
  gtk_text_layout_set_contexts (layout, ltr, rtl);
  g_object_unref (ltr);
  g_object_unref (rtl);
  gtk_text_layout_set_screen_width (layout, 500);
  gtk_text_layout_set_buffer (layout, buffer);
  return layout;
}

static void
assert_display (GtkTextLayout *layout, int line, const char *expected)
{
  GtkTextIter iter;
  gtk_text_buffer_get_iter_at_line (layout->buffer, &iter, line);
  GtkTextLineDisplay *display = gtk_text_layout_get_line_display (
    layout, _gtk_text_iter_get_text_line (&iter), FALSE);
  g_assert_cmpstr (pango_layout_get_text (display->layout), ==, expected);
  gtk_text_line_display_unref (display);
}

static gboolean
replace (GtkTextBuffer *buffer, const char *replacement)
{
  return gtk_text_buffer_typomorph_splice (buffer,
    gtk_text_buffer_typomorph_revision (buffer), 0, "ghbdtn", 6,
    replacement, strlen (replacement));
}

static void
count_invalidation (GtkTextLayout *layout, guint *count)
{
  (void) layout;
  (*count)++;
}

static void
layout_legacy_boundary (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  GtkTextLayout *first = layout_new (buffer);
  GtkTextLayout *second = layout_new (buffer);
  guint count = 0;
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  assert_display (first, 0, "ghbdtn");
  assert_display (second, 0, "ghbdtn");
  g_assert_true (g_signal_has_handler_pending (buffer,
    g_signal_lookup ("insert-text", GTK_TYPE_TEXT_BUFFER), 0, TRUE));
  g_assert_true (g_signal_has_handler_pending (buffer,
    g_signal_lookup ("delete-range", GTK_TYPE_TEXT_BUFFER), 0, TRUE));
  g_signal_connect (first, "invalidated", G_CALLBACK (count_invalidation), &count);
  g_signal_connect (second, "invalidated", G_CALLBACK (count_invalidation), &count);
  g_assert_false (replace (buffer, "привет"));
  g_assert_cmpuint (count, ==, 0);
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  assert_display (first, 0, "ghbdtn");
  assert_display (second, 0, "ghbdtn");
  g_object_unref (first);
  g_object_unref (second);
  g_assert_false (g_signal_has_handler_pending (buffer,
    g_signal_lookup ("insert-text", GTK_TYPE_TEXT_BUFFER), 0, TRUE));
  g_assert_true (replace (buffer, "привет"));
}

static void
layout_reversal_boundary (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  g_assert_true (replace (buffer, "привет"));
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  GtkTextLayout *layout = layout_new (buffer);
  assert_display (layout, 0, "привет");
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_false (gtk_text_buffer_typomorph_revert (buffer, token));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  assert_display (layout, 0, "привет");
  g_object_unref (layout);
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, token));
}

int
main (int argc, char **argv)
{
  g_test_init (&argc, &argv, NULL);
  gtk_set_debug_flags (GTK_DEBUG_TEXT);
  g_test_add_func ("/layout/legacy-observers-refuse", layout_legacy_boundary);
  g_test_add_func ("/layout/reversal-refuses", layout_reversal_boundary);
  return g_test_run ();
}
