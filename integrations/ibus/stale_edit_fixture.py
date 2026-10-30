"""Test-only IBus engine for stale text-input-v3 edit characterization.

It records the surrounding text the client reports, keeps one explicit snapshot,
and on request sends a fixed deletion plus commit computed from that snapshot,
whatever the field holds by then. It is NOT TypoMorph eligibility, analysis or
correction; it processes no key events and refuses any non-ordinary field.
Fixed synthetic strings only: no arbitrary replacement is accepted.
"""
import os
import sys
from pathlib import Path

import gi

if os.environ.get('TYPOMORPH_PRIVATE_WAYLAND') != '1':
    raise SystemExit('Private Wayland session required')
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'gnome' / 'tests'))
from run_wayland_probe import check_environment
check_environment()
gi.require_version('IBus', '1.0')
from gi.repository import Gio, GLib, IBus

NAME = 'org.typomorph.StaleFixture'
PATH = '/org/typomorph/StaleFixture'
ENGINE = 'typomorph-stale-fixture'
WORD = 'ghbdtn'
REPLACEMENT = 'привет'
XML = '''<node><interface name="org.typomorph.StaleFixture">
<method name="State"><arg direction="out" type="s"/><arg direction="out" type="s"/>
<arg direction="out" type="i"/><arg direction="out" type="i"/><arg direction="out" type="u"/>
<arg direction="out" type="u"/></method>
<method name="Snapshot"><arg direction="out" type="b"/></method>
<method name="SendFromSnapshot"><arg direction="out" type="b"/></method>
<method name="Quit"/>
</interface></node>'''


LOG = Path(os.environ['XDG_RUNTIME_DIR'], 'stale-engine-events.log')


def note(event):
    """Event names only, inside the private session directory; never text."""
    with LOG.open('a') as stream:
        stream.write(event + '\n')


class StaleEngine(IBus.Engine):
    def __init__(self, connection, path):
        super().__init__(connection=connection, object_path=path, engine_name=ENGINE,
                         has_focus_id=True)
        self.focus = ''
        self.text = ''
        self.cursor = 0
        self.anchor = 0
        self.updates = 0
        self.snapshot = None

    def do_focus_in_id(self, context, client):
        self.focus = context
        note('focus_in')
        # Asking once emits the require-surrounding-text signal; passing the
        # active-surrounding-text property at construction would suppress it.
        self.get_surrounding_text()

    def do_enable(self):
        note('enable')
        self.get_surrounding_text()

    def do_focus_out_id(self, context):
        self.focus = ''

    def do_set_surrounding_text(self, text, cursor_index, anchor_pos):
        self.text = text.get_text()
        self.cursor = cursor_index
        self.anchor = anchor_pos
        self.updates += 1
        note('set_surrounding')

    def do_process_key_event(self, keyval, keycode, state):
        return False  # Never consume or originate typing.


def main():
    IBus.init()
    bus = IBus.Bus.new()
    assert bus.is_connected()
    engines = []
    factory = IBus.Factory.new(bus.get_connection())

    def create(factory, name):
        assert name == ENGINE
        engine = StaleEngine(bus.get_connection(), '/org/typomorph/StaleEngine' + str(len(engines)))
        engines.append(engine)
        return engine
    factory.connect('create-engine', create)
    component = IBus.Component.new(NAME, 'Stale edit stimulus', '0', 'MIT', 'TypoMorph', '', '', '')
    component.add_engine(IBus.EngineDesc.new(ENGINE, 'Stale edit fixture',
        'Synthetic stale-edit stimulus only', 'en', 'MIT', 'TypoMorph', '', 'us'))
    assert bus.register_component(component)
    session = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    session.call_sync('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
        'RequestName', GLib.Variant('(su)', (NAME, 4)), None, 0, 1000, None)

    def control(c, sender, path, interface, method, params, invocation):
        active = [e for e in engines if e.focus]
        engine = active[0] if len(active) == 1 else None
        ordinary = engine is not None and engine.get_content_type()[0] == 0
        if method == 'State':
            invocation.return_value(GLib.Variant('(ssiiuu)', (
                engine.focus if engine else '', engine.text if engine else '',
                engine.cursor if engine else -1, engine.anchor if engine else -1,
                engine.updates if engine else 0, engine.get_content_type()[0] if engine else 999)))
        elif method == 'Snapshot':
            if ordinary:
                engine.snapshot = (engine.text, engine.cursor)
            invocation.return_value(GLib.Variant('(b)', (bool(ordinary),)))
        elif method == 'SendFromSnapshot':
            sent = False
            if ordinary and engine.snapshot is not None:
                text, cursor = engine.snapshot
                chars = list(text)
                # The request is derived only from the snapshot, never the current field.
                if ''.join(chars[max(0, cursor - len(WORD)):cursor]) == WORD:
                    engine.delete_surrounding_text(-len(WORD), len(WORD))
                    engine.commit_text(IBus.Text.new_from_string(REPLACEMENT))
                    sent = True
            invocation.return_value(GLib.Variant('(b)', (sent,)))
        elif method == 'Quit':
            invocation.return_value(GLib.Variant('()', ()))
            GLib.idle_add(lambda: (loop.quit(), False)[1])
    session.register_object(PATH, Gio.DBusNodeInfo.new_for_xml(XML).interfaces[0], control, None, None)
    loop = GLib.MainLoop()
    print('READY', flush=True)
    loop.run()


if __name__ == '__main__':
    main()
