"""Private-bus synthetic prototype; never registers with the desktop IBus daemon."""
import os
import gi
from eligibility import Eligibility

gi.require_version('IBus', '1.0')
from gi.repository import Gio, GLib, IBus


class Prototype(IBus.Engine):
    """Only the synthetic test pair is corrected; no production classifier yet."""
    def __init__(self, connection, object_path="/org/typomorph/TestEngine"):
        super().__init__(connection=connection, object_path=object_path,
                         engine_name='typomorph-private-test', has_focus_id=True,
                         active_surrounding_text=False)
        self.eligibility = Eligibility()
        self._focus = None
        self._enabled = False
        self._capabilities = 0
        self._draft = ''
        self._pressed = set()

    def clear(self):
        self._draft = ''
        self._pressed.clear()
        self.eligibility.invalidate()

    def do_focus_in_id(self, object_path, client):
        self.clear()
        self._focus = object_path
        self.eligibility.focus(object_path)

    def do_focus_out_id(self, object_path):
        self.clear()
        self._focus = None
        self.eligibility.focus(None)
        # No commit here: routing may already point at a different field.
        # A real client must resolve COMMIT-mode preedit on the old target.

    def do_enable(self):
        self._enabled = True

    def do_disable(self):
        self.clear()
        self._enabled = False

    def do_reset(self):
        self.clear()

    def do_set_capabilities(self, caps):
        self._capabilities = caps
        required = int(IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.FOCUS)
        if caps & required != required:
            self.clear()

    @staticmethod
    def safe_type(purpose, hints):
        private = int(IBus.InputHints.PRIVATE | IBus.InputHints.HIDDEN_TEXT)
        return purpose == int(IBus.InputPurpose.FREE_FORM) and not hints & private

    def do_set_content_type(self, purpose, hints):
        # Cached content type may veto, but never grants a new focus permission.
        if not self.safe_type(purpose, hints):
            self.clear()

    def confirm_field(self, context, field, generation, purpose, hints, editable, selected, composing):
        actual_purpose, actual_hints = self.get_content_type()
        safe = (self._enabled and self._focus == context and editable and not selected
                and not composing and self.safe_type(purpose, hints)
                and self.safe_type(actual_purpose, actual_hints))
        return self.eligibility.confirm(context, field, generation, safe)

    def show(self):
        self.update_preedit_text_with_mode(IBus.Text.new_from_string(self._draft),
                                          len(self._draft), bool(self._draft),
                                          IBus.PreeditFocusMode.COMMIT)

    def finish(self, suffix='', correct=False):
        value = self._draft
        if correct and value == 'ghbdtn':
            value = 'привет'  # deliberately fixed synthetic fixture
        self._draft = ''
        self.show()
        if value or suffix:
            self.commit_text(IBus.Text.new_from_string(value + suffix))

    def do_process_key_event(self, keyval, keycode, state):
        required = int(IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.FOCUS)
        if not (self._enabled and self._focus and self.eligibility.permitted and
                self._capabilities & required == required):
            self.eligibility.invalidate()
            return False
        if state & int(IBus.ModifierType.RELEASE_MASK):
            consumed = keycode in self._pressed
            self._pressed.discard(keycode)
            return consumed
        modifiers = int(IBus.ModifierType.CONTROL_MASK | IBus.ModifierType.MOD1_MASK |
                        IBus.ModifierType.SUPER_MASK | IBus.ModifierType.SHIFT_MASK |
                        IBus.ModifierType.LOCK_MASK)
        if state & modifiers:
            self.finish()
            return False
        if keyval == IBus.KEY_space:
            if not self._draft:
                return False
            self.finish(' ', correct=True)
        elif keyval == IBus.KEY_BackSpace and self._draft:
            self._draft = self._draft[:-1]
            self.show()
        elif ord('a') <= keyval <= ord('z'):
            if len(self._draft) == 32:
                self.finish()  # preserve a long word, do not truncate it
            self._draft += chr(keyval)
            self.show()
        else:
            self.finish()
            return False
        self._pressed.add(keycode)
        return True


def main():
    if os.environ.get('TYPOMORPH_PRIVATE_IBUS_TEST') != '1' or os.environ.get('DISPLAY') or os.environ.get('WAYLAND_DISPLAY'):
        raise SystemExit('Use run_tests.py; desktop execution is refused')
    if os.environ.get('IBUS_ADDRESS') != 'unix:path=/nonexistent/typomorph-isolated':
        raise SystemExit('Desktop IBus must be disabled in this test')
    connection = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    connection.call_sync('org.freedesktop.DBus', '/org/freedesktop/DBus',
                         'org.freedesktop.DBus', 'RequestName',
                         GLib.Variant('(su)', ('org.typomorph.PrivateEngine', 4)),
                         None, Gio.DBusCallFlags.NONE, 1000, None)
    engine = Prototype(connection)
    from test_control import ObserverFixture
    observer = ObserverFixture([engine])
    print('READY', flush=True)
    GLib.MainLoop().run()


if __name__ == '__main__':
    main()
