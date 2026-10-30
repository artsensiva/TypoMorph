/* SPDX-License-Identifier: LGPL-2.1-or-later
 * Natural AT-SPI context readiness of an attached GtkTextView on the private
 * headless Wayland session. Never calls gtk_at_context_realize(). Fixed synthetic
 * text only; no production capture or personal documents.
 *
 * One scenario per fresh process (argv[1]), so no scenario sees another's window,
 * root registration or client traversal. "Cold" means exactly that: the first and
 * only GTK window of this process, in a freshly started private session.
 *
 *   primary       view present at window presentation; passive wait; then a client walk
 *   late-added    view added to an already presented, mapped window; no client walk
 *   listener-gap  characterizes the interval where a view is ready but GTK has not
 *                 yet learned of any event listener
 *   listener-gap-timeout, listener-gap-settled
 *                 harness self-tests of listener-gap's rare exits: a 1 ms deadline
 *                 (expect exit 3, timeout before readiness) and waiting for the
 *                 listener table before sampling (expect exit 7, gap absent)
 *
 * Exit codes (anything else, including an abort from a failed assertion, is failure):
 *   0 EXIT_OK            the scenario's pinned outcome was observed and verified
 *   2 EXIT_USAGE         unknown scenario or private fixture boundary violated
 *   3 EXIT_NOT_READY     readiness not established (or timeout); splice verified to refuse
 *   4 EXIT_LISTENERS     ready, but GTK's listener table never populated
 *   5 EXIT_PASSIVE       primary: passive readiness reached (pinned claim changed)
 *   7 EXIT_GAP_ABSENT    listener-gap: the interval was not observed (non-reproduction)
 */
#include <gtk/gtk.h>
#include "gtk/gtkaccessibleprivate.h"
#include "gtk/gtkatcontextprivate.h"
#include "gtk/a11y/gtkatspicontextprivate.h"
#include "gtk/a11y/gtkatspirootprivate.h"

enum { EXIT_OK = 0, EXIT_USAGE = 2, EXIT_NOT_READY = 3, EXIT_LISTENERS = 4,
       EXIT_PASSIVE = 5, EXIT_GAP_ABSENT = 7 };

typedef struct {
  GtkTextBuffer *buffer;
  GPtrArray *events;
  char *first_text;
} Probe;

static GtkWidget *window, *view;
static GtkAtSpiContext *view_spi, *window_spi;

static char *
buffer_text (GtkTextBuffer *buffer)
{
  GtkTextIter start, end;
  gtk_text_buffer_get_bounds (buffer, &start, &end);
  return gtk_text_buffer_get_text (buffer, &start, &end, TRUE);
}

