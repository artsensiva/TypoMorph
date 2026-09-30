"""Two coordinated read-only snapshots. No text interfaces, input devices or mutations."""
import signal
import sys
import time
from collections import deque
import gi

gi.require_version('IBus', '1.0')
from gi.repository import Gio, GLib, IBus


def snapshot():
    signal.alarm(5)
    try:
        bus = IBus.Bus.new()
        if not bus.is_connected():
            return None
        ibus_id = (bus.get_connection().get_guid(), bus.current_input_context())
        if not ibus_id[1]:
            return None
        session = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        address = session.call_sync('org.a11y.Bus', '/org/a11y/bus', 'org.a11y.Bus',
            'GetAddress', None, None, Gio.DBusCallFlags.NONE, 500, None).unpack()[0]
        a11y = Gio.DBusConnection.new_for_address_sync(address,
            Gio.DBusConnectionFlags.AUTHENTICATION_CLIENT | Gio.DBusConnectionFlags.MESSAGE_BUS_CONNECTION,
            None, None)

        def call(obj, method):
            return a11y.call_sync(*obj, 'org.a11y.atspi.Accessible', method,
                None, None, Gio.DBusCallFlags.NONE, 150, None).unpack()[0]

        def field():
            deadline = time.monotonic() + 1
            root = ('org.a11y.atspi.Registry', '/org/a11y/atspi/accessible/root')
            queue = deque((tuple(o), None, 0) for o in call(root, 'GetChildren'))
            seen = set()
            found = None
            while queue:
                if len(seen) >= 256 or len(queue) >= 256 or time.monotonic() > deadline:
                    return None
                obj, window, depth = queue.popleft()
                if obj[1] == '/org/a11y/atspi/null':
                    continue
                if obj in seen or depth > 32:
                    return None
                seen.add(obj)
                role, states = call(obj, 'GetRole'), call(obj, 'GetState')
                def has(bit):
                    return len(states) > bit // 32 and bool(states[bit // 32] & (1 << (bit % 32)))
                if has(6):
                    continue
                if role in (16, 23, 69):
                    if not has(1):
                        continue
                    window = obj
                if has(12):
                    if (found is not None or window is None or role not in (61, 79)
                        or has(43) or not all(has(bit) for bit in (7, 24, 25, 30))):
                        return None
                    found = (a11y.get_guid(), obj, window)
                    continue
                queue.extend((tuple(child), window, depth + 1) for child in call(obj, 'GetChildren'))
            return found

        first = field()
        if first is None or first != field():
            return None
        if ibus_id != (bus.get_connection().get_guid(), bus.current_input_context()):
            return None
        return first, ibus_id
    except Exception:
        return None
    finally:
        signal.alarm(0)


def timeout(signum, frame):
    print('metadata_timeout', flush=True)
    raise SystemExit(1)


def main():
    if sys.argv[1:] != ['--read-only']:
        raise SystemExit('Use --read-only only for a coordinated two-field check')
    signal.signal(signal.SIGALRM, timeout)
    time.sleep(45)
    first = snapshot()
    if first is None:
        print('baseline_unavailable; no binding conclusion', flush=True)
        return 1
    print('baseline_ready; second snapshot in 120 seconds', flush=True)
    time.sleep(120)
    second = snapshot()
    if second is None:
        print('second_snapshot_unavailable; no binding conclusion', flush=True)
        return 1
    field_changed = first[0] != second[0]
    context_same = first[1] == second[1]
    print('accessible_field_changed=' + str(field_changed), flush=True)
    print('ibus_context_unchanged=' + str(context_same), flush=True)
    print('binding_not_proven; paired snapshots do not establish authoritative identity', flush=True)
    return 0 if field_changed else 2


if __name__ == '__main__':
    raise SystemExit(main())
