"""Bounded metadata-only GTK focus observer; never an eligibility authority."""
import os
import sys
import time
import signal
from gi.repository import Gio, GLib
from probe_field_binding import snapshot, timeout

FLAGS = Gio.DBusConnectionFlags.AUTHENTICATION_CLIENT | Gio.DBusConnectionFlags.MESSAGE_BUS_CONNECTION
IFACE = 'org.a11y.atspi.Event.Object'
EVENT = 'object:state-changed:focused'


def connect(address):
    return Gio.DBusConnection.new_for_address_sync(address, FLAGS, None, None)


def subscribe(connection, sender, callback):
    if not sender.startswith(':'):
        raise ValueError('unique provider required')
    return connection.signal_subscribe(sender, IFACE, 'StateChanged', None,
        'focused', Gio.DBusSignalFlags.NONE, callback)


def valid(parameters):
    if parameters.get_type_string() != '(siiva{sv})':
        return False
    detail, enabled, unused, payload, properties = parameters.unpack()
    return (detail == 'focused' and enabled in (0, 1) and unused == 0
            and payload == '0' and not properties)


def call(c, dest, path, interface, method, args=None):
    return c.call_sync(dest, path, interface, method, args, None,
                       Gio.DBusCallFlags.NONE, 500, None)


def self_test():
    # Only the runner's explicitly supplied private address is accepted.
    address = os.environ.get('TYPOMORPH_FOCUS_TEST_ADDRESS')
    if not address or address != os.environ.get('DBUS_SESSION_BUS_ADDRESS'):
        return 1
    receiver, provider, other = [connect(address) for _ in range(3)]
    received = []
    token = subscribe(receiver, provider.get_unique_name(),
                      lambda c, s, p, i, m, v: received.append((p, valid(v))))
    call(receiver, 'org.freedesktop.DBus', '/org/freedesktop/DBus',
         'org.freedesktop.DBus', 'GetId')  # AddMatch round-trip barrier
    def emit(c, member='StateChanged', detail='focused'):
        c.emit_signal(None, '/fixture', IFACE, member,
            GLib.Variant('(siiva{sv})', (detail, 1, 0, GLib.Variant('s', '0'), {})))
        c.flush_sync(None)
    emit(provider)
    emit(other)
    emit(provider, detail='selected')
    emit(provider, member='TextChanged')
    loop = GLib.MainLoop()
    GLib.timeout_add(200, lambda: (loop.quit(), False)[1])
    loop.run()
    assert received == [('/fixture', True)], 'subscription leaked unrelated signals'
    assert not valid(GLib.Variant('(siiva{sv})', ('focused', 1, 0, GLib.Variant('s', 'unexpected'), {})))
    receiver.signal_unsubscribe(token)
    for c in (receiver, provider, other):
        c.close_sync(None)
    print('PASS: selected provider/focus only; unrelated sender, state and text event excluded; unexpected payload rejected')
    return 0


def live(prepare_seconds=45):
    signal.signal(signal.SIGALRM, timeout)
    time.sleep(prepare_seconds)
    baseline = snapshot()
    if baseline is None:
        print('baseline_unavailable')
        return 1
    sender, field_path = baseline[0][1]
    session = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    address = call(session, 'org.a11y.Bus', '/org/a11y/bus', 'org.a11y.Bus', 'GetAddress').unpack()[0]
    c = connect(address)
    registered = False
    token = None
    try:
        if c.get_guid() != baseline[0][0]:
            raise ValueError()
        app = call(c, sender, field_path, 'org.a11y.atspi.Accessible', 'GetApplication').unpack()[0]
        if app[0] != sender:
            raise ValueError()
        def prop(name):
            return call(c, *app, 'org.freedesktop.DBus.Properties', 'Get',
                GLib.Variant('(ss)', ('org.a11y.atspi.Application', name))).unpack()[0]
        # Reviewed publisher version only; do not generalize to other toolkits.
        if prop('ToolkitName') != 'GTK' or prop('Version') != '4.22.4':
            print('reviewed_provider_unavailable')
            return 1
        counts = {'gained': 0, 'lost': 0, 'new_field': False, 'invalid': False}
        loop = GLib.MainLoop()
        def event(connection, source, path, interface, member, parameters):
            if not valid(parameters):
                counts['invalid'] = True
                loop.quit()
                return
            enabled = parameters.get_child_value(1).get_int32()
            counts['gained' if enabled else 'lost'] += 1
            if enabled and path != field_path:
                counts['new_field'] = True
            if counts['gained'] + counts['lost'] >= 128:
                loop.quit()
        token = subscribe(c, sender, event)
        call(c, 'org.a11y.atspi.Registry', '/org/a11y/atspi/registry',
             'org.a11y.atspi.Registry', 'RegisterEvent', GLib.Variant('(sass)', (EVENT, [], sender)))
        registered = True
        print('observer_ready; switch to empty search field; 120 seconds', flush=True)
        GLib.timeout_add_seconds(120, lambda: (loop.quit(), False)[1])
        loop.run()
        print('focus_gained_count=' + str(counts['gained']))
        print('focus_lost_count=' + str(counts['lost']))
        print('different_accessible_focused=' + str(counts['new_field']))
        print('unexpected_payload=' + str(counts['invalid']))
        print('binding_not_proven; no input ordering or eligibility conclusion')
        return 0 if counts['new_field'] and not counts['invalid'] else 2
    except Exception:
        print('metadata_observer_unavailable')
        return 1
    finally:
        if token is not None:
            c.signal_unsubscribe(token)
        if registered:
            try:
                call(c, 'org.a11y.atspi.Registry', '/org/a11y/atspi/registry',
                     'org.a11y.atspi.Registry', 'DeregisterEvent', GLib.Variant('(ss)', (EVENT, sender)))
            except Exception:
                pass  # Closing the dedicated connection also removes registration.
        c.close_sync(None)


if __name__ == '__main__':
    if sys.argv[1:] == ['--private-test']:
        raise SystemExit(self_test())
    if sys.argv[1:] == ['--read-only']:
        raise SystemExit(live())
    if sys.argv[1:] == ['--read-only', '--long-preparation']:
        raise SystemExit(live(90))
    raise SystemExit('Use --read-only for a coordinated session, or --private-test on an isolated bus')
