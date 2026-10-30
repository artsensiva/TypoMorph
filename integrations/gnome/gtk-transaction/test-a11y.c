/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Fixed payload tests against real GTK AT-SPI and a private registry/listener.
 * No transaction wiring, production capture, or personal documents.
 */
#include <gtk/gtk.h>
#include "gtk/gtkaccessibleprivate.h"
#include "gtk/gtkatcontextprivate.h"
#include "gtk/a11y/gtkatspicontextprivate.h"
#include "gtk/a11y/gtkatspirootprivate.h"

typedef struct {
  GtkTextBuffer *buffer;
  GPtrArray *events;
  gboolean edit_on_delete;
} Listener;

static void
on_event (GDBusConnection *connection, const char *sender, const char *path,
          const char *interface, const char *signal, GVariant *parameters, gpointer data)
{
  (void) connection; (void) sender; (void) path; (void) interface; (void) signal;
  Listener *listener = data;
  g_ptr_array_add (listener->events, g_variant_ref (parameters));
  if (listener->edit_on_delete)
    {
      listener->edit_on_delete = FALSE;
      GtkTextIter start, end;
      gtk_text_buffer_get_bounds (listener->buffer, &start, &end);
      g_autofree char *text = gtk_text_buffer_get_text (listener->buffer, &start, &end, TRUE);
      g_assert_cmpstr (text, ==, "xxпривет");
      gtk_text_buffer_insert (listener->buffer, &end, "x", 1);
    }
}

static void
wait_events (Listener *listener, guint count)
{
  gint64 deadline = g_get_monotonic_time () + 3 * G_TIME_SPAN_SECOND;
  while (listener->events->len < count && g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
  g_assert_cmpuint (listener->events->len, ==, count);
}

static void
assert_event (Listener *listener, guint index, const char *kind, int offset,
              int length, const char *text)
{
  GVariant *event = g_ptr_array_index (listener->events, index);
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
  g_autoptr(GtkTextBuffer) buffer = gtk_text_buffer_new (NULL);
  gtk_text_buffer_set_text (buffer, "xxпривет", -1);
  GtkWidget *view = g_object_ref_sink (gtk_text_view_new_with_buffer (buffer));
  g_autoptr(GtkATContext) context = gtk_accessible_get_at_context (GTK_ACCESSIBLE (view));
  g_assert_true (GTK_IS_AT_SPI_CONTEXT (context));
  GtkAtSpiContext *spi = GTK_AT_SPI_CONTEXT (context);
  gtk_at_context_realize (context);
  gint64 deadline = g_get_monotonic_time () + 3 * G_TIME_SPAN_SECOND;
  while ((!gtk_at_spi_context_typomorph_ready (spi) ||
          !gtk_at_spi_root_has_event_listeners (gtk_at_spi_context_get_root (spi))) &&
         g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
  g_assert_true (gtk_at_spi_context_typomorph_ready (spi));
  g_assert_true (gtk_at_spi_root_has_event_listeners (gtk_at_spi_context_get_root (spi)));
  Listener listener = { buffer, g_ptr_array_new_with_free_func ((GDestroyNotify) g_variant_unref), FALSE };
  guint subscription = g_dbus_connection_signal_subscribe (bus, NULL,
    "org.a11y.atspi.Event.Object", "TextChanged", gtk_at_spi_context_get_context_path (spi),
    NULL, G_DBUS_SIGNAL_FLAGS_NONE, on_event, &listener, NULL);

  g_assert_true (gtk_at_spi_context_typomorph_publish (spi, 2, "ghbdtn", 6, "привет", 12));
  wait_events (&listener, 2);
  assert_event (&listener, 0, "delete", 2, 6, "ghbdtn");
  assert_event (&listener, 1, "insert", 2, 6, "привет");
  g_print ("a11y: SNAPSHOT_PAYLOADS_AND_UNICODE_OFFSETS_PASS\n");

  g_assert_false (gtk_at_spi_context_typomorph_publish (spi, 2, NULL, 6, "привет", 12));
  g_assert_false (gtk_at_spi_context_typomorph_publish (spi, 2, "g\0bdtn", 6, "привет", 12));
  g_assert_false (gtk_at_spi_context_typomorph_publish (spi, 2, "ghbdtn", 6, "\xff", 1));
  g_assert_false (gtk_at_spi_context_typomorph_publish (spi, 2, "ghbdtn", 6, "hi", 2));
  g_assert_false (gtk_at_spi_context_typomorph_publish (spi, G_MAXUINT, "ghbdtn", 6, "привет", 12));
  g_assert_false (gtk_at_spi_context_typomorph_publish (spi, 2, " ", 1, "a", 1));
  g_assert_false (gtk_at_spi_context_typomorph_publish (spi, 2, "a", 129, "b", 1));
  g_assert_false (gtk_at_spi_context_typomorph_publish (spi, 2,
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 33, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", 33));

  /* Full 128-byte bound: 32 supplementary alphabetic scalars. */
  char maximum[129] = { 0 };
  char replacement[65] = { 0 };
  for (int i = 0; i < 32; i++)
    {
      memcpy (maximum + 4 * i, "𐐀", 4);
      memcpy (replacement + 2 * i, "п", 2);
    }
  g_assert_true (gtk_at_spi_context_typomorph_publish (spi, 0, maximum, 128, replacement, 64));
  wait_events (&listener, 4);
  assert_event (&listener, 2, "delete", 0, 32, maximum);
  assert_event (&listener, 3, "insert", 0, 32, replacement);
  g_print ("a11y: MAXIMUM_UTF8_PAYLOAD_AND_SCALAR_COUNT_PASS\n");
  g_ptr_array_set_size (listener.events, 0);

  /* This callback edits on receipt of deletion. The already queued insertion
   * must precede its new ordinary edit event, with its owned payload intact. */
  listener.edit_on_delete = TRUE;
  g_assert_true (gtk_at_spi_context_typomorph_publish (spi, 2, "ghbdtn", 6, "привет", 12));
  wait_events (&listener, 3);
  assert_event (&listener, 0, "delete", 2, 6, "ghbdtn");
  assert_event (&listener, 1, "insert", 2, 6, "привет");
  assert_event (&listener, 2, "insert", 8, 1, "x");
  g_print ("a11y: INVALID_PAYLOAD_REFUSAL_AND_CALLBACK_EVENT_ORDER_PASS\n");

  gtk_at_context_unrealize (context);
  g_assert_false (gtk_at_spi_context_typomorph_ready (spi));
  g_assert_false (gtk_at_spi_context_typomorph_publish (spi, 2, "ghbdtn", 6, "привет", 12));
  g_print ("a11y: UNREALIZED_CONTEXT_REFUSES_PASS\n");
  g_dbus_connection_signal_unsubscribe (bus, subscription);
  g_ptr_array_unref (listener.events);
  g_object_unref (view);
  return 0;
}
