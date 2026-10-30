/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Fixed-input splice through an attached, realized GtkTextView and a real AT-SPI
 * listener on a private Wayland session. No production capture or personal text.
 */
#include <gtk/gtk.h>
#include "gtk/gtkaccessibleprivate.h"
#include "gtk/gtkatcontextprivate.h"
#include "gtk/a11y/gtkatspicontextprivate.h"
#include "gtk/a11y/gtkatspirootprivate.h"

typedef struct {
  GtkTextBuffer *buffer;
  GPtrArray *events;
  char *first_text;
  guint first_revision;
  guint changes;
  gboolean edit_on_change;
} Probe;

static char *
buffer_text (GtkTextBuffer *buffer)
{
  GtkTextIter start, end;
  gtk_text_buffer_get_bounds (buffer, &start, &end);
  return gtk_text_buffer_get_text (buffer, &start, &end, TRUE);
}

static void
assert_text (GtkTextBuffer *buffer, const char *expected)
{
  g_autofree char *text = buffer_text (buffer);
  g_assert_cmpstr (text, ==, expected);
}

/* Buffer state is sampled when the listener receives the first event. */
static void
on_event (GDBusConnection *connection, const char *sender, const char *path,
          const char *interface, const char *signal, GVariant *parameters, gpointer data)
{
  (void) connection; (void) sender; (void) path; (void) interface; (void) signal;
  Probe *probe = data;
  if (probe->events->len == 0)
    {
      g_free (probe->first_text);
      probe->first_text = buffer_text (probe->buffer);
      probe->first_revision = gtk_text_buffer_typomorph_revision (probe->buffer);
    }
  g_ptr_array_add (probe->events, g_variant_ref (parameters));
}

/* An application callback that edits after commit. Its event must follow the
 * replacement events, which proves publication precedes application callbacks. */
static void
on_changed (GtkTextBuffer *buffer, Probe *probe)
{
  probe->changes++;
  if (probe->edit_on_change)
    {
      GtkTextIter end;
      probe->edit_on_change = FALSE;
      assert_text (buffer, "xxпривет");
      gtk_text_buffer_get_end_iter (buffer, &end);
      gtk_text_buffer_insert (buffer, &end, "x", 1);
    }
}

