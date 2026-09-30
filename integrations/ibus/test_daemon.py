"""Actual private daemon routing with a deliberately synthetic client model."""
import os
import time
import unittest
import warnings
import gi

gi.require_version('IBus', '1.0')
from gi.repository import GLib, IBus
from test_control import acknowledge, request

warnings.filterwarnings('ignore', category=DeprecationWarning, module=r'gi\.events')


def drain():
    while GLib.MainContext.default().pending():
        GLib.MainContext.default().iteration(False)


class Client:
    def __init__(self, bus, name, client_commit=False):
        self.bus = bus
        self.context = bus.create_input_context(name)
        self.text = ''
        self.preedit = ''
        self.mode = None
        self.commits = []
        self.client_commit = client_commit
        self.subscription = bus.get_connection().signal_subscribe(
            None, 'org.freedesktop.IBus.InputContext', None,
            self.context.get_object_path(), None, 0, self.signal)
        self.context.set_client_commit_preedit(client_commit)
        self.context.set_capabilities(int(IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.FOCUS))

    def signal(self, connection, sender, path, interface, member, params):
        if member not in ['CommitText', 'UpdatePreeditText', 'UpdatePreeditTextWithMode']:
            return
        text = IBus.Serializable.deserialize_object(params.get_child_value(0).get_variant()).get_text()
        if member == 'CommitText':
            self.commits.append(text)
            self.text += text
        else:
            visible = params.get_child_value(2).get_boolean()
            self.preedit = text if visible else ''
            self.mode = params.get_child_value(3).get_uint32() if params.n_children() > 3 else 0

    def focus(self):
        self.context.focus_in()
        if not self.bus.set_global_engine('typomorph-private-test'):
            raise RuntimeError('Private global engine activation failed')
        deadline = time.monotonic() + 3
        while True:
            engine = self.context.get_engine()
            if engine and engine.get_name() == 'typomorph-private-test':
                break
            if time.monotonic() > deadline:
                raise RuntimeError('Private engine activation timeout')
            drain()
            time.sleep(0.01)
        self.key(0, insert=False)  # fence engine activation

    def refresh(self):
        self.context.set_content_type(int(IBus.InputPurpose.FREE_FORM), 0)
        self.key(0, insert=False)
        if not acknowledge(self.context.get_object_path(), '/field/test'):
            raise RuntimeError('Synthetic field acknowledgement rejected')

    def key(self, value, insert=True):
        handled = self.context.process_key_event(value, value & 255, 0)
        drain()
        if not handled and insert and 32 <= value < 127:
            self.text += chr(value)
        return handled

    def type(self, text):
        for ch in text:
            self.key(ord(ch))
            self.context.process_key_event(ord(ch), ord(ch) & 255, int(IBus.ModifierType.RELEASE_MASK))
            drain()

    def blur(self):
        # Model only; actual GTK/Shell behavior remains to be validated.
        if self.client_commit and self.mode == int(IBus.PreeditFocusMode.COMMIT):
            self.text += self.preedit
            self.preedit = ''
        self.context.focus_out()


