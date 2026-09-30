/* SPDX-License-Identifier: GPL-3.0-or-later
 * Fixed-input GtkSourceView API experiment, not an application adapter.
 * Creates only an in-memory test buffer; no display, keyboard, clipboard or
 * user-document access is requested.
 * Exit 0: normal observation; 1: unexpected result; 2: known unsafe observation.
 */
#include <gtksourceview/gtksource.h>
#include <string.h>

typedef struct
{
  guint inserts, deletes;
  gboolean fired, saw_empty;
} Observation;

static void
committed (GtkTextBuffer *buffer, GtkTextBufferNotifyFlags flags,
           guint position, guint length, gpointer data)
{
  Observation *observation = data;
  (void) buffer; (void) position; (void) length;
  /* Notification is read-only; it does not isolate the combined operation. */
  if (flags == GTK_TEXT_BUFFER_NOTIFY_AFTER_INSERT)
    observation->inserts++;
  if (flags == GTK_TEXT_BUFFER_NOTIFY_AFTER_DELETE)
    observation->deletes++;
}

static gboolean
matches (GtkTextBuffer *buffer, const char *expected)
{
  GtkTextIter start, end;
  g_autofree char *actual = NULL;
  if (gtk_text_buffer_get_char_count (buffer) > 12)
    return FALSE;
  gtk_text_buffer_get_bounds (buffer, &start, &end);
  actual = gtk_text_buffer_get_text (buffer, &start, &end, TRUE);
  return g_str_equal (actual, expected);
}

static void
veto_insert (GtkTextBuffer *buffer, GtkTextIter *position,
             char *text, int length, Observation *observation)
{
  (void) position; (void) text; (void) length;
  observation->fired = TRUE;
  g_signal_stop_emission_by_name (buffer, "insert-text");
}

static void
veto_delete (GtkTextBuffer *buffer, GtkTextIter *start,
             GtkTextIter *end, Observation *observation)
{
  (void) start; (void) end;
  observation->fired = TRUE;
  g_signal_stop_emission_by_name (buffer, "delete-range");
}

static void
append_after_delete (GtkTextBuffer *buffer, GtkTextIter *start,
                     GtkTextIter *end, Observation *observation)
{
  GtkTextIter position;
  int offset = gtk_text_iter_get_offset (start);
  observation->fired = TRUE;
  observation->saw_empty = gtk_text_buffer_get_char_count (buffer) == 0;
  gtk_text_buffer_get_end_iter (buffer, &position);
  gtk_text_buffer_insert (buffer, &position, "x", 1);
  /* Respect GTK's signal contract: revalidate iterators after our mutation. */
  gtk_text_buffer_get_iter_at_offset (buffer, start, offset);
  gtk_text_buffer_get_iter_at_offset (buffer, end, offset);
}

int
main (int argc, char **argv)
{
  g_autoptr(GtkSourceBuffer) source = NULL;
  g_autoptr(GtkSourceSearchSettings) settings = NULL;
  g_autoptr(GtkSourceSearchContext) search = NULL;
  g_autoptr(GError) error = NULL;
  GtkTextBuffer *buffer;
  GtkTextIter start, end;
  Observation observation = { 0 };
  gulong handler = 0;
  guint notifier;
  gboolean replaced, expected = FALSE;
  const char *mode;

  if (argc != 2)
    return 1;
  mode = argv[1];
  if (!g_str_equal (mode, "normal") && !g_str_equal (mode, "veto-insert") &&
      !g_str_equal (mode, "veto-delete") && !g_str_equal (mode, "callback-delete-edit"))
    return 1;

  source = gtk_source_buffer_new (NULL);
  buffer = GTK_TEXT_BUFFER (source);
  gtk_text_buffer_set_enable_undo (buffer, TRUE);
  gtk_text_buffer_set_text (buffer, "ghbdtn", -1);
  settings = gtk_source_search_settings_new ();
  gtk_source_search_settings_set_search_text (settings, "ghbdtn");
  gtk_source_search_settings_set_case_sensitive (settings, TRUE);
  search = gtk_source_search_context_new (source, settings);
  gtk_source_search_context_set_highlight (search, FALSE);
  notifier = gtk_text_buffer_add_commit_notify (
    buffer, GTK_TEXT_BUFFER_NOTIFY_AFTER_INSERT | GTK_TEXT_BUFFER_NOTIFY_AFTER_DELETE,
    committed, &observation, NULL);

  if (g_str_equal (mode, "veto-insert"))
    handler = g_signal_connect (buffer, "insert-text", G_CALLBACK (veto_insert), &observation);
  else if (g_str_equal (mode, "veto-delete"))
    handler = g_signal_connect (buffer, "delete-range", G_CALLBACK (veto_delete), &observation);
  else if (g_str_equal (mode, "callback-delete-edit"))
    handler = g_signal_connect_after (buffer, "delete-range", G_CALLBACK (append_after_delete), &observation);

  gtk_text_buffer_get_bounds (buffer, &start, &end);
  replaced = gtk_source_search_context_replace (search, &start, &end, "привет", -1, &error);
  if (handler != 0)
    g_signal_handler_disconnect (buffer, handler);
  gtk_text_buffer_remove_commit_notify (buffer, notifier);

  if (replaced && error == NULL)
    {
      if (g_str_equal (mode, "normal"))
        {
          expected = matches (buffer, "привет") && observation.inserts == 1 &&
                     observation.deletes == 1 && gtk_text_buffer_get_can_undo (buffer);
          if (expected)
            {
              gtk_text_buffer_undo (buffer);
              expected = matches (buffer, "ghbdtn") && gtk_text_buffer_get_can_redo (buffer);
              if (expected)
                {
                  gtk_text_buffer_redo (buffer);
                  expected = matches (buffer, "привет");
                }
            }
        }
      else if (g_str_equal (mode, "veto-insert"))
        expected = observation.fired && observation.inserts == 0 &&
                   observation.deletes == 1 && matches (buffer, "");
      else if (g_str_equal (mode, "veto-delete"))
        expected = observation.fired && observation.inserts == 1 &&
                   observation.deletes == 0 && matches (buffer, "ghbdtnпривет");
      else
        expected = observation.fired && observation.saw_empty && observation.inserts == 2 &&
                   observation.deletes == 1 && matches (buffer, "приветx");
    }

  /* Never print buffer contents, callback payloads or arbitrary error messages. */
  if (!expected)
    {
      g_print ("search-replace-probe: UNEXPECTED_RESULT\n");
      return 1;
    }
  if (g_str_equal (mode, "normal"))
    {
      g_print ("search-replace-probe: NORMAL_AND_ORDINARY_UNDO_REDO_OBSERVED\n");
      return 0;
    }
  g_print ("search-replace-probe: KNOWN_NONATOMIC_RESULT_OBSERVED\n");
  return 2;
}
