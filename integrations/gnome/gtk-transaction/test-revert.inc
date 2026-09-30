/* SPDX-License-Identifier: LGPL-2.1-or-later */
static void
insert_at (GtkTextBuffer *buffer, int offset, const char *text)
{
  GtkTextIter iter;
  gtk_text_buffer_get_iter_at_offset (buffer, &iter, offset);
  gtk_text_buffer_insert (buffer, &iter, text, -1);
}

static void
delete_at (GtkTextBuffer *buffer, int start, int end)
{
  GtkTextIter a, b;
  gtk_text_buffer_get_iter_at_offset (buffer, &a, start);
  gtk_text_buffer_get_iter_at_offset (buffer, &b, end);
  gtk_text_buffer_delete (buffer, &a, &b);
}

static void
revert_later_typing (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  g_assert_true (replace (buffer, 0));
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  g_assert_cmpuint (token, !=, 0);
  insert_at (buffer, 6, " x");
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, token));
  g_assert_true (has_text (buffer, "ghbdtn x"));
  g_assert_cmpuint (gtk_text_buffer_typomorph_correction_token (buffer), ==, 0);
  g_assert_false (gtk_text_buffer_typomorph_revert (buffer, token));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "привет x"));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "привет"));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn"));
  gtk_text_buffer_redo (buffer);
  gtk_text_buffer_redo (buffer);
  gtk_text_buffer_redo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn x"));
}

static void
revert_disjoint_edits (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ab ghbdtn cd");
  g_assert_true (replace (buffer, 3));
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  insert_at (buffer, 3, "é𐐀"); /* Exact start: correction moves past insertion. */
  insert_at (buffer, 11, "x"); /* Exact end: insertion stays outside correction. */
  delete_at (buffer, 0, 2);
  delete_at (buffer, 10, 13);
  g_assert_true (has_text (buffer, " é𐐀приветx"));
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, token));
  g_assert_true (has_text (buffer, " é𐐀ghbdtnx"));
}

static void
revert_overlap_refuses (void)
{
  for (int offset = 0; offset < 6; offset++)
    {
      g_autoptr(GtkTextBuffer) buffer = history_fixture ();
      g_assert_true (replace (buffer, 0));
      guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
      delete_at (buffer, offset, offset + 1);
      /* Even undo restoring identical text cannot resurrect authorization. */
      gtk_text_buffer_undo (buffer);
      g_assert_true (has_text (buffer, "привет"));
      guint revision = gtk_text_buffer_typomorph_revision (buffer);
      g_assert_false (gtk_text_buffer_typomorph_revert (buffer, token));
      g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
    }
  for (int offset = 1; offset < 6; offset++)
    {
      g_autoptr(GtkTextBuffer) buffer = history_fixture ();
      g_assert_true (replace (buffer, 0));
      guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
      insert_at (buffer, offset, "x");
      delete_at (buffer, offset, offset + 1);
      g_assert_true (has_text (buffer, "привет"));
      g_assert_false (gtk_text_buffer_typomorph_revert (buffer, token));
    }
}

static void
revert_token_scope (void)
{
  g_autoptr(GtkTextBuffer) a = history_fixture ();
  g_autoptr(GtkTextBuffer) b = history_fixture ();
  g_assert_true (replace (a, 0));
  g_assert_true (replace (b, 0));
  guint64 token_a = gtk_text_buffer_typomorph_correction_token (a);
  guint64 token_b = gtk_text_buffer_typomorph_correction_token (b);
  g_assert_cmpuint (token_a, !=, token_b);
  g_assert_false (gtk_text_buffer_typomorph_revert (b, token_a));
  gtk_text_buffer_typomorph_forget (a); /* Owner/focus cancellation hook. */
  g_assert_false (gtk_text_buffer_typomorph_revert (a, token_a));
  gtk_text_buffer_undo (b);
  gtk_text_buffer_redo (b);
  g_assert_false (gtk_text_buffer_typomorph_revert (b, token_b));
}

static void
revert_refusal_retry (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  g_assert_true (replace (buffer, 0));
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  gulong handler = g_signal_connect (buffer, "insert-text", G_CALLBACK (veto_insert), NULL);
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_false (gtk_text_buffer_typomorph_revert (buffer, token));
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  g_assert_cmpuint (gtk_text_buffer_typomorph_correction_token (buffer), ==, token);
  g_signal_handler_disconnect (buffer, handler);
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, token));
  g_assert_true (has_text (buffer, "ghbdtn"));
}

static void
revert_callback_edit (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  HistoryCallback state = { FALSE, "ghbdtn" };
  g_assert_true (replace (buffer, 0));
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  gulong handler = g_signal_connect (buffer, "changed", G_CALLBACK (edit_during_history), &state);
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, token));
  g_signal_handler_disconnect (buffer, handler);
  g_assert_true (state.fired);
  g_assert_true (has_text (buffer, "ghbdtnx"));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "ghbdtn"));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "привет"));
}