class Routing(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        runtime = os.environ.get('XDG_RUNTIME_DIR', '')
        assert os.environ.get('TYPOMORPH_PRIVATE_IBUS_DAEMON') == '1'
        assert runtime.startswith('/tmp/typomorph-ibus-daemon-')
        assert os.environ.get('IBUS_ADDRESS') == 'unix:path=' + runtime + '/bus'
        IBus.init()
        cls.bus = IBus.Bus.new()
        assert cls.bus.is_connected()

    def setUp(self):
        warnings.filterwarnings('ignore', category=DeprecationWarning, module=r'gi\.events')
        self.clients = []

    def client(self, name, client_commit=False):
        client = Client(self.bus, name, client_commit)
        self.clients.append(client)
        return client

    def tearDown(self):
        for client in self.clients:
            self.bus.get_connection().signal_unsubscribe(client.subscription)
            client.context.destroy()
        drain()

    def test_daemon_routes_corrected_word_and_following_input(self):
        a = self.client('synthetic-a')
        a.focus()
        a.refresh()
        a.type('ghbdtn hello ')
        self.assertEqual(a.text, 'привет hello ')

    def test_server_commit_focus_loss_keeps_draft_on_original_context(self):
        a, b = self.client('synthetic-a'), self.client('synthetic-b')
        a.focus(); a.refresh(); a.type('ghb')
        a.blur(); b.focus(); b.refresh(); b.type('new ')
        drain()
        self.assertEqual(a.text, 'ghb')
        self.assertEqual(b.text, 'new ')
        b.blur(); a.focus(); a.refresh(); a.type('x ')
        self.assertEqual(a.text, 'ghbx ')

    def test_client_commit_model_keeps_draft_once(self):
        a, b = self.client('synthetic-a', True), self.client('synthetic-b', True)
        a.focus(); a.refresh(); a.type('ghb')
        a.blur(); b.focus(); b.refresh(); b.type('new ')
        drain()
        self.assertEqual(a.text, 'ghb')
        self.assertEqual(b.text, 'new ')
        self.assertEqual(a.commits, [])  # server must not duplicate client commit

    def test_server_reset_preserves_original_draft_once(self):
        a = self.client('synthetic-reset')
        a.focus(); a.refresh(); a.type('ghb')
        a.context.reset()
        a.key(0, insert=False)
        self.assertEqual(a.text, 'ghb')
        self.assertEqual(a.preedit, '')
        self.assertFalse(a.key(ord('x')))
        self.assertEqual(a.text, 'ghbx')
        self.assertEqual(a.commits, ['ghb'])

    def test_normal_to_normal_without_changed_content_type_stays_suspended(self):
        a, b = self.client('synthetic-a'), self.client('synthetic-b')
        a.focus(); a.refresh(); a.type('ghb')
        a.blur(); b.focus()
        b.context.set_content_type(int(IBus.InputPurpose.FREE_FORM), 0)
        b.key(0, insert=False)
        self.assertFalse(b.key(ord('x')))
        self.assertEqual(a.text, 'ghb')
        self.assertEqual(b.text, 'x')
        self.assertEqual(b.preedit, '')

    def test_draft_cannot_leak_into_password_context(self):
        a, b = self.client('synthetic-a'), self.client('synthetic-password')
        b.context.set_content_type(int(IBus.InputPurpose.PASSWORD), int(IBus.InputHints.PRIVATE))
        a.focus(); a.refresh(); a.type('ghb')
        a.blur(); b.focus(); b.type('ghbdtn ')
        self.assertEqual(a.text, 'ghb')
        self.assertEqual(b.text, 'ghbdtn ')
        self.assertEqual(b.preedit, '')
        self.assertEqual(b.commits, [])

    def test_fresh_field_ack_resumes_same_content_type_in_new_context(self):
        a, b = self.client('synthetic-a'), self.client('synthetic-b')
        a.focus(); a.refresh(); a.type('ghb')
        a.blur(); b.focus()
        b.context.set_content_type(int(IBus.InputPurpose.FREE_FORM), 0)
        b.key(0, insert=False)
        self.assertFalse(b.key(ord('x')))
        self.assertTrue(acknowledge(b.context.get_object_path(), '/field/b'))
        b.type('ghbdtn ')
        self.assertEqual(a.text, 'ghb')
        self.assertEqual(b.text, 'xпривет ')

    def test_old_ack_cannot_reenable_context_after_return(self):
        a, b = self.client('synthetic-a'), self.client('synthetic-b')
        a.focus(); a.refresh()
        ticket = request('Begin', '(ss)', (a.context.get_object_path(), '/field/a'))
        a.blur(); b.focus(); b.blur(); a.focus()
        self.assertFalse(request('Confirm', '(sstuubbb)',
            (a.context.get_object_path(), '/field/a', ticket, 0, 0, True, False, False)))
        self.assertFalse(a.key(ord('x')))
        self.assertEqual(a.text, 'x')

    def test_password_context_has_no_preedit_or_correction(self):
        a = self.client('synthetic-protected')
        a.focus()
        a.context.set_content_type(int(IBus.InputPurpose.PASSWORD), int(IBus.InputHints.PRIVATE))
        a.key(0, insert=False)
        a.type('ghbdtn ')
        self.assertEqual(a.text, 'ghbdtn ')
        self.assertEqual(a.preedit, '')
        self.assertEqual(a.commits, [])


if __name__ == '__main__':
    unittest.main(verbosity=2)
