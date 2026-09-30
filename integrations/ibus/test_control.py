"""Synthetic observer only. Not an authenticated production eligibility service."""
import os
from gi.repository import Gio, GLib

NAME = 'org.typomorph.TestEligibility'
PATH = '/org/typomorph/TestEligibility'
XML = '''<node><interface name="org.typomorph.TestEligibility">
<method name="Begin"><arg type="s" direction="in"/><arg type="s" direction="in"/><arg type="t" direction="out"/></method>
<method name="Confirm"><arg type="s" direction="in"/><arg type="s" direction="in"/><arg type="t" direction="in"/>
<arg type="u" direction="in"/><arg type="u" direction="in"/><arg type="b" direction="in"/><arg type="b" direction="in"/><arg type="b" direction="in"/><arg type="b" direction="out"/></method>
<method name="Revoke"><arg type="s" direction="in"/><arg type="b" direction="out"/></method>
</interface></node>'''


class ObserverFixture:
    def __init__(self, engines):
        if not (os.environ.get('TYPOMORPH_PRIVATE_IBUS_TEST') == '1' or
                os.environ.get('TYPOMORPH_PRIVATE_IBUS_DAEMON') == '1'):
            raise RuntimeError('Private tests only')
        self.engines = engines
        self.bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        self.bus.call_sync('org.freedesktop.DBus', '/org/freedesktop/DBus',
                          'org.freedesktop.DBus', 'RequestName', GLib.Variant('(su)', (NAME, 4)),
                          None, Gio.DBusCallFlags.NONE, 1000, None)
        self.registration = self.bus.register_object(PATH,
            Gio.DBusNodeInfo.new_for_xml(XML).interfaces[0], self.call, None, None)

    def call(self, connection, sender, path, interface, method, params, invocation):
        args = params.unpack()
        matches = [e for e in self.engines if e._focus == args[0] and e._enabled]
        engine = matches[0] if len(matches) == 1 else None
        if method == 'Begin':
            if engine:
                engine.clear()
            value = engine.eligibility.begin(*args) if engine else 0
            invocation.return_value(GLib.Variant('(t)', (value,)))
        elif method == 'Confirm':
            value = engine.confirm_field(*args) if engine else False
            invocation.return_value(GLib.Variant('(b)', (value,)))
        elif method == 'Revoke':
            if engine:
                engine.clear()
            invocation.return_value(GLib.Variant('(b)', (engine is not None,)))


def request(method, signature, args):
    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    return bus.call_sync(NAME, PATH, NAME, method, GLib.Variant(signature, args),
                         None, Gio.DBusCallFlags.NONE, 1000, None).unpack()[0]


def acknowledge(context, field, purpose=0, hints=0, editable=True, selected=False, composing=False):
    generation = request('Begin', '(ss)', (context, field))
    return request('Confirm', '(sstuubbb)',
                   (context, field, generation, int(purpose), int(hints), editable, selected, composing))
