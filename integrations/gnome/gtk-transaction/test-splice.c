/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Synthetic tests against the separately built, actually patched GTK library.
 * No application, display or real input is used.
 */
#include <gtk/gtk.h>
#include <string.h>

static GtkTextBuffer *
fixture (const char *text)
{
  GtkTextBuffer *buffer = gtk_text_buffer_new (NULL);
  gtk_text_buffer_set_enable_undo (buffer, FALSE);
  gtk_text_buffer_set_text (buffer, text, -1);
  return buffer;
}

static gboolean
has_text (GtkTextBuffer *buffer, const char *expected)
{
  GtkTextIter start, end;
  g_autofree char *actual = NULL;
  if (gtk_text_buffer_get_char_count (buffer) > 512)
    return FALSE;
  gtk_text_buffer_get_bounds (buffer, &start, &end);
  actual = gtk_text_buffer_get_text (buffer, &start, &end, TRUE);
  return g_str_equal (actual, expected);
}

static gboolean
replace (GtkTextBuffer *buffer, int offset)
{
  return gtk_text_buffer_typomorph_splice (
    buffer, gtk_text_buffer_typomorph_revision (buffer), offset,
    "ghbdtn", 6, "привет", 12);
}

static void
normal (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("before ghbdtn after\nsecond line");
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_true (replace (buffer, 7));
  g_assert_true (has_text (buffer, "before привет after\nsecond line"));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), !=, revision);
}

static void
unicode_byte_lengths (void)
{
  const char *words[] = { "привет", "אבגדהו", "𐐀𐐁𐐂𐐃𐐄𐐅" };
  for (guint i = 0; i < G_N_ELEMENTS (words); i++)
    {
      g_autoptr(GtkTextBuffer) buffer = fixture ("é ghbdtn suffix");
      g_autofree char *expected = g_strconcat ("é ", words[i], " suffix", NULL);
      g_assert_true (gtk_text_buffer_typomorph_splice (
        buffer, gtk_text_buffer_typomorph_revision (buffer), 2,
        "ghbdtn", 6, words[i], strlen (words[i])));
      g_assert_true (has_text (buffer, expected));
      /* A second explicit transaction, NOT an undo implementation. */
      g_assert_true (gtk_text_buffer_typomorph_splice (
        buffer, gtk_text_buffer_typomorph_revision (buffer), 2,
        words[i], strlen (words[i]), "ghbdtn", 6));
      g_assert_true (has_text (buffer, "é ghbdtn suffix"));
    }
}

static void
stale_and_invalid (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  GtkTextIter end;
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  gtk_text_buffer_get_end_iter (buffer, &end);
  gtk_text_buffer_insert (buffer, &end, "x", 1);
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 0, "ghbdtn", 6, "привет", 12));
  revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 0, "foobar", 6, "привет", 12));
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 0, "ghbdtn", 6, "bad", 3));
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 0, "ghbdtn", 6, "\xff", 1));
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 0, "ghbdtn", 6, "hi\0bye", 6));
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 0, "ghbdtn", 6, "hello\n", 6));
  g_assert_false (replace (buffer, -1));
  g_assert_false (replace (buffer, G_MAXINT));
  g_assert_true (has_text (buffer, "ghbdtnx"));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
}

static void
marks_and_lengths (void)
{
  for (int length = 1; length <= 32; length++)
    {
      char original[33];
      GString *replacement = g_string_new (NULL);
      GString *expected = g_string_new ("left ");
      GtkTextMark *marks[66];
      GtkTextIter position;
      memset (original, 'a', length);
      original[length] = 0;
      for (int i = 0; i < length; i++)
        g_string_append (replacement, "я");
      g_string_append (expected, original);
      g_string_append (expected, " right");
      g_autoptr(GtkTextBuffer) buffer = fixture (expected->str);
      for (int i = 0; i <= length; i++)
        {
          gtk_text_buffer_get_iter_at_offset (buffer, &position, 5 + i);
          marks[2*i] = gtk_text_buffer_create_mark (buffer, NULL, &position, TRUE);
          marks[2*i+1] = gtk_text_buffer_create_mark (buffer, NULL, &position, FALSE);
        }
      gtk_text_buffer_get_iter_at_offset (buffer, &position, 5 + length);
      gtk_text_buffer_place_cursor (buffer, &position);
      g_assert_true (gtk_text_buffer_typomorph_splice (
        buffer, gtk_text_buffer_typomorph_revision (buffer), 5,
        original, length, replacement->str, replacement->len));
      g_string_assign (expected, "left ");
      g_string_append (expected, replacement->str);
      g_string_append (expected, " right");
      g_assert_true (has_text (buffer, expected->str));
      for (int i = 0; i <= length; i++)
        for (int gravity = 0; gravity <= 1; gravity++)
          {
            gtk_text_buffer_get_iter_at_mark (buffer, &position, marks[2*i+gravity]);
            g_assert_cmpint (gtk_text_iter_get_offset (&position), ==, 5 + i);
          }
      gtk_text_buffer_get_iter_at_mark (buffer, &position, gtk_text_buffer_get_insert (buffer));
      g_assert_cmpint (gtk_text_iter_get_offset (&position), ==, 5 + length);
      g_string_free (replacement, TRUE);
      g_string_free (expected, TRUE);
    }
}