static void
iterate (gint64 duration)
{
  gint64 deadline = g_get_monotonic_time () + duration;
  while (g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
}

static void
reset (Probe *probe)
{
  g_ptr_array_set_size (probe->events, 0);
  g_clear_pointer (&probe->first_text, g_free);
  probe->first_revision = 0;
}

/* Waits for the expected count, then drains briefly to detect extra events. */
static void
wait_events (Probe *probe, guint count)
{
  gint64 deadline = g_get_monotonic_time () + 3 * G_TIME_SPAN_SECOND;
  while (probe->events->len < count && g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
  iterate (200 * G_TIME_SPAN_MILLISECOND);
  g_assert_cmpuint (probe->events->len, ==, count);
}

static void
assert_event (Probe *probe, guint index, const char *kind, int offset,
              int length, const char *text)
{
  GVariant *event = g_ptr_array_index (probe->events, index);
  const char *actual_kind;
  int actual_offset, actual_length;
  GVariant *payload, *properties;
  g_variant_get (event, "(&siiv@a{sv})", &actual_kind, &actual_offset, &actual_length,
                 &payload, &properties);
  g_assert_cmpstr (actual_kind, ==, kind);
  g_assert_cmpint (actual_offset, ==, offset);
  g_assert_cmpint (actual_length, ==, length);
  g_assert_cmpstr (g_variant_get_string (payload, NULL), ==, text);
  g_variant_unref (payload);
  g_variant_unref (properties);
}

static gboolean
context_ready (GtkAtSpiContext *spi)
{
  return gtk_at_spi_context_typomorph_ready (spi) &&
         gtk_at_spi_root_has_event_listeners (gtk_at_spi_context_get_root (spi));
}

static void
wait_ready (GtkAtSpiContext *spi)
{
  gint64 deadline = g_get_monotonic_time () + 3 * G_TIME_SPAN_SECOND;
  while (!context_ready (spi) && g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
}

/* A refused operation leaves text, revision, callbacks and events unchanged. */
static void
assert_refused_unchanged (Probe *probe, const char *text, guint revision, guint changes)
{
  assert_text (probe->buffer, text);
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (probe->buffer), ==, revision);
  g_assert_cmpuint (probe->changes, ==, changes);
  iterate (300 * G_TIME_SPAN_MILLISECOND);
  g_assert_cmpuint (probe->events->len, ==, 0);
}

int
main (void)
{
  const char *runtime = g_getenv ("XDG_RUNTIME_DIR");
  const char *address = g_getenv ("AT_SPI_BUS_ADDRESS");
  if (runtime == NULL || !g_str_has_prefix (runtime, "/tmp/typomorph-a11y-") ||
      address == NULL || strstr (address, runtime) == NULL ||
      g_strcmp0 (g_getenv ("GTK_A11Y"), "atspi") != 0 ||
      g_strcmp0 (g_getenv ("GDK_BACKEND"), "wayland") != 0 ||
      g_getenv ("DISPLAY") != NULL || g_getenv ("WAYLAND_SOCKET") != NULL ||
      g_strcmp0 (g_getenv ("WAYLAND_DISPLAY"), "typomorph-a11y") != 0)
    return 1;
  g_assert_true (gtk_init_check ());
  g_autoptr(GDBusConnection) bus = g_bus_get_sync (G_BUS_TYPE_SESSION, NULL, NULL);
  g_assert_nonnull (bus);
  g_autoptr(GVariant) registration = g_dbus_connection_call_sync (bus,
    "org.a11y.atspi.Registry", "/org/a11y/atspi/registry", "org.a11y.atspi.Registry",
    "RegisterEvent", g_variant_new ("(s@ass)", "object:text-changed",
                                    g_variant_new_strv (NULL, 0), ""), NULL,
    G_DBUS_CALL_FLAGS_NONE, 3000, NULL, NULL);
  g_assert_nonnull (registration);

  GtkTextBuffer *buffer = gtk_text_buffer_new (NULL);
  gtk_text_buffer_set_text (buffer, "xxghbdtn", -1);
  GtkWidget *window = gtk_window_new ();
  GtkWidget *view = gtk_text_view_new_with_buffer (buffer);
  gtk_window_set_child (GTK_WINDOW (window), view);
  gtk_window_present (GTK_WINDOW (window));
  gtk_widget_grab_focus (view);
  gint64 deadline = g_get_monotonic_time () + 5 * G_TIME_SPAN_SECOND;
  while (!gtk_widget_get_mapped (view) && g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
  g_assert_true (gtk_widget_get_realized (view));
  g_assert_true (gtk_widget_get_mapped (view));
  g_assert_true (gtk_text_view_get_buffer (GTK_TEXT_VIEW (view)) == buffer);

  g_autoptr(GtkATContext) context = gtk_accessible_get_at_context (GTK_ACCESSIBLE (view));
  g_assert_true (GTK_IS_AT_SPI_CONTEXT (context));
  GtkAtSpiContext *spi = GTK_AT_SPI_CONTEXT (context);
  /* This fixture realizes the context explicitly. Natural readiness is
   * measured separately by test-a11y-natural.c; presenting alone does not do it. */
  wait_ready (spi);
  if (context_ready (spi))
    g_print ("view-a11y: VIEW_CONTEXT_READY_FROM_PRESENTED_WINDOW\n");
  else
    {
      g_print ("view-a11y: VIEW_CONTEXT_REQUIRED_EXPLICIT_REALIZE\n");
      gtk_at_context_realize (context);
      wait_ready (spi);
    }
  g_assert_true (context_ready (spi));

  Probe probe = { buffer, g_ptr_array_new_with_free_func ((GDestroyNotify) g_variant_unref),
                  NULL, 0, 0, FALSE };
  g_signal_connect (buffer, "changed", G_CALLBACK (on_changed), &probe);
  guint subscription = g_dbus_connection_signal_subscribe (bus, NULL,
    "org.a11y.atspi.Event.Object", "TextChanged", gtk_at_spi_context_get_context_path (spi),
    NULL, G_DBUS_SIGNAL_FLAGS_NONE, on_event, &probe, NULL);
  iterate (200 * G_TIME_SPAN_MILLISECOND);
  g_assert_cmpuint (probe.events->len, ==, 0);

  /* Correction through the attached view: snapshot payloads after commit. */
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_true (gtk_text_buffer_typomorph_splice (buffer, revision, 2, "ghbdtn", 6,
                                                   "привет", 12));
  assert_text (buffer, "xxпривет");
  g_assert_cmpuint (probe.changes, ==, 1);
  guint committed = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_cmpuint (committed, !=, revision);
  wait_events (&probe, 2);
  g_assert_cmpstr (probe.first_text, ==, "xxпривет");
  g_assert_cmpuint (probe.first_revision, ==, committed);
  assert_event (&probe, 0, "delete", 2, 6, "ghbdtn");
  assert_event (&probe, 1, "insert", 2, 6, "привет");
  g_assert_false (gtk_text_buffer_typomorph_notification_failed (buffer));
  g_print ("view-a11y: SPLICE_EVENTS_AFTER_COMMITTED_STORAGE_PASS\n");

  /* Chronological undo/redo replay publishes the inverse and repeated payloads. */
  reset (&probe);
  gtk_text_buffer_undo (buffer);
  assert_text (buffer, "xxghbdtn");
  wait_events (&probe, 2);
  g_assert_cmpstr (probe.first_text, ==, "xxghbdtn");
  assert_event (&probe, 0, "delete", 2, 6, "привет");
  assert_event (&probe, 1, "insert", 2, 6, "ghbdtn");
  reset (&probe);
  gtk_text_buffer_redo (buffer);
  assert_text (buffer, "xxпривет");
  wait_events (&probe, 2);
  g_assert_cmpstr (probe.first_text, ==, "xxпривет");
  assert_event (&probe, 0, "delete", 2, 6, "ghbdtn");
  assert_event (&probe, 1, "insert", 2, 6, "привет");
  g_print ("view-a11y: HISTORY_REPLAY_EVENTS_PASS\n");

  /* Application change callback edits after commit; ordinary insert follows. */
  reset (&probe);
  gtk_text_buffer_undo (buffer);
  wait_events (&probe, 2);
  reset (&probe);
  probe.edit_on_change = TRUE;
  revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_true (gtk_text_buffer_typomorph_splice (buffer, revision, 2, "ghbdtn", 6,
                                                   "привет", 12));
  assert_text (buffer, "xxприветx");
  wait_events (&probe, 3);
  assert_event (&probe, 0, "delete", 2, 6, "ghbdtn");
  assert_event (&probe, 1, "insert", 2, 6, "привет");
  assert_event (&probe, 2, "insert", 8, 1, "x");
  g_print ("view-a11y: CALLBACK_EDIT_EVENT_FOLLOWS_REPLACEMENT_PASS\n");

  /* The migrated ordinary path still reports removed text before deletion. */
  reset (&probe);
  GtkTextIter start, end;
  gtk_text_buffer_get_iter_at_offset (buffer, &start, 8);
  gtk_text_buffer_get_end_iter (buffer, &end);
  gtk_text_buffer_delete (buffer, &start, &end);
  assert_text (buffer, "xxпривет");
  wait_events (&probe, 1);
  assert_event (&probe, 0, "delete", 8, 1, "x");
  g_print ("view-a11y: ORDINARY_DELETE_EVENT_PASS\n");

  /* A second attached view with an unrealized context refuses without edit. */
  reset (&probe);
  GtkWidget *hidden = g_object_ref_sink (gtk_text_view_new_with_buffer (buffer));
  g_autoptr(GtkATContext) hidden_context = gtk_accessible_get_at_context (GTK_ACCESSIBLE (hidden));
  g_assert_true (GTK_IS_AT_SPI_CONTEXT (hidden_context));
  g_assert_false (gtk_at_spi_context_typomorph_ready (GTK_AT_SPI_CONTEXT (hidden_context)));
  revision = gtk_text_buffer_typomorph_revision (buffer);
  guint changes = probe.changes;
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 2, "привет", 12,
                                                    "ghbdtn", 6));
  assert_refused_unchanged (&probe, "xxпривет", revision, changes);
  g_object_unref (hidden);
  g_print ("view-a11y: UNREALIZED_SECOND_VIEW_REFUSES_WITHOUT_EDIT_PASS\n");

  /* Commit a replacement, then unrealize the view context: splice, undo and
   * targeted reversal all refuse, and the history entry is retained. */
  revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_true (gtk_text_buffer_typomorph_splice (buffer, revision, 2, "привет", 12,
                                                   "ghbdtn", 6));
  wait_events (&probe, 2);
  guint64 token = gtk_text_buffer_typomorph_correction_token (buffer);
  g_assert_cmpuint (token, !=, 0);
  reset (&probe);
  gtk_at_context_unrealize (context);
  g_assert_false (gtk_at_spi_context_typomorph_ready (spi));
  revision = gtk_text_buffer_typomorph_revision (buffer);
  changes = probe.changes;
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 2, "ghbdtn", 6,
                                                    "привет", 12));
  gtk_text_buffer_undo (buffer);
  g_assert_false (gtk_text_buffer_typomorph_revert (buffer, token));
  assert_refused_unchanged (&probe, "xxghbdtn", revision, changes);
  g_assert_true (gtk_text_buffer_get_can_undo (buffer));
  g_assert_cmpuint (gtk_text_buffer_typomorph_correction_token (buffer), ==, token);
  g_assert_false (gtk_text_buffer_typomorph_notification_failed (buffer));
  g_print ("view-a11y: UNREALIZED_CONTEXT_REFUSES_SPLICE_UNDO_REVERT_PASS\n");

  g_dbus_connection_signal_unsubscribe (bus, subscription);
  g_ptr_array_unref (probe.events);
  g_free (probe.first_text);
  g_signal_handlers_disconnect_by_func (buffer, on_changed, &probe);
  gtk_window_destroy (GTK_WINDOW (window));
  iterate (100 * G_TIME_SPAN_MILLISECOND);
  g_object_unref (buffer);
  return 0;
}