static void
revert_bound_and_unknown_mutation (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  g_assert_true (replace (buffer, 0));
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  for (int i = 0; i < 129; i++)
    {
      insert_at (buffer, 6, "x");
      delete_at (buffer, 6, 7);
    }
  g_assert_true (has_text (buffer, "привет"));
  g_assert_false (gtk_text_buffer_typomorph_revert (buffer, token));
  gtk_text_buffer_set_text (buffer, "ghbdtn", -1);
  g_assert_true (replace (buffer, 0));
  token = gtk_text_buffer_typomorph_correction_token (buffer);
  GtkTextIter end;
  gtk_text_buffer_get_end_iter (buffer, &end);
  gtk_text_buffer_create_child_anchor (buffer, &end);
  g_assert_false (gtk_text_buffer_typomorph_revert (buffer, token));
  g_assert_cmpint (gtk_text_buffer_get_char_count (buffer), ==, 7);
}

static void
revert_exact_tracking_bound (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  g_assert_true (replace (buffer, 0));
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  for (int i = 0; i < 128; i++)
    {
      insert_at (buffer, 6, "x");
      delete_at (buffer, 6, 7);
    }
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, token));
  g_assert_true (has_text (buffer, "ghbdtn"));
}

static void
revert_superseded_and_saved (void)
{
  g_autoptr(GtkTextBuffer) buffer = fixture ("ghbdtn ghbdtn");
  gtk_text_buffer_set_enable_undo (buffer, TRUE);
  g_assert_true (replace (buffer, 0));
  guint64 old = gtk_text_buffer_typomorph_correction_token (buffer);
  g_assert_true (replace (buffer, 7));
  guint64 current = gtk_text_buffer_typomorph_correction_token (buffer);
  g_assert_false (gtk_text_buffer_typomorph_revert (buffer, old));
  gtk_text_buffer_set_modified (buffer, FALSE);
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, current));
  g_assert_true (has_text (buffer, "привет ghbdtn"));
  g_assert_true (gtk_text_buffer_get_modified (buffer));
  gtk_text_buffer_undo (buffer);
  g_assert_true (has_text (buffer, "привет привет"));
  g_assert_false (gtk_text_buffer_get_modified (buffer));
  gtk_text_buffer_redo (buffer);
  g_assert_true (has_text (buffer, "привет ghbdtn"));
  g_assert_true (gtk_text_buffer_get_modified (buffer));
}

typedef struct { guint64 token; gboolean fired; } PendingRevert;
static void
revert_while_insert_pending (GObject *object, GParamSpec *spec, PendingRevert *state)
{
  (void) spec;
  if (state->fired)
    return;
  state->fired = TRUE;
  g_assert_false (gtk_text_buffer_typomorph_revert (GTK_TEXT_BUFFER (object), state->token));
}

static void
revert_pending_mutation (void)
{
  g_autoptr(GtkTextBuffer) buffer = history_fixture ();
  g_assert_true (replace (buffer, 0));
  PendingRevert state = { gtk_text_buffer_typomorph_correction_token (buffer), FALSE };
  insert_at (buffer, 6, "x");
  gtk_text_buffer_undo (buffer);
  g_assert_true (gtk_text_buffer_get_can_redo (buffer));
  gulong handler = g_signal_connect (buffer, "notify::can-redo", G_CALLBACK (revert_while_insert_pending), &state);
  insert_at (buffer, 6, "y");
  g_signal_handler_disconnect (buffer, handler);
  g_assert_true (state.fired);
  g_assert_true (has_text (buffer, "приветy"));
  g_assert_true (gtk_text_buffer_typomorph_revert (buffer, state.token));
  g_assert_true (has_text (buffer, "ghbdtny"));
}

static void
add_revert_tests (void)
{
  g_test_add_func ("/revert/exact-tracking-bound", revert_exact_tracking_bound);
  g_test_add_func ("/revert/superseded-and-savepoint", revert_superseded_and_saved);
  g_test_add_func ("/revert/pending-mutation-refuses", revert_pending_mutation);
  g_test_add_func ("/revert/later-typing-history", revert_later_typing);
  g_test_add_func ("/revert/disjoint-unicode-edits", revert_disjoint_edits);
  g_test_add_func ("/revert/overlap-restored-still-refuses", revert_overlap_refuses);
  g_test_add_func ("/revert/token-scope-cancel-history", revert_token_scope);
  g_test_add_func ("/revert/refusal-retry", revert_refusal_retry);
  g_test_add_func ("/revert/callback-edit-history", revert_callback_edit);
  g_test_add_func ("/revert/bound-unknown-mutation", revert_bound_and_unknown_mutation);
}