static void
on_event (GDBusConnection *connection, const char *sender, const char *path,
          const char *interface, const char *signal, GVariant *parameters, gpointer data)
{
  (void) connection; (void) sender; (void) path; (void) interface; (void) signal;
  Probe *probe = data;
  if (probe->events->len == 0)
    probe->first_text = buffer_text (probe->buffer);
  g_ptr_array_add (probe->events, g_variant_ref (parameters));
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

static gboolean
view_ready (void)
{
  return gtk_at_spi_context_typomorph_ready (view_spi);
}

/* The root exists only after a context realizes; -1 means no root yet. */
static int
listeners (void)
{
  GtkAtSpiRoot *root = gtk_at_spi_context_get_root (view_spi);
  if (root == NULL)
    root = gtk_at_spi_context_get_root (window_spi);
  return root == NULL ? -1 : gtk_at_spi_root_has_event_listeners (root);
}

static void
snapshot (const char *label)
{
  g_print ("natural: PHASE %-28s widget_realized=%d mapped=%d window_ctx_realized=%d "
           "view_ctx_realized=%d view_ready=%d listeners=%d\n", label,
           gtk_widget_get_realized (view), gtk_widget_get_mapped (view),
           gtk_at_context_is_realized (GTK_AT_CONTEXT (window_spi)),
           gtk_at_context_is_realized (GTK_AT_CONTEXT (view_spi)), view_ready (),
           listeners ());
}

typedef struct { GVariant *reply; gchar *error; gboolean done; } Call;

static void
on_reply (GObject *source, GAsyncResult *result, gpointer data)
{
  Call *call = data;
  g_autoptr(GError) error = NULL;
  call->reply = g_dbus_connection_call_finish (G_DBUS_CONNECTION (source), result, &error);
  if (call->reply == NULL)
    call->error = g_strdup (error->message);
  call->done = TRUE;
}

/* Asynchronous because the GTK side is served by this thread's main context. */
static GVariant *
children_of (GDBusConnection *bus, const char *name, const char *path, char **error)
{
  Call call = { NULL, NULL, FALSE };
  g_dbus_connection_call (bus, name, path, "org.a11y.atspi.Accessible", "GetChildren",
                          NULL, G_VARIANT_TYPE ("(a(so))"), G_DBUS_CALL_FLAGS_NONE, 3000,
                          NULL, on_reply, &call);
  gint64 deadline = g_get_monotonic_time () + 4 * G_TIME_SPAN_SECOND;
  while (!call.done && g_get_monotonic_time () < deadline)
    g_main_context_iteration (NULL, TRUE);
  if (error)
    *error = call.error;
  else
    g_free (call.error);
  return call.reply;
}

/* Walks Registry root -> application root -> window -> window children, as an
 * assistive technology would. Returns whether the view's path was listed. */
static gboolean
client_walk (GDBusConnection *bus, GtkAtSpiRoot *root)
{
  const char *app_name = g_dbus_connection_get_unique_name (gtk_at_spi_root_get_connection (root));
  const char *window_path = gtk_at_spi_context_get_context_path (window_spi);
  g_autofree char *error = NULL;
  g_autoptr(GVariant) desktop = children_of (bus, "org.a11y.atspi.Registry",
                                             "/org/a11y/atspi/accessible/root", &error);
  if (desktop == NULL)
    {
      g_print ("natural: WALK registry root GetChildren failed: %s\n", error);
      return FALSE;
    }
  g_autoptr(GVariant) applications = g_variant_get_child_value (desktop, 0);
  g_print ("natural: WALK registry lists %zu application(s)\n", g_variant_n_children (applications));
  gboolean found_app = FALSE;
  const char *name, *path;
  GVariantIter iter;
  g_variant_iter_init (&iter, applications);
  while (g_variant_iter_next (&iter, "(&s&o)", &name, &path))
    if (g_strcmp0 (name, app_name) == 0)
      {
        found_app = TRUE;
        break;
      }
  g_print ("natural: WALK application found in registry: %d\n", found_app);
  if (!found_app)
    return FALSE;
  snapshot ("after_registry_discovery");

  g_autofree char *app_error = NULL;
  g_autoptr(GVariant) app = children_of (bus, name, path, &app_error);
  if (app == NULL)
    {
      g_print ("natural: WALK application GetChildren failed: %s\n", app_error);
      return FALSE;
    }
  g_autoptr(GVariant) windows = g_variant_get_child_value (app, 0);
  g_print ("natural: WALK application lists %zu window(s)\n", g_variant_n_children (windows));
  gboolean found_window = FALSE;
  g_autofree char *window_name = NULL, *window_ref = NULL;
  g_variant_iter_init (&iter, windows);
  while (g_variant_iter_next (&iter, "(&s&o)", &name, &path))
    if (g_strcmp0 (path, window_path) == 0)
      {
        found_window = TRUE;
        window_name = g_strdup (name);
        window_ref = g_strdup (path);
      }
  g_print ("natural: WALK window path listed by application: %d\n", found_window);
  snapshot ("after_application_children");
  if (!found_window)
    return FALSE;

  g_autofree char *window_error = NULL;
  g_autoptr(GVariant) body = children_of (bus, window_name, window_ref, &window_error);
  if (body == NULL)
    {
      g_print ("natural: WALK window GetChildren failed: %s\n", window_error);
      return FALSE;
    }
  g_autoptr(GVariant) children = g_variant_get_child_value (body, 0);
  g_print ("natural: WALK window lists %zu child(ren)\n", g_variant_n_children (children));
  gboolean found_view = FALSE;
  /* A context has no path until it is realized, which this reply does. */
  const char *view_path = gtk_at_spi_context_get_context_path (view_spi);
  g_variant_iter_init (&iter, children);
  while (g_variant_iter_next (&iter, "(&s&o)", &name, &path))
    if (view_path != NULL && g_strcmp0 (path, view_path) == 0)
      found_view = TRUE;
  g_print ("natural: WALK view path listed by window: %d\n", found_view);
  snapshot ("after_window_children");
  return found_view;
}

static void
assert_event (Probe *probe, guint index, const char *kind, int offset, int length,
              const char *text)
{
  const char *actual_kind;
  int actual_offset, actual_length;
  GVariant *payload, *properties;
  g_variant_get (g_ptr_array_index (probe->events, index), "(&siiv@a{sv})", &actual_kind,
                 &actual_offset, &actual_length, &payload, &properties);
  g_assert_cmpstr (actual_kind, ==, kind);
  g_assert_cmpint (actual_offset, ==, offset);
  g_assert_cmpint (actual_length, ==, length);
  g_assert_cmpstr (g_variant_get_string (payload, NULL), ==, text);
  g_variant_unref (payload);
  g_variant_unref (properties);
}

/* Subscribes to this view's TextChanged events. The path exists once realized. */
static guint
subscribe (GDBusConnection *bus, Probe *probe, const char *path)
{
  return g_dbus_connection_signal_subscribe (bus, NULL, "org.a11y.atspi.Event.Object",
    "TextChanged", path, NULL, G_DBUS_SIGNAL_FLAGS_NONE, on_event, probe, NULL);
}

static void
probe_init (Probe *probe, GtkTextBuffer *buffer)
{
  probe->buffer = buffer;
  probe->events = g_ptr_array_new_with_free_func ((GDestroyNotify) g_variant_unref);
  probe->first_text = NULL;
}

static void
probe_clear (Probe *probe)
{
  g_ptr_array_unref (probe->events);
  g_free (probe->first_text);
}

static void
wait_events (Probe *probe, guint count)
{
  gint64 deadline = g_get_monotonic_time () + 3 * G_TIME_SPAN_SECOND;
  while (probe->events->len < count && g_get_monotonic_time () < deadline)
    g_main_context_iteration (NULL, TRUE);
  iterate (200 * G_TIME_SPAN_MILLISECOND);
}

/* Waits (bounded) for GTK's listener table, then splices through the ready view
 * and asserts ordered events and committed state at the first event. Returns
 * EXIT_OK or EXIT_LISTENERS. Mismatched events abort, which is a failure. */
static int
verify_splice (GDBusConnection *bus, GtkTextBuffer *buffer, GtkAtSpiContext *spi)
{
  GtkAtSpiRoot *root = gtk_at_spi_context_get_root (spi);
  gint64 begin = g_get_monotonic_time ();
  while (!gtk_at_spi_root_has_event_listeners (root) &&
         g_get_monotonic_time () < begin + 3 * G_TIME_SPAN_SECOND)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
  if (!gtk_at_spi_root_has_event_listeners (root))
    {
      g_print ("natural: LISTENER_STATE_NOT_ESTABLISHED_AFTER_3S; splice skipped\n");
      return EXIT_LISTENERS;
    }
  g_print ("natural: LISTENERS_KNOWN_AFTER_WAIT_MS %" G_GINT64_FORMAT "\n",
           (g_get_monotonic_time () - begin) / 1000);
  Probe probe;
  probe_init (&probe, buffer);
  guint subscription = subscribe (bus, &probe, gtk_at_spi_context_get_context_path (spi));
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_true (gtk_text_buffer_typomorph_splice (buffer, revision, 2, "ghbdtn", 6,
                                                   "привет", 12));
  wait_events (&probe, 2);
  g_assert_cmpuint (probe.events->len, ==, 2);
  g_assert_cmpstr (probe.first_text, ==, "xxпривет");
  assert_event (&probe, 0, "delete", 2, 6, "ghbdtn");
  assert_event (&probe, 1, "insert", 2, 6, "привет");
  g_dbus_connection_signal_unsubscribe (bus, subscription);
  probe_clear (&probe);
  return EXIT_OK;
}

/* Readiness was not established: the splice must refuse with no edit, no revision
 * change and no event. */
static void
verify_refusal (GDBusConnection *bus, GtkTextBuffer *buffer)
{
  Probe probe;
  probe_init (&probe, buffer);
  guint subscription = subscribe (bus, &probe, NULL);
  guint revision = gtk_text_buffer_typomorph_revision (buffer);
  g_assert_false (gtk_text_buffer_typomorph_splice (buffer, revision, 2, "ghbdtn", 6,
                                                    "привет", 12));
  g_autofree char *text = buffer_text (buffer);
  g_assert_cmpstr (text, ==, "xxghbdtn");
  g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer), ==, revision);
  g_assert_false (gtk_text_buffer_typomorph_notification_failed (buffer));
  iterate (300 * G_TIME_SPAN_MILLISECOND);
  g_assert_cmpuint (probe.events->len, ==, 0);
  g_dbus_connection_signal_unsubscribe (bus, subscription);
  probe_clear (&probe);
}