typedef struct { gboolean fired; guint notifications; } CallbackState;

static void
append_from_changed (GtkTextBuffer *buffer, CallbackState *state)
{
  GtkTextIter end;
  state->notifications++;
  if (state->fired)
    return;
  state->fired = TRUE;
  g_assert_true (has_text (buffer, "привет"));
  g_assert_false (gtk_text_buffer_typomorph_splice (
    buffer, gtk_text_buffer_typomorph_revision (buffer), 0, "привет", 12, "ghbdtn", 6));
  gtk_text_buffer_get_end_iter (buffer, &end);
  gtk_text_buffer_insert (buffer, &end, "x", 1);
}

static void
callback_edit (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  CallbackState state = { 0 };
  g_signal_connect (buffer, "changed", G_CALLBACK (append_from_changed), &state);
  g_assert_true (replace (buffer, 0));
  g_assert_true (state.fired);
  g_assert_cmpuint (state.notifications, ==, 2);
  g_assert_true (has_text (buffer, "приветx"));
}

static void
delete_from_changed (GtkTextBuffer *buffer, gboolean *fired)
{
  GtkTextIter start, end;
  if (*fired)
    return;
  *fired = TRUE;
  g_assert_true (has_text (buffer, "привет"));
  gtk_text_buffer_get_iter_at_offset (buffer, &start, 5);
  gtk_text_buffer_get_end_iter (buffer, &end);
  gtk_text_buffer_delete (buffer, &start, &end);
}

static void
callback_delete_preserved (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  gboolean fired = FALSE;
  g_signal_connect (buffer, "changed", G_CALLBACK (delete_from_changed), &fired);
  g_assert_true (replace (buffer, 0));
  g_assert_true (fired);
  g_assert_true (has_text (buffer, "приве"));
}

static void
append_from_notify (GObject *object, GParamSpec *spec, CallbackState *state)
{
  (void) spec;
  append_from_changed (GTK_TEXT_BUFFER (object), state);
}

static void
other_callback_paths (void)
{
  for (int notify = 0; notify < 2; notify++)
    {
      g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
      CallbackState state = { 0 };
      gtk_text_buffer_set_modified (buffer, FALSE);
      if (notify)
        g_signal_connect (buffer, "notify::text", G_CALLBACK (append_from_notify), &state);
      else
        g_signal_connect (buffer, "modified-changed", G_CALLBACK (append_from_changed), &state);
      g_assert_true (replace (buffer, 0));
      g_assert_true (state.fired);
      g_assert_true (has_text (buffer, "приветx"));
    }
}

static void
release_external_owner (GtkTextBuffer *buffer, GtkTextBuffer **owner)
{
  g_assert_true (has_text (buffer, "привет"));
  g_clear_object (owner);
}

static void
callback_lifetime (void)
{
  GtkTextBuffer *buffer = fixture ("ghbdtn");
  GWeakRef weak;
  g_weak_ref_init (&weak, buffer);
  g_signal_connect (buffer, "changed", G_CALLBACK (release_external_owner), &buffer);
  g_assert_true (replace (buffer, 0));
  g_assert_null (buffer);
  g_assert_null (g_weak_ref_get (&weak));
  g_weak_ref_clear (&weak);
}

static void
commit_observer (GtkTextBuffer *buffer, GtkTextBufferNotifyFlags flags,
                 guint position, guint length, gpointer data)
{
  (void) buffer; (void) flags; (void) position; (void) length;
  *(gboolean *) data = TRUE;
}

