/* SPDX-License-Identifier: LGPL-2.1-or-later */
static GtkTextBuffer *
history_fixture (void)
{
  GtkTextBuffer *buffer = fixture ("ghbdtn");
  gtk_text_buffer_set_enable_undo (buffer, TRUE);
  gtk_text_buffer_set_modified (buffer, FALSE);
  return buffer;
}

static void
history_roundtrip (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  g_assert_true (replace (buffer, 0));
  g_assert_true (gtk_text_buffer_get_can_undo (buffer));
  g_assert_true (gtk_text_buffer_get_modified (buffer));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn"));
  g_assert_false (gtk_text_buffer_get_modified (buffer));
  g_assert_true (gtk_text_buffer_get_can_redo (buffer));
  gtk_text_buffer_redo (buffer);
  g_assert_true (has_text (buffer, "привет"));
  g_assert_true (gtk_text_buffer_get_modified (buffer));
}

static void
history_saved_correction (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  g_assert_true (replace (buffer, 0));
  gtk_text_buffer_set_modified (buffer, FALSE);
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn"));
  g_assert_true (gtk_text_buffer_get_modified (buffer));
  gtk_text_buffer_redo (buffer);
  g_assert_true (has_text (buffer, "привет"));
  g_assert_false (gtk_text_buffer_get_modified (buffer));
}

static void
history_later_typing (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  GtkTextIter end;
  g_assert_true (replace (buffer, 0));
  gtk_text_buffer_begin_user_action (buffer);
  gtk_text_buffer_get_end_iter (buffer, &end);
  gtk_text_buffer_insert (buffer, &end, "x", 1);
  gtk_text_buffer_end_user_action (buffer);
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "привет"));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn"));
  gtk_text_buffer_redo (buffer);
  g_assert_true (has_text (buffer, "привет"));
  gtk_text_buffer_redo (buffer);
  g_assert_true (has_text (buffer, "приветx"));
}

static void
history_earlier_typing (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("");
  GtkTextIter start;
  gtk_text_buffer_set_enable_undo (buffer, TRUE);
  gtk_text_buffer_get_start_iter (buffer, &start);
  gtk_text_buffer_insert (buffer, &start, "ghbdtn", 6);
  g_assert_true (replace (buffer, 0));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn"));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, ""));
  gtk_text_buffer_redo (buffer);
  gtk_text_buffer_redo (buffer);
  g_assert_true (has_text (buffer, "привет"));
}

static void
history_refusal_keeps_entry (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  g_assert_true (replace (buffer, 0));
  gulong handler = g_signal_connect (buffer, "insert-text", G_CALLBACK (veto_insert), NULL);
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "привет"));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  g_assert_true (gtk_text_buffer_get_can_undo (buffer));
  g_assert_false (gtk_text_buffer_get_can_redo (buffer));
  g_signal_handler_disconnect (buffer, handler);
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn"));
  handler = g_signal_connect (buffer, "delete-range", G_CALLBACK (veto_delete), NULL);
  gtk_text_buffer_redo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn"));
  g_assert_true (gtk_text_buffer_get_can_redo (buffer));
  g_signal_handler_disconnect (buffer, handler);
  gtk_text_buffer_redo (buffer);
  g_assert_true (has_text (buffer, "привет"));
}

typedef struct { gboolean fired; const char *expected; } HistoryCallback;
static void
edit_during_history (GtkTextBuffer *buffer, HistoryCallback *state)
{
  GtkTextIter end;
  if (state->fired)
    return;
  state->fired = TRUE;
  g_assert_true (has_text (buffer, state->expected));
  gtk_text_buffer_undo (buffer); /* Nested history request must refuse. */
  g_assert_true (has_text (buffer, state->expected));
  gtk_text_buffer_get_end_iter (buffer, &end);
  gtk_text_buffer_insert (buffer, &end, "x", 1);
}

static void
history_callback_edit (void)
{
  for (int redo = 0; redo <= 1; redo++)
    {
      g_autoptr(GtkTextBuffer) buffer = history_fixture ();
      HistoryCallback state = { FALSE, redo ? "привет" : "ghbdtn" };
      g_assert_true (replace (buffer, 0));
      if (redo)
        gtk_text_buffer_undo (buffer);
      gulong handler = g_signal_connect (buffer, "changed", G_CALLBACK (edit_during_history), &state);
      if (redo)
        gtk_text_buffer_redo (buffer);
      else
        gtk_text_buffer_undo (buffer);
      g_signal_handler_disconnect (buffer, handler);
      g_assert_true (state.fired);
      g_assert_true (has_text (buffer, redo ? "приветx" : "ghbdtnx"));
      g_assert_true (gtk_text_buffer_get_modified (buffer));
      g_assert_false (gtk_text_buffer_get_can_redo (buffer));
      gtk_text_buffer_undo (buffer);
      g_assert_true (has_text (buffer, state.expected));
    }
}

static void
edit_from_history_notify (GObject *object, GParamSpec *spec, HistoryCallback *state)
{
  (void) spec;
  edit_during_history (GTK_TEXT_BUFFER (object), state);
}

static void
history_notify_edit (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  HistoryCallback state = { FALSE, "ghbdtn" };
  g_assert_true (replace (buffer, 0));
  gulong handler = g_signal_connect (buffer, "notify::can-redo", G_CALLBACK (edit_from_history_notify), &state);
  gtk_text_buffer_undo (buffer);
  g_signal_handler_disconnect (buffer, handler);
  g_assert_true (state.fired);
  g_assert_true (has_text (buffer, "ghbdtnx"));
  g_assert_true (gtk_text_buffer_get_modified (buffer));
  g_assert_false (gtk_text_buffer_get_can_redo (buffer));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn"));
}

static void
add_history_tests (void)
{
  g_test_add_func ("/history/roundtrip", history_roundtrip);
  g_test_add_func ("/history/savepoint", history_saved_correction);
  g_test_add_func ("/history/later-typing", history_later_typing);
  g_test_add_func ("/history/earlier-typing", history_earlier_typing);
  g_test_add_func ("/history/refusal-keeps-entry", history_refusal_keeps_entry);
  g_test_add_func ("/history/callback-edits-retained-and-undoable", history_callback_edit);
  g_test_add_func ("/history/availability-notify-edit", history_notify_edit);
}