static void
present_and_map (GtkWidget *w)
{
  gtk_window_present (GTK_WINDOW (w));
  gint64 deadline = g_get_monotonic_time () + 5 * G_TIME_SPAN_SECOND;
  while (!gtk_widget_get_mapped (w) && g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
  g_assert_true (gtk_widget_get_mapped (w));
}

/* Scenario primary: the pinned claim is that a view present at presentation is NOT
 * ready passively, and IS ready after a client walk, with verified events. */
static int
scenario_primary (GDBusConnection *bus)
{
  GtkTextBuffer *buffer = gtk_text_buffer_new (NULL);
  gtk_text_buffer_set_text (buffer, "xxghbdtn", -1);
  window = gtk_window_new ();
  view = gtk_text_view_new_with_buffer (buffer);
  gtk_window_set_child (GTK_WINDOW (window), view);
  g_autoptr(GtkATContext) view_context = gtk_accessible_get_at_context (GTK_ACCESSIBLE (view));
  g_autoptr(GtkATContext) window_context = gtk_accessible_get_at_context (GTK_ACCESSIBLE (window));
  g_assert_true (GTK_IS_AT_SPI_CONTEXT (view_context));
  g_assert_true (GTK_IS_AT_SPI_CONTEXT (window_context));
  view_spi = GTK_AT_SPI_CONTEXT (view_context);
  window_spi = GTK_AT_SPI_CONTEXT (window_context);
  snapshot ("before_present");
  gtk_window_present (GTK_WINDOW (window));
  gtk_widget_grab_focus (view);
  snapshot ("after_present_no_iteration");
  gint64 deadline = g_get_monotonic_time () + 5 * G_TIME_SPAN_SECOND;
  while (!gtk_widget_get_mapped (view) && g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
  g_assert_true (gtk_widget_get_mapped (view));
  snapshot ("after_mapped");

  /* Passive wait only: no AT-SPI client activity, no explicit realization. */
  deadline = g_get_monotonic_time () + 3 * G_TIME_SPAN_SECOND;
  while (!view_ready () && g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
  gboolean passive = view_ready ();
  snapshot ("after_passive_wait_3s");
  g_print ("natural: PASSIVE_READINESS %s\n", passive ? "REACHED" : "NOT_REACHED");

  int status;
  if (passive)
    {
      /* Not a safety failure, but the documented claim no longer holds. */
      status = verify_splice (bus, buffer, view_spi);
      g_print ("natural: PRIMARY_PASSIVE_READINESS_UNEXPECTED_CLAIM_CHANGED\n");
      status = status == EXIT_OK ? EXIT_PASSIVE : status;
    }
  else
    {
      /* Before the walk, the unready view must refuse a splice without effect. */
      verify_refusal (bus, buffer);
      g_print ("natural: UNREADY_VIEW_SPLICE_REFUSED_WITHOUT_EDIT\n");
      gboolean walked = client_walk (bus, gtk_at_spi_context_get_root (window_spi));
      deadline = g_get_monotonic_time () + 3 * G_TIME_SPAN_SECOND;
      while (!view_ready () && g_get_monotonic_time () < deadline)
        {
          while (g_main_context_iteration (NULL, FALSE));
          g_usleep (1000);
        }
      snapshot ("after_walk_wait_3s");
      g_print ("natural: CLIENT_WALK_READINESS %s (view listed: %d)\n",
               view_ready () ? "REACHED" : "NOT_REACHED", walked);
      if (!view_ready () || !walked)
        {
          verify_refusal (bus, buffer);
          g_print ("natural: READINESS_NOT_ESTABLISHED_SPLICE_REFUSED_WITHOUT_EDIT\n");
          status = EXIT_NOT_READY;
        }
      else
        {
          status = verify_splice (bus, buffer, view_spi);
          if (status == EXIT_OK)
            g_print ("natural: PRIMARY_SPLICE_EVENTS_AFTER_CLIENT_WALK_PASS\n");
        }
    }
  gtk_window_destroy (GTK_WINDOW (window));
  iterate (100 * G_TIME_SPAN_MILLISECOND);
  g_object_unref (buffer);
  return status;
}

/* Scenario late-added: a view added to an already presented, mapped window must be
 * ready without a client walk, and its splice events must verify. */
static int
scenario_late_added (GDBusConnection *bus)
{
  GtkWidget *window2 = gtk_window_new ();
  gint64 presented = g_get_monotonic_time ();
  present_and_map (window2);
  iterate (300 * G_TIME_SPAN_MILLISECOND);
  GtkTextBuffer *buffer2 = gtk_text_buffer_new (NULL);
  gtk_text_buffer_set_text (buffer2, "xxghbdtn", -1);
  GtkWidget *view2 = gtk_text_view_new_with_buffer (buffer2);
  g_autoptr(GtkATContext) context2 = gtk_accessible_get_at_context (GTK_ACCESSIBLE (view2));
  GtkAtSpiContext *spi2 = GTK_AT_SPI_CONTEXT (context2);
  gtk_window_set_child (GTK_WINDOW (window2), view2);
  /* Root registration completes about 3 s after the first present in this session,
   * so the budget must exceed that with margin. */
  gint64 deadline = g_get_monotonic_time () + 8 * G_TIME_SPAN_SECOND;
  while (!gtk_at_spi_context_typomorph_ready (spi2) && g_get_monotonic_time () < deadline)
    {
      while (g_main_context_iteration (NULL, FALSE));
      g_usleep (1000);
    }
  gboolean ready = gtk_at_spi_context_typomorph_ready (spi2);
  g_print ("natural: LATE_ADDED_VIEW_READINESS %s (ctx_realized=%d, no client walk, "
           "%" G_GINT64_FORMAT " ms after present)\n",
           ready ? "REACHED" : "NOT_REACHED", gtk_at_context_is_realized (context2),
           (g_get_monotonic_time () - presented) / 1000);
  int status;
  if (ready)
    {
      status = verify_splice (bus, buffer2, spi2);
      if (status == EXIT_OK)
        g_print ("natural: LATE_ADDED_VIEW_SPLICE_EVENTS_PASS\n");
    }
  else
    {
      verify_refusal (bus, buffer2);
      g_print ("natural: READINESS_NOT_ESTABLISHED_SPLICE_REFUSED_WITHOUT_EDIT\n");
      status = EXIT_NOT_READY;
    }
  gtk_window_destroy (GTK_WINDOW (window2));
  iterate (100 * G_TIME_SPAN_MILLISECOND);
  g_object_unref (buffer2);
  return status;
}

static gboolean
stop_wait (gpointer data)
{
  *(gboolean *) data = TRUE;
  return G_SOURCE_REMOVE;
}

/* Scenario listener-gap. State-driven, not sleep-driven: dispatch one main-loop
 * source at a time and test, after each dispatch, whether the view is ready while
 * GTK's root reports no event listener. If that state is observed, splice at once,
 * before any further dispatch, and assert the prototype's actual behavior: the
 * splice commits, nothing is reported as failed, and no event is delivered.
 * If the state is never observed, this is a non-reproduction, not a pass. */
static int
scenario_listener_gap (GDBusConnection *bus, guint timeout_ms, gboolean settle)
{
  GtkWidget *window2 = gtk_window_new ();
  GtkTextBuffer *buffer2 = gtk_text_buffer_new (NULL);
  gtk_text_buffer_set_text (buffer2, "xxghbdtn", -1);
  GtkWidget *view2 = gtk_text_view_new_with_buffer (buffer2);
  g_autoptr(GtkATContext) context2 = gtk_accessible_get_at_context (GTK_ACCESSIBLE (view2));
  GtkAtSpiContext *spi2 = GTK_AT_SPI_CONTEXT (context2);
  Probe probe;
  probe_init (&probe, buffer2);
  /* Present and add the view before any main-loop dispatch: the window context
   * realizes synchronously, so the child context realizes with it. */
  gtk_window_present (GTK_WINDOW (window2));
  gtk_window_set_child (GTK_WINDOW (window2), view2);
  guint subscription = subscribe (bus, &probe, NULL);

  gboolean timed_out = FALSE;
  guint timer = g_timeout_add (timeout_ms, stop_wait, &timed_out);
  gint64 start = g_get_monotonic_time ();
  guint dispatches = 0;
  gboolean saw_ready = FALSE, in_gap = FALSE;
  gint64 ready_us = 0, root_us = -1, chatty_end_us = -1;
  gboolean last_listeners = FALSE;
  while (!timed_out)
    {
      g_main_context_iteration (NULL, TRUE);
      dispatches++;
      GtkAtSpiRoot *root = gtk_at_spi_context_get_root (spi2);
      if (root != NULL)
        {
          gboolean now = gtk_at_spi_root_has_event_listeners (root);
          if (root_us < 0)
            root_us = g_get_monotonic_time () - start;
          if (last_listeners && !now && chatty_end_us < 0)
            chatty_end_us = g_get_monotonic_time () - start;
          last_listeners = now;
        }
      if (root == NULL || !gtk_at_spi_context_typomorph_ready (spi2))
        continue;
      if (!saw_ready)
        {
          saw_ready = TRUE;
          ready_us = g_get_monotonic_time () - start;
        }
      if (settle)
        while (!gtk_at_spi_root_has_event_listeners (root) && !timed_out)
          g_main_context_iteration (NULL, TRUE);
      in_gap = !gtk_at_spi_root_has_event_listeners (root);
      break;
    }
  /* stop_wait returns G_SOURCE_REMOVE, so a fired timer no longer exists. */
  if (!timed_out)
    g_source_remove (timer);

  int status;
  if (!saw_ready)
    {
      g_print ("natural: LISTENER_GAP_VIEW_NEVER_READY\n");
      status = EXIT_NOT_READY;
    }
  else if (!in_gap)
    {
      g_print ("natural: LISTENER_GAP_NOT_OBSERVED (listeners already known when ready, "
               "after %u dispatches, %" G_GINT64_FORMAT " us)\n", dispatches, ready_us);
      status = EXIT_GAP_ABSENT;
    }
  else
    {
      GtkAtSpiRoot *root = gtk_at_spi_context_get_root (spi2);
      g_print ("natural: LISTENER_GAP_OBSERVED ready_and_no_listeners after %u dispatches\n"
               "natural: LISTENER_GAP_TIMELINE_US root_assigned=%" G_GINT64_FORMAT
               " listeners_flag_true_to_false=%" G_GINT64_FORMAT " view_ready=%" G_GINT64_FORMAT "\n",
               dispatches, root_us, chatty_end_us, ready_us);
      guint revision = gtk_text_buffer_typomorph_revision (buffer2);
      /* Same dispatch step: no main-loop iteration between the observation and this call. */
      gboolean committed = gtk_text_buffer_typomorph_splice (buffer2, revision, 2, "ghbdtn", 6,
                                                             "привет", 12);
      gboolean still_gap = !gtk_at_spi_root_has_event_listeners (root);
      g_assert_true (committed);
      g_assert_true (still_gap);
      g_autofree char *text = buffer_text (buffer2);
      g_assert_cmpstr (text, ==, "xxпривет");
      g_assert_cmpuint (gtk_text_buffer_typomorph_revision (buffer2), !=, revision);
      g_assert_false (gtk_text_buffer_typomorph_notification_failed (buffer2));
      /* Characterization of the hazard: committed, flagged nowhere, nothing delivered. */
      gint64 gap_start = g_get_monotonic_time ();
      gint64 deadline = gap_start + 3 * G_TIME_SPAN_SECOND;
      while (!gtk_at_spi_root_has_event_listeners (root) && g_get_monotonic_time () < deadline)
        g_main_context_iteration (NULL, TRUE);
      gboolean listeners_later = gtk_at_spi_root_has_event_listeners (root);
      gint64 learned_us = g_get_monotonic_time () - gap_start;
      iterate (300 * G_TIME_SPAN_MILLISECOND);
      g_assert_cmpuint (probe.events->len, ==, 0);
      g_print ("natural: LISTENER_GAP_SPLICE_COMMITTED_NO_EVENTS_NO_FAILURE_FLAG "
               "(listeners learned %" G_GINT64_FORMAT " us later: %d)\n",
               learned_us, listeners_later);
      g_assert_true (listeners_later);
      /* Steady state: the reverse splice must now deliver events. */
      revision = gtk_text_buffer_typomorph_revision (buffer2);
      g_assert_true (gtk_text_buffer_typomorph_splice (buffer2, revision, 2, "привет", 12,
                                                       "ghbdtn", 6));
      wait_events (&probe, 2);
      g_assert_cmpuint (probe.events->len, ==, 2);
      assert_event (&probe, 0, "delete", 2, 6, "привет");
      assert_event (&probe, 1, "insert", 2, 6, "ghbdtn");
      g_print ("natural: LISTENER_GAP_STEADY_STATE_SPLICE_DELIVERS_EVENTS\n");
      status = EXIT_OK;
    }
  g_dbus_connection_signal_unsubscribe (bus, subscription);
  probe_clear (&probe);
  gtk_window_destroy (GTK_WINDOW (window2));
  iterate (100 * G_TIME_SPAN_MILLISECOND);
  g_object_unref (buffer2);
  return status;
}

int
main (int argc, char **argv)
{
  const char *scenario = argc == 2 ? argv[1] : NULL;
  const char *runtime = g_getenv ("XDG_RUNTIME_DIR");
  const char *address = g_getenv ("AT_SPI_BUS_ADDRESS");
  if (scenario == NULL ||
      (strcmp (scenario, "primary") && strcmp (scenario, "late-added") &&
       strcmp (scenario, "listener-gap") && strcmp (scenario, "listener-gap-timeout") &&
       strcmp (scenario, "listener-gap-settled")) ||
      runtime == NULL || !g_str_has_prefix (runtime, "/tmp/typomorph-a11y-") ||
      address == NULL || strstr (address, runtime) == NULL ||
      g_strcmp0 (g_getenv ("GTK_A11Y"), "atspi") != 0 ||
      g_strcmp0 (g_getenv ("GDK_BACKEND"), "wayland") != 0 ||
      g_getenv ("DISPLAY") != NULL || g_getenv ("WAYLAND_SOCKET") != NULL ||
      g_strcmp0 (g_getenv ("WAYLAND_DISPLAY"), "typomorph-a11y") != 0)
    return EXIT_USAGE;
  g_assert_true (gtk_init_check ());
  g_autoptr(GDBusConnection) bus = g_bus_get_sync (G_BUS_TYPE_SESSION, NULL, NULL);
  g_assert_nonnull (bus);
  g_autoptr(GVariant) registration = g_dbus_connection_call_sync (bus,
    "org.a11y.atspi.Registry", "/org/a11y/atspi/registry", "org.a11y.atspi.Registry",
    "RegisterEvent", g_variant_new ("(s@ass)", "object:text-changed",
                                    g_variant_new_strv (NULL, 0), ""), NULL,
    G_DBUS_CALL_FLAGS_NONE, 3000, NULL, NULL);
  g_assert_nonnull (registration);

  if (strcmp (scenario, "primary") == 0)
    return scenario_primary (bus);
  if (strcmp (scenario, "late-added") == 0)
    return scenario_late_added (bus);
  /* The last two are harness self-tests that force the otherwise rare exits. */
  if (strcmp (scenario, "listener-gap-timeout") == 0)
    return scenario_listener_gap (bus, 1, FALSE);
  if (strcmp (scenario, "listener-gap-settled") == 0)
    return scenario_listener_gap (bus, 5000, TRUE);
  return scenario_listener_gap (bus, 5000, FALSE);
}