static void
commit_observer_refuses (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  gboolean fired = FALSE;
  guint handler = gtk_text_buffer_add_commit_notify (
    buffer, GTK_TEXT_BUFFER_NOTIFY_AFTER_INSERT, commit_observer, &fired, NULL);
  g_assert_false (replace (buffer, 0));
  g_assert_false (fired);
  g_assert_true (has_text (buffer, "ghbdtn"));
  gtk_text_buffer_remove_commit_notify (buffer, handler);
  g_assert_true (replace (buffer, 0));
}

static void
veto_insert (GtkTextBuffer *buffer, GtkTextIter *iter, char *text, int length, gpointer data)
{
  (void) iter; (void) text; (void) length; (void) data;
  g_signal_stop_emission_by_name (buffer, "insert-text");
}

static void
veto_delete (GtkTextBuffer *buffer, GtkTextIter *start, GtkTextIter *end, gpointer data)
{
  (void) start; (void) end; (void) data;
  g_signal_stop_emission_by_name (buffer, "delete-range");
}

static void
legacy_handlers_refuse (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  gulong handler = g_signal_connect (buffer, "insert-text", G_CALLBACK (veto_insert), NULL);
  g_assert_false (replace (buffer, 0));
  g_signal_handler_block (buffer, handler);
  g_assert_false (replace (buffer, 0));
  g_signal_handler_disconnect (buffer, handler);
  handler = g_signal_connect (buffer, "delete-range", G_CALLBACK (veto_delete), NULL);
  g_assert_false (replace (buffer, 0));
  g_signal_handler_disconnect (buffer, handler);
  g_assert_true (has_text (buffer, "ghbdtn"));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
}

static void
unsupported_refuse (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn");
  gtk_text_buffer_set_enable_undo (buffer, TRUE);
  gtk_text_buffer_begin_irreversible_action (buffer);
  g_assert_false (replace (buffer, 0));
  gtk_text_buffer_end_irreversible_action (buffer);
  gtk_text_buffer_set_enable_undo (buffer, FALSE);
  gtk_text_buffer_begin_user_action (buffer);
  g_assert_false (replace (buffer, 0));
  gtk_text_buffer_end_user_action (buffer);
  gtk_text_buffer_create_tag (buffer, "ordinary-tag", NULL);
  g_assert_false (replace (buffer, 0));
  g_assert_true (has_text (buffer, "ghbdtn"));
}

static void
bounds_refuse (void)
{
  char long_line[300];
  memset (long_line, 'a', sizeof long_line - 1);
  long_line[sizeof long_line - 1] = 0;
  memcpy (long_line, "ghbdtn", 6);
  g_autoptr(GtkTextBuffer) buffer = fixture (long_line);
  g_assert_false (replace (buffer, 0));
  g_assert_true (has_text (buffer, long_line));
  gtk_text_buffer_set_text (buffer, "ghbdtn", -1);
  GtkTextIter start;
  gtk_text_buffer_get_start_iter (buffer, &start);
  for (int i = 0; i < 513; i++)
    gtk_text_buffer_create_mark (buffer, NULL, &start, TRUE);
  g_assert_false (replace (buffer, 0));
  g_assert_true (has_text (buffer, "ghbdtn"));
}


#include "test-history.inc"
#include "test-revert.inc"

int
main (int argc, char **argv)
{
  g_test_init (&argc, &argv, NULL);
  gtk_set_debug_flags (GTK_DEBUG_TEXT);
  g_test_add_func ("/splice/normal", normal);
  g_test_add_func ("/splice/unicode-byte-lengths", unicode_byte_lengths);
  g_test_add_func ("/splice/stale-invalid", stale_and_invalid);
  g_test_add_func ("/splice/marks-lengths", marks_and_lengths);
  g_test_add_func ("/splice/callback-edit-nested-refusal", callback_edit);
  g_test_add_func ("/splice/callback-delete-preserved", callback_delete_preserved);
  g_test_add_func ("/splice/modified-notify-callbacks", other_callback_paths);
  g_test_add_func ("/splice/callback-lifetime", callback_lifetime);
  g_test_add_func ("/splice/commit-observer-refuses", commit_observer_refuses);
  g_test_add_func ("/splice/legacy-handlers-refuse", legacy_handlers_refuse);
  g_test_add_func ("/splice/unsupported-refuse", unsupported_refuse);
  g_test_add_func ("/splice/bounds-refuse", bounds_refuse);
  add_history_tests ();
  add_revert_tests ();
  return g_test_run ();
}
