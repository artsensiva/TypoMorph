"""Synthetic client: validates transport/order, not GTK/Chrome behavior."""
import os
import unittest
import warnings
import gi

gi.require_version('IBus', '1.0')
from gi.repository import Gio, GLib, IBus
from test_control import acknowledge, request


class Protocol(unittest.TestCase):
    def setUp(self):
        assert os.environ.get('TYPOMORPH_PRIVATE_IBUS_TEST') == '1'
        assert os.environ.get('IBUS_ADDRESS') == 'unix:path=/nonexistent/typomorph-isolated'
        self.bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        warnings.filterwarnings('ignore', category=DeprecationWarning, module=r'gi\.events')
        self.events = []
        self.modes = []
        self.subscription = self.bus.signal_subscribe('org.typomorph.PrivateEngine',
            'org.freedesktop.IBus.Engine', None, '/org/typomorph/TestEngine', None,
            Gio.DBusSignalFlags.NONE, self.signal)
        self.call('FocusOutId', '(s)', ('/fixture/previous',))
        self.call('Enable')
        self.call('FocusInId', '(ss)', ('/fixture/a', 'synthetic-client'))
        self.call('SetCapabilities', '(u)', (int(IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.FOCUS),))
        self.content()
        self.events.clear()

    def tearDown(self):
        self.bus.signal_unsubscribe(self.subscription)

    def signal(self, connection, sender, path, interface, member, params):
        if member in ['CommitText', 'UpdatePreeditText']:
            text = IBus.Serializable.deserialize_object(params.get_child_value(0).get_variant()).get_text()
            self.events.append((member, text))
            if member == 'UpdatePreeditText':
                self.modes.append(params.get_child_value(3).get_uint32())
        else:
            self.events.append((member, None))

    def call(self, method, signature=None, args=()):
        if method == 'FocusInId':
            self.current_context = args[0]
        result = self.bus.call_sync('org.typomorph.PrivateEngine', '/org/typomorph/TestEngine',
            'org.freedesktop.IBus.Engine', method,
            GLib.Variant(signature, args) if signature else None,
            None, Gio.DBusCallFlags.NONE, 1000, None)
        # D-Bus reply fences signals sent before it; dispatch locally queued callbacks.
        while GLib.MainContext.default().pending():
            GLib.MainContext.default().iteration(False)
        return result.unpack()

    def content(self, purpose=IBus.InputPurpose.FREE_FORM, hints=0):
        self.set_content(purpose, hints)
        acknowledge(self.current_context, '/field/test', purpose, hints)

    def set_content(self, purpose, hints=0):
        self.bus.call_sync('org.typomorph.PrivateEngine', '/org/typomorph/TestEngine',
            'org.freedesktop.DBus.Properties', 'Set',
            GLib.Variant('(ssv)', ('org.freedesktop.IBus.Engine', 'ContentType',
                                  GLib.Variant('(uu)', (int(purpose), int(hints))))),
            None, Gio.DBusCallFlags.NONE, 1000, None)
        while GLib.MainContext.default().pending():
            GLib.MainContext.default().iteration(False)

    def test_unchanged_content_type_after_focus_does_not_authorize(self):
        self.call('FocusOutId', '(s)', ('/fixture/a',))
        self.call('FocusInId', '(ss)', ('/fixture/b', 'synthetic-client'))
        self.set_content(IBus.InputPurpose.FREE_FORM)
        self.assertFalse(self.key(ord('a')))
        self.assertEqual(self.commits(), [])

    def key(self, value, state=0):
        code = value & 255
        return self.call('ProcessKeyEvent', '(uuu)', (value, code, int(state)))[0]

    def type_word(self, text):
        for ch in text:
            self.assertTrue(self.key(ord(ch)))
            self.assertTrue(self.key(ord(ch), IBus.ModifierType.RELEASE_MASK))

    def commits(self):
        return [text for kind, text in self.events if kind == 'CommitText']

    def test_correction_and_next_word_order(self):
        self.type_word('ghbdtn ')
        self.type_word('hello ')
        self.assertEqual(self.commits(), ['привет ', 'hello '])
        self.assertEqual(set(self.modes), {int(IBus.PreeditFocusMode.COMMIT)})
        self.assertFalse(any(kind == 'DeleteSurroundingText' for kind, _ in self.events))

    def test_focus_round_trip_never_replays_old_draft(self):
        self.type_word('ghb')
        self.call('FocusOutId', '(s)', ('/fixture/a',))
        self.call('FocusInId', '(ss)', ('/fixture/b', 'synthetic-client'))
        self.assertFalse(self.key(ord('x')))  # content type not refreshed
        self.content()
        self.type_word('new ')
        self.call('FocusOutId', '(s)', ('/fixture/b',))
        self.call('FocusInId', '(ss)', ('/fixture/a', 'synthetic-client'))
        self.content()
        self.assertFalse(self.key(IBus.KEY_space))
        self.assertEqual(self.commits(), ['new '])
        # Old draft disposition belongs to the real client and is NOT proven here.

    def test_protected_unknown_and_missing_capabilities_pass_through(self):
        for purpose, hints in [(IBus.InputPurpose.PASSWORD, 0), (IBus.InputPurpose.PIN, 0),
            (IBus.InputPurpose.TERMINAL, 0), (999, 0),
            (IBus.InputPurpose.FREE_FORM, IBus.InputHints.PRIVATE),
            (IBus.InputPurpose.FREE_FORM, IBus.InputHints.HIDDEN_TEXT)]:
            self.content(purpose, hints)
            self.events.clear()
            self.assertFalse(self.key(ord('a')))
            self.assertEqual(self.events, [])
        self.content()
        self.call('SetCapabilities', '(u)', (0,))
        self.assertFalse(self.key(ord('a')))

    def test_reset_and_disable_invalidate_permission_and_draft(self):
        for action in ['Reset', 'Disable']:
            self.call('Enable')
            self.content()
            self.type_word('ghb')
            self.call(action)
            self.assertFalse(self.key(ord('x')))
            self.call('Enable')
            self.content()
            self.assertFalse(self.key(IBus.KEY_space))
        self.assertEqual(self.commits(), [])

    def test_backspace_and_shortcut_preserve_order(self):
        self.type_word('abc')
        self.assertTrue(self.key(IBus.KEY_BackSpace))
        self.assertFalse(self.key(ord('c'), IBus.ModifierType.CONTROL_MASK))
        self.assertEqual(self.commits(), ['ab'])

    def test_capability_loss_cannot_resume_old_draft(self):
        self.type_word('ghb')
        self.call('SetCapabilities', '(u)', (0,))
        self.call('SetCapabilities', '(u)', (int(IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.FOCUS),))
        self.assertFalse(self.key(ord('x')))
        self.content()
        self.assertFalse(self.key(IBus.KEY_space))
        self.assertEqual(self.commits(), [])

    def begin(self, field='/field/test'):
        return request('Begin', '(ss)', (self.current_context, field))

    def confirm(self, ticket, field='/field/test', editable=True, selected=False, composing=False):
        return request('Confirm', '(sstuubbb)', (self.current_context, field, ticket,
                       int(IBus.InputPurpose.FREE_FORM), 0, editable, selected, composing))

    def test_late_ack_after_focus_round_trip_is_rejected(self):
        ticket = self.begin()
        self.call('FocusOutId', '(s)', ('/fixture/a',))
        self.call('FocusInId', '(ss)', ('/fixture/b', 'synthetic-client'))
        self.call('FocusOutId', '(s)', ('/fixture/b',))
        self.call('FocusInId', '(ss)', ('/fixture/a', 'synthetic-client'))
        self.assertFalse(self.confirm(ticket))
        self.assertFalse(self.key(ord('x')))
        self.assertTrue(self.confirm(self.begin()))
        self.type_word('ghbdtn ')
        self.assertEqual(self.commits(), ['привет '])

    def test_same_ibus_context_different_field_invalidates_old_ack(self):
        first = self.begin('/field/a')
        self.begin('/field/b')
        current = self.begin('/field/a')
        self.assertFalse(self.confirm(first, '/field/a'))
        self.assertFalse(self.confirm(current, '/field/b'))
        self.assertTrue(self.confirm(current, '/field/a'))
        self.assertFalse(self.confirm(current, '/field/a'))  # one-shot response
        self.type_word('ok ')
        self.assertEqual(self.commits(), ['ok '])

    def test_input_before_ack_invalidates_the_pending_observation(self):
        ticket = self.begin()
        self.assertFalse(self.key(ord('x')))
        self.assertFalse(self.confirm(ticket))
        self.assertTrue(self.confirm(self.begin()))
        self.type_word('ok ')
        self.assertEqual(self.commits(), ['ok '])

    def test_reset_disable_and_revoke_reject_late_ack(self):
        for action in ['Reset', 'Disable', 'Revoke']:
            self.call('Enable')
            ticket = self.begin()
            if action == 'Revoke':
                self.assertTrue(request('Revoke', '(s)', (self.current_context,)))
            else:
                self.call(action)
            self.call('Enable')
            self.assertFalse(self.confirm(ticket))
            self.assertFalse(self.key(ord('x')))

    def test_selected_readonly_composing_and_protected_ack_refused(self):
        for flags in [(False, False, False), (True, True, False), (True, False, True)]:
            self.assertFalse(self.confirm(self.begin(), '/field/test', *flags))
            self.assertFalse(self.key(ord('x')))
        self.set_content(IBus.InputPurpose.PASSWORD, IBus.InputHints.PRIVATE)
        self.assertFalse(self.confirm(self.begin()))  # observer cannot override veto
        self.assertFalse(self.key(ord('x')))
        self.assertEqual(self.commits(), [])

    # These characterize known unsafe schedules, not production safety guarantees.
    # A hidden field transition is deliberately not sent to the engine: the
    # current input protocol has no per-key field identity to distinguish it.
    def test_known_gap_delayed_field_notice_allows_stale_permission(self):
        self.type_word('ghb')
        # Model target A -> B with the same IBus context; notice is delayed.
        self.events.clear()
        self.type_word('dtn ')
        self.assertEqual(self.commits(), ['привет '])
        # A late notification cannot retract the already emitted commit.
        self.begin('/field/b')
        self.assertFalse(self.key(ord('x')))
        self.assertEqual(self.commits(), ['привет '])

    def test_known_gap_unreported_round_trip_keeps_old_permission(self):
        self.type_word('ghb')
        # Model A -> B -> A before either notification reaches the engine.
        # Matching endpoint identity does not prove continuous field ownership.
        self.type_word('dtn ')
        self.assertEqual(self.commits(), ['привет '])
        self.begin('/field/b')
        current = self.begin('/field/a')
        self.assertTrue(self.confirm(current, '/field/a'))
        self.assertFalse(self.key(IBus.KEY_space))

    def test_known_gap_revoke_does_not_resolve_client_preedit(self):
        self.type_word('ghb')
        self.assertEqual([value for kind, value in self.events
                          if kind == 'UpdatePreeditText'][-1], 'ghb')
        self.events.clear()
        self.assertTrue(request('Revoke', '(s)', (self.current_context,)))
        # Dispatch pending messages after the control endpoint reply.
        self.assertFalse(self.key(ord('x')))
        self.assertEqual(self.events, [])
        self.assertTrue(self.confirm(self.begin()))
        self.assertFalse(self.key(IBus.KEY_space))
        self.assertEqual(self.commits(), [])
        # Engine draft was cleared, but no client disposition was requested.
        # This does not establish whether a real client preserves or loses it.

    def test_long_input_is_not_truncated(self):
        self.type_word('a' * 80 + ' ')
        self.assertEqual(''.join(self.commits()), 'a' * 80 + ' ')


if __name__ == '__main__':
    unittest.main(verbosity=2)
