"""Real GTK widgets, forced IBus module on Broadway; no physical input coverage."""
import time
import unittest
import warnings
import gi
from run_gtk_tests import check_environment
check_environment()
gi.require_version('Gtk', '4.0')
gi.require_version('IBus', '1.0')
from gi.repository import Gtk, Gio, GLib, IBus


def drain(seconds=.15):
    end = time.monotonic()+seconds
    while time.monotonic()<end:
        while GLib.MainContext.default().pending():
            GLib.MainContext.default().iteration(False)
        time.sleep(.005)


def control(method):
    return Gio.bus_get_sync(Gio.BusType.SESSION, None).call_sync(
        'org.typomorph.GtkFixture', '/org/typomorph/GtkFixture', 'org.typomorph.GtkFixture',
        method, None, None, 0, 1000, None).unpack()


class Widgets(unittest.TestCase):
    def setUp(self):
        warnings.filterwarnings('ignore', category=DeprecationWarning, module=r'gi\.events')
        Gtk.init(); IBus.init()
        self.bus = IBus.Bus.new()
        self.window = Gtk.Window()
        box = Gtk.Box()
        self.a, self.b, self.secret = Gtk.Text(), Gtk.Text(), Gtk.Text()
        self.secret.set_visibility(False)
        self.secret.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        for widget in (self.a, self.b, self.secret):
            box.append(widget)
        self.window.set_child(box)
        self.window.present()
        self.a.grab_focus()
        drain(.5)
        self.assertTrue(self.window.is_active())
        self.assertTrue(self.bus.set_global_engine('typomorph-gtk-fixture'))
        drain(.3)

    def tearDown(self):
        self.window.destroy()
        drain()

    def focus(self, widget):
        self.assertTrue(widget.grab_focus())
        drain()
        self.assertTrue(widget.has_focus())
        state = control('State')
        self.assertTrue(state[0])
        return state

    def context(self, widget):
        controllers = widget.observe_controllers()
        contexts = [controllers.get_item(i).get_im_context()
                    for i in range(controllers.get_n_items())
                    if isinstance(controllers.get_item(i), Gtk.EventControllerKey)]
        return next(context for context in contexts if context is not None)

    def test_native_reset_preserves_seeded_draft_once(self):
        self.focus(self.a)
        self.assertTrue(control('Seed')[0])
        drain()
        self.context(self.a).reset()
        drain(.5)
        control('State')  # fence the engine connection before judging disposition
        drain()
        if self.a.get_text() != 'ghb':
            print('reset_draft_still_preedit=' + str(self.context(self.a).get_preedit_string()[0] == 'ghb'), flush=True)
        self.assertEqual(self.a.get_text(), 'ghb')
        self.context(self.a).reset()
        drain()
        self.assertEqual(self.a.get_text(), 'ghb')

    def test_rapid_round_trip_preserves_seeded_draft(self):
        self.focus(self.a)
        self.assertTrue(control('Seed')[0])
        drain()
        self.b.grab_focus()
        self.a.grab_focus()  # no GLib drain between the two focus requests
        drain()
        self.assertEqual(self.a.get_text(), 'ghb')
        self.assertEqual(self.b.get_text(), '')

    def test_key_filter_reaches_native_engine_without_permission_fixture(self):
        self.focus(self.a)
        before = control('State')[2]
        im = self.context(self.a)
        display = self.window.get_display()
        keyboard = display.get_default_seat().get_keyboard()
        im.filter_key(True, self.window.get_surface(), keyboard, 1, ord('x'), 0, 0)
        drain()
        self.assertGreater(control('State')[2], before)
        self.assertEqual(self.a.get_text(), 'x')

    def echo_key(self, widget):
        self.context(widget).filter_key(True, self.window.get_surface(),
            self.window.get_display().get_default_seat().get_keyboard(), 1, ord('x'), 0, 0)

    def test_input_immediately_after_focus_keeps_draft_on_old_widget(self):
        self.focus(self.a)
        self.assertTrue(control('Seed')[0])
        drain()
        self.b.grab_focus()
        self.echo_key(self.b)  # no wait for an external observer acknowledgement
        drain()
        self.assertEqual(self.a.get_text(), 'ghb')
        self.assertEqual(self.b.get_text(), 'x')

    def test_native_commit_replaces_only_selected_range(self):
        self.focus(self.a)
        self.a.set_text('abc')
        self.a.select_region(1, 2)
        self.echo_key(self.a)
        drain()
        self.assertEqual(self.a.get_text(), 'axc')
        self.assertEqual(self.b.get_text(), '')

    def test_protected_filter_does_not_echo(self):
        self.focus(self.secret)
        before = control('State')[3]
        self.echo_key(self.secret)
        drain()
        self.assertEqual(control('State')[3], before)
        self.assertEqual(self.secret.get_text(), 'x')
        # The native client's fallback commits the unhandled character.
        # No fixture echo is allowed in the protected field.

    def test_focus_change_with_key_reply_outstanding(self):
        self.focus(self.a)
        self.echo_key(self.a)
        self.b.grab_focus()
        drain(1)
        self.assertGreater(control('State')[3], 0)
        observed, engine_same, daemon_same = control('Routing')
        self.assertTrue(observed)
        print('routing_engine_still_original=' + str(engine_same)
              + '; routing_daemon_still_original=' + str(daemon_same), flush=True)
        drain()
        self.assertEqual((self.a.get_text(), self.b.get_text()), ('x', ''))
        # A genuinely outstanding reply is exercised only in --async-client mode.

    def test_z_engine_disconnect_preserves_seeded_draft_on_focus_loss(self):
        self.focus(self.a)
        self.assertTrue(control('Seed')[0])
        drain()
        control('Quit')
        drain(.3)
        self.b.grab_focus()
        drain()
        self.assertEqual(self.a.get_text(), 'ghb')
        self.assertEqual(self.b.get_text(), '')

    def test_native_context_identity_round_trip(self):
        a = self.focus(self.a)[0]
        b = self.focus(self.b)[0]
        again = self.focus(self.a)[0]
        self.assertNotEqual(a, b)
        self.assertEqual(a, again)

    def test_native_focus_loss_commits_seeded_preedit_to_old_widget_once(self):
        self.focus(self.a)
        self.assertTrue(control('Seed')[0])
        drain()
        self.assertEqual(self.a.get_text(), '')
        self.focus(self.b)
        self.assertEqual(self.a.get_text(), 'ghb')
        self.assertEqual(self.b.get_text(), '')
        self.focus(self.a)
        self.assertEqual(self.a.get_text(), 'ghb')

    def test_protected_purpose_and_old_draft_disposition(self):
        self.focus(self.a)
        self.assertTrue(control('Seed')[0])
        drain()
        state = self.focus(self.secret)
        self.assertEqual(state[1], int(IBus.InputPurpose.PASSWORD))
        self.assertFalse(control('Seed')[0])
        self.assertEqual(self.a.get_text(), 'ghb')
        self.assertEqual(self.secret.get_text(), '')


if __name__ == '__main__':
    unittest.main(verbosity=2)
