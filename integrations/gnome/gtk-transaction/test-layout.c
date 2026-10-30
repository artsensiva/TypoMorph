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

typedef struct {
  GtkTextBuffer *buffer;
  GtkTextLayout *first, *second;
  const char *expected;
  guint calls;
} Observation;

static void
observe_commit (GtkTextLayout *layout, Observation *state)
{
  (void) layout;
  state->calls++;
  assert_display (state->first, 0, state->expected);
  assert_display (state->second, 0, state->expected);
  /* All layouts must be refreshed before any callback; nested transactions refuse. */
  g_assert_false (gtk_text_buffer_typomorph_splice (state->buffer,
    gtk_text_buffer_typomorph_revision (state->buffer), 0,
    state->expected, strlen (state->expected), state->expected, strlen (state->expected)));
}

static void
layout_roundtrip (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  Observation state = { buffer, layout_new (buffer), layout_new (buffer), "привет", 0 };
  assert_display (state.first, 0, "ghbdtn");
  assert_display (state.second, 0, "ghbdtn");
  g_signal_connect (state.first, "invalidated", G_CALLBACK (observe_commit), &state);
  g_signal_connect (state.second, "invalidated", G_CALLBACK (observe_commit), &state);
  g_assert_true (replace (buffer, "привет"));
  g_assert_cmpuint (state.calls, ==, 2);
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  state.expected = "ghbdtn";
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, token));
  state.expected = "привет";
  gtk_text_buffer_undo (buffer);
  state.expected = "ghbdtn";
  gtk_text_buffer_redo (buffer);
  g_assert_cmpuint (state.calls, ==, 8);
  g_signal_handlers_disconnect_by_data (state.first, &state);
  g_signal_handlers_disconnect_by_data (state.second, &state);
  g_object_unref (state.first);
  g_object_unref (state.second);
}

static void
edit_and_detach (GtkTextLayout *layout, Observation *state)
{
  if (state->calls++)
    return;
  assert_display (layout, 0, "привет");
  assert_display (state->second, 0, "привет");
  g_assert_true (gtk_text_buffer_get_can_undo (state->buffer));
  gtk_text_layout_set_buffer (state->second, NULL);
  g_clear_object (&state->second);
  GtkTextIter end;
  gtk_text_buffer_get_end_iter (state->buffer, &end);
  gtk_text_buffer_insert (state->buffer, &end, "x", 1);
}

static void
layout_callback_edit_detach (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  Observation state = { buffer, NULL, layout_new (buffer), NULL, 0 };
  /* B-tree prepends layouts; this is notified before the one it detaches. */
  state.first = layout_new (buffer);
  assert_display (state.first, 0, "ghbdtn");
  assert_display (state.second, 0, "ghbdtn");
  gulong handler = g_signal_connect (state.first, "invalidated", G_CALLBACK (edit_and_detach), &state);
  g_assert_true (replace (buffer, "привет"));
  g_assert_null (state.second);
  g_signal_handler_disconnect (state.first, handler);
  assert_display (state.first, 0, "приветx");
  gtk_text_buffer_undo (buffer);
  assert_display (state.first, 0, "привет");
  gtk_text_buffer_undo (buffer);
  assert_display (state.first, 0, "ghbdtn");
  g_object_unref (state.first);
}

static void
layout_bidi_neighbours (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn\n \n");
  GtkTextLayout *layout = layout_new (buffer);
  assert_display (layout, 0, "ghbdtn");
  assert_display (layout, 1, " ");
  g_assert_true (replace (buffer, "אבגדהו"));
  assert_display (layout, 0, "אבגדהו");
  GtkTextIter iter;
  gtk_text_buffer_get_iter_at_line (buffer, &iter, 1);
  GtkTextLineDisplay *display = gtk_text_layout_get_line_display (
    layout, _gtk_text_iter_get_text_line (&iter), FALSE);
  g_assert_cmpint (display->direction, ==, GTK_TEXT_DIR_RTL);
  gtk_text_line_display_unref (display);
  gtk_text_buffer_undo (buffer);
  assert_display (layout, 0, "ghbdtn");
  gtk_text_buffer_get_iter_at_line (buffer, &iter, 1);
  display = gtk_text_layout_get_line_display (layout, _gtk_text_iter_get_text_line (&iter), FALSE);
  g_assert_cmpint (display->direction, ==, GTK_TEXT_DIR_LTR);
  gtk_text_line_display_unref (display);
  g_object_unref (layout);
}

