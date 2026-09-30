"""Native-client lifecycle stimulus only; NOT TypoMorph eligibility or correction."""
import gi
import time
import os
if os.environ.get('TYPOMORPH_PRIVATE_WAYLAND') == '1':
    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'gnome' / 'tests'))
    from run_wayland_probe import check_environment
else:
    from run_gtk_tests import check_environment
check_environment()
gi.require_version('IBus', '1.0')
from gi.repository import IBus, Gio, GLib

NAME = 'org.typomorph.GtkFixture'
PATH = '/org/typomorph/GtkFixture'
XML = '''<node><interface name="org.typomorph.GtkFixture">
<method name="State"><arg direction="out" type="s"/><arg direction="out" type="u"/><arg direction="out" type="u"/><arg direction="out" type="u"/></method>
<method name="Seed"><arg direction="out" type="b"/></method>
<method name="Quit"/>
<signal name="KeyStarted"/>
<method name="Routing"><arg direction="out" type="b"/><arg direction="out" type="b"/><arg direction="out" type="b"/></method>
</interface></node>'''


class LifecycleEngine(IBus.Engine):
    def __init__(self, connection, path):
        super().__init__(connection=connection, object_path=path,
            engine_name='typomorph-gtk-fixture', has_focus_id=True, active_surrounding_text=False)
        self.focus = ''
        self.keys = 0
        self.echoes = 0
        self.routing = (False, False, False)

    def do_focus_in_id(self, context, client):
        self.focus = context

    def do_focus_out_id(self, context):
        self.focus = ''

    def do_process_key_event(self, keyval, keycode, state):
        self.keys += 1
        # A fixed key echo exercises native commit delivery, not correction.
        # Protected input is never inspected by the echo branch.
        purpose, hints = self.get_content_type()
        if purpose != 0 or hints & int(IBus.InputHints.PRIVATE | IBus.InputHints.HIDDEN_TEXT):
            return False
        if state == 0 and keyval == ord('x'):
            origin = self.focus
            self.key_started()
            time.sleep(.2 if os.environ.get('TYPOMORPH_PRIVATE_WAYLAND') == '1' else .05)
            # Diagnostic query only: separate from commit, not a safety check.
            current = self.routing_bus.current_input_context()
            self.routing = (True, self.focus == origin, current == origin)
            self.echoes += 1
            self.commit_text(IBus.Text.new_from_string('x'))
            return True
        return False


def main():
    IBus.init()
    bus = IBus.Bus.new()
    assert bus.is_connected()
    engines = []
    factory = IBus.Factory.new(bus.get_connection())
    def create(factory, name):
        assert name == 'typomorph-gtk-fixture'
        engine = LifecycleEngine(bus.get_connection(), '/org/typomorph/GtkEngine'+str(len(engines)))
        engine.routing_bus = bus
        def started():
            session.emit_signal(None, PATH, NAME, 'KeyStarted', GLib.Variant('()', ()))
            session.flush_sync(None)
        engine.key_started = started
        engines.append(engine)
        return engine
    factory.connect('create-engine', create)
    component = IBus.Component.new(NAME, 'GTK lifecycle stimulus', '0', 'MIT', 'TypoMorph', '', '', '')
    component.add_engine(IBus.EngineDesc.new('typomorph-gtk-fixture', 'GTK fixture',
        'Synthetic preedit only', 'en', 'MIT', 'TypoMorph', '', 'us'))
    assert bus.register_component(component)
    session = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    session.call_sync('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
        'RequestName', GLib.Variant('(su)', (NAME, 4)), None, 0, 1000, None)
    def control(c, sender, path, interface, method, params, invocation):
        active = [e for e in engines if e.focus]
        engine = active[0] if len(active)==1 else None
        if method == 'State':
            invocation.return_value(GLib.Variant('(suuu)', (engine.focus if engine else '',
                engine.get_content_type()[0] if engine else 999, sum(e.keys for e in engines), sum(e.echoes for e in engines))))
        elif method == 'Routing':
            observed = [e.routing for e in engines if e.routing[0]]
            invocation.return_value(GLib.Variant('(bbb)', observed[-1] if observed else (False, False, False)))
        elif method == 'Quit':
            invocation.return_value(GLib.Variant('()', ()))
            GLib.idle_add(lambda: (loop.quit(), False)[1])
        elif method == 'Seed':
            safe = engine is not None and engine.get_content_type()[0] == 0
            if safe:
                # Explicit injected test stimulus, never an eligibility decision.
                engine.update_preedit_text_with_mode(IBus.Text.new_from_string('ghb'), 3, True,
                                                    IBus.PreeditFocusMode.COMMIT)
            invocation.return_value(GLib.Variant('(b)', (safe,)))
    registration = session.register_object(PATH, Gio.DBusNodeInfo.new_for_xml(XML).interfaces[0], control, None, None)
    loop = GLib.MainLoop()
    print('READY', flush=True)
    loop.run()


if __name__ == '__main__':
    main()