static void
layout_refusals (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  GtkTextLayout *layouts[5];
  for (int i = 0; i < 5; i++)
    layouts[i] = layout_new (buffer);
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_false (replace (buffer, "привет"));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  g_object_unref (layouts[4]);
  PangoAttrList *attrs = pango_attr_list_new ();
  gtk_text_layout_set_preedit_string (layouts[0], "x", attrs, 1);
  g_assert_false (replace (buffer, "привет"));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  gtk_text_layout_set_preedit_string (layouts[0], "", attrs, 0);
  pango_attr_list_unref (attrs);
  g_assert_true (replace (buffer, "привет"));
  for (int i = 0; i < 4; i++)
    {
      assert_display (layouts[i], 0, "привет");
      g_object_unref (layouts[i]);
    }

  GString *lines = g_string_new ("ghbdtn");
  for (int i = 1; i < 256; i++)
    g_string_append (lines, "\nx");
  gtk_text_buffer_set_text (buffer, lines->str, lines->len);
  g_string_free (lines, TRUE);
  GtkTextLayout *layout = layout_new (buffer);
  g_assert_cmpint (gtk_text_buffer_get_line_count (buffer), ==, 256);
  assert_display (layout, 0, "ghbdtn");
  g_assert_true (replace (buffer, "привет"));
  gtk_text_buffer_undo (buffer);
  GtkTextIter end;
  gtk_text_buffer_get_end_iter (buffer, &end);
  gtk_text_buffer_insert (buffer, &end, "\nx", 2);
  g_assert_cmpint (gtk_text_buffer_get_line_count (buffer), ==, 257);
  revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_false (replace (buffer, "привет"));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  assert_display (layout, 0, "ghbdtn");
  g_object_unref (layout);
}

static void
veto_insert (GtkTextBuffer *buffer, GtkTextIter *iter, char *text, int len, gpointer data)
{
  (void) iter; (void) text; (void) len; (void) data;
  g_signal_stop_emission_by_name (buffer, "insert-text");
}

static void
layout_external_observer_refuses (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  GtkTextLayout *layout = layout_new (buffer);
  assert_display (layout, 0, "ghbdtn");
  gulong handler = g_signal_connect (buffer, "insert-text", G_CALLBACK (veto_insert), NULL);
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_false (replace (buffer, "привет"));
  g_signal_handler_block (buffer, handler);
  g_assert_false (replace (buffer, "привет"));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  assert_display (layout, 0, "ghbdtn");
  g_signal_handler_disconnect (buffer, handler);
  g_assert_true (replace (buffer, "привет"));
  assert_display (layout, 0, "привет");
  g_object_unref (layout);
}

static void
layout_ordinary_multiline_edits (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn\nlast");
  GtkTextLayout *layout = layout_new (buffer);
  GtkTextIter a, b;
  assert_display (layout, 0, "ghbdtn");
  assert_display (layout, 1, "last");
  gtk_text_buffer_get_iter_at_offset (buffer, &a, 3);
  gtk_text_buffer_insert (buffer, &a, "\nnew\n", -1);
  assert_display (layout, 0, "ghb");
  assert_display (layout, 1, "new");
  assert_display (layout, 2, "dtn");
  gtk_text_buffer_get_iter_at_offset (buffer, &a, 3);
  gtk_text_buffer_get_iter_at_offset (buffer, &b, 8);
  gtk_text_buffer_delete (buffer, &a, &b);
  assert_display (layout, 0, "ghbdtn");
  assert_display (layout, 1, "last");
  g_assert_true (replace (buffer, "привет"));
  assert_display (layout, 0, "привет");
  g_object_unref (layout);
}

int
main (int argc, char **argv)
{
  g_test_init (&argc, &argv, NULL);
  gtk_set_debug_flags (GTK_DEBUG_TEXT);
  g_test_add_func ("/layout/roundtrip-two-caches", layout_roundtrip);
  g_test_add_func ("/layout/callback-edit-detach", layout_callback_edit_detach);
  g_test_add_func ("/layout/bidi-neighbours", layout_bidi_neighbours);
  g_test_add_func ("/layout/bounds-preedit-refusal", layout_refusals);
  g_test_add_func ("/layout/external-observer-refuses", layout_external_observer_refuses);
  g_test_add_func ("/layout/ordinary-multiline-edits", layout_ordinary_multiline_edits);
  return g_test_run ();
}
