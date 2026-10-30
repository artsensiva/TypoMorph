"""Stale text-input-v3 edit characterization on an unmodified GTK4 client.

Private compositor, private IBus and fixed synthetic text only. A test-only IBus
engine takes a snapshot of the field, the harness then changes the field (or
focus), and the engine sends a deletion plus commit computed from the snapshot.
Each case records what the stock client did. An outcome that is neither the
untouched field nor the exact intended replacement is corruption and a FAIL.
A finite fixture that happens not to corrupt is characterization, not a safety pass.
"""
import fcntl
import os
import re
import socket
import struct
import sys
import termios
import time
from pathlib import Path

import gi

gi.require_version('Gtk', '4.0')
gi.require_version('IBus', '1.0')
from gi.repository import Gtk, Gio, GLib, IBus

INITIAL = 'xx ghbdtn'
OTHER = 'keep'
WORD, REPLACEMENT = 'ghbdtn', 'привет'
CORRECT = 'xx привет'
# One batch as Mutter delivers it: delete_surrounding_text (16 bytes), a null
# preedit_string (20 bytes), commit_string for 12 bytes padded to 16 (28 bytes),
# and done (12 bytes).
EXPECTED_BATCH_BYTES = 16 + 20 + 28 + 12
TRACE = Path(os.environ.get('TYPOMORPH_WAYLAND_TRACE', ''))


def drain():
    context = GLib.MainContext.default()
    while context.pending():
        context.iteration(False)


def wait_for(condition, timeout=5.0):
    """Condition-driven wait with a deadline; never used to manufacture a race."""
    deadline = time.monotonic() + timeout
    while True:
        drain()
        if condition():
            return True
        if time.monotonic() >= deadline:
            return False
        time.sleep(.001)


def trace_events():
    """(is_request, name, args) for zwp_text_input_v3 lines in the client trace."""
    events = []
    for line in TRACE.read_text(errors='replace').splitlines():
        match = re.search(r'(->\s+)?zwp_text_input_v3[@#]\d+\.(\w+)\((.*)\)', line)
        if match:
            events.append((bool(match.group(1)), match.group(2), match.group(3)))
    return events


def diagnostics():
    """Content-free protocol counts, for explaining a harness precondition failure."""
    counts = {}
    for is_request, name, _ in trace_events():
        key = ('->' if is_request else '<-') + name
        counts[key] = counts.get(key, 0) + 1
    log = Path(os.environ['XDG_RUNTIME_DIR'], 'stale-engine-events.log')
    events = log.read_text().split() if log.exists() else []
    sequence = [('->' if r else '<-') + n + ('(' + a[:48] + ')' if n in ('set_surrounding_text', 'done', 'enable', 'disable', 'commit') else '')
                for r, n, a in trace_events()]
    return 'sequence=' + repr(sequence) + '; engine_events=' + repr(events)


def find_wayland_fd(shell_pid):
    found = []
    for name in os.listdir('/proc/self/fd'):
        fd = int(name)
        try:
            duplicate = socket.fromfd(fd, socket.AF_UNIX, socket.SOCK_STREAM)
        except OSError:
            continue
        try:
            pid = struct.unpack('3i', duplicate.getsockopt(
                socket.SOL_SOCKET, socket.SO_PEERCRED, struct.calcsize('3i')))[0]
        except OSError:
            pid = None
        finally:
            duplicate.close()
        if pid == shell_pid:
            found.append(fd)
    assert len(found) == 1, 'Compositor socket not uniquely identified: ' + repr(found)
    return found[0]


def unread_bytes(fd):
    buffer = bytearray(4)
    fcntl.ioctl(fd, termios.FIONREAD, buffer)
    return struct.unpack('i', buffer)[0]


def main():
    runtime = os.environ.get('XDG_RUNTIME_DIR', '')
    assert runtime.startswith('/tmp/typomorph-wayland-')
    assert os.environ.get('TYPOMORPH_PRIVATE_WAYLAND') == '1'
    assert os.environ.get('WAYLAND_DISPLAY') == 'typomorph-test'
    assert os.environ.get('IBUS_ADDRESS') == 'unix:path=' + runtime + '/ibus'
    assert not os.environ.get('DISPLAY') and not os.environ.get('GTK_IM_MODULE')
    assert TRACE.parent == Path(runtime) and TRACE.exists(), 'Protocol trace required'
    session = Gio.bus_get_sync(Gio.BusType.SESSION, None)

    def call(name, path, interface, method, value=None):
        return session.call_sync(name, path, interface, method, value, None, 0, 2000, None).unpack()
    name = 'org.gnome.Mutter.RemoteDesktop'
    owner = call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
                 'GetConnectionUnixProcessID', GLib.Variant('(s)', (name,)))[0]
    shell_pid = int(os.environ['TYPOMORPH_SHELL_PID'])
    assert owner == shell_pid
    remote = call(name, '/org/gnome/Mutter/RemoteDesktop', name, 'CreateSession')[0]
    iface = name + '.Session'
    call(name, remote, iface, 'Start')

    def fixture(method):
        return call('org.typomorph.StaleFixture', '/org/typomorph/StaleFixture',
                    'org.typomorph.StaleFixture', method)

    def engine_state():
        focus, text, cursor, anchor, updates, purpose = fixture('State')
        return dict(focus=focus, text=text, cursor=cursor, anchor=anchor, updates=updates, purpose=purpose)

    window = None
    try:
        Gtk.init()
        IBus.init()
        window = Gtk.Window()
        a, b = Gtk.Text(), Gtk.Text()
        box = Gtk.Box()
        for widget in (a, b):
            box.append(widget)
        window.set_child(box)
        window.present()
        a.grab_focus()
        wait_for(lambda: False, .8)
        for pressed in (True, False):  # Escape on the private compositor only: leave overview.
            call(name, remote, iface, 'NotifyKeyboardKeysym', GLib.Variant('(ub)', (0xff1b, pressed)))
        wait_for(lambda: False, .4)
        window.present()
        a.grab_focus()
        assert wait_for(lambda: window.is_active() and a.has_focus()), 'Private widget focus unavailable'
        bus = IBus.Bus.new()
        assert bus.is_connected()
        assert bus.set_global_engine('typomorph-stale-fixture')
        assert wait_for(lambda: bool(engine_state()['focus'])), 'No Shell focus reached the fixture'
        wayland_fd = find_wayland_fd(shell_pid)
        libraries = sorted({line.split()[-1] for line in Path('/proc/self/maps').read_text().splitlines()
                            if 'libgtk-4.so' in line})
        print('stale_fixture_route_verified=True; client=GTK %d.%d.%d; client_gtk_library=%s; '
              'protocol=zwp_text_input_v3 v1' % (Gtk.get_major_version(), Gtk.get_minor_version(),
                                                  Gtk.get_micro_version(), ','.join(libraries)), flush=True)

        def im_context(widget):
            controllers = widget.observe_controllers()
            contexts = [controllers.get_item(i).get_im_context() for i in range(controllers.get_n_items())
                        if isinstance(controllers.get_item(i), Gtk.EventControllerKey)]
            context = next(c for c in contexts if c is not None)
            assert context.get_context_id() == 'wayland', 'Not native Wayland input'
            return context

        def announce(widget):
            # GtkText itself resets its IM context after user edits; this is the
            # stock call that makes GTK publish the field's surrounding text and
            # queue a text-input commit. A programmatic set_text() publishes nothing.
            im_context(widget).reset()

        def focus_widget(widget):
            window.present()
            widget.grab_focus()
            assert wait_for(lambda: widget.has_focus()), 'Field did not gain focus'

        def prepare():
            focus_widget(b)
            b.set_text(OTHER)
            b.set_position(len(OTHER))
            announce(b)
            focus_widget(a)
            a.set_text(INITIAL)
            a.set_position(len(INITIAL))
            announce(a)
            assert wait_for(lambda: engine_state()['text'] == INITIAL and
                            engine_state()['cursor'] == len(INITIAL)), (
                'Engine did not receive initial state; ' + diagnostics() + '; engine=' +
                repr({k: (v if k != 'text' else len(v)) for k, v in engine_state().items()}))
            assert fixture('Snapshot')[0], 'Snapshot refused'

        def acknowledged(text, cursor):
            return wait_for(lambda: engine_state()['text'] == text and engine_state()['cursor'] == cursor)

        # Each change returns the field state the engine must acknowledge (matched
        # serial) before the stale request, or None when it is made while in flight.
        def cursor_moved():
            a.set_position(2)
            announce(a)
            return (INITIAL, 2)

        def prefix_inserted():
            a.insert_text('QQ', 0)
            a.set_position(len(INITIAL) + 2)
            announce(a)
            return ('QQ' + INITIAL, len(INITIAL) + 2)

        def word_changed():
            a.set_text('xx ghbdtx')
            a.set_position(9)
            announce(a)
            return ('xx ghbdtx', 9)

        def other_field():
            focus_widget(b)
            b.set_position(len(OTHER))
            announce(b)
            return (OTHER, len(OTHER))

        def selection_over_word():
            a.select_region(3, 9)
            announce(a)
            return (INITIAL, 9)

        def unchanged():
            return (INITIAL, len(INITIAL))
        # case -> (change, safe outcomes for (a, b), inflight)
        cases = {
            'control_unchanged': (unchanged, [(CORRECT, OTHER)], False),
            'cursor_moved': (cursor_moved, [(INITIAL, OTHER), (CORRECT, OTHER)], False),
            'prefix_inserted': (prefix_inserted, [('QQ' + INITIAL, OTHER), ('QQxx привет', OTHER)], False),
            'word_changed': (word_changed, [('xx ghbdtx', OTHER)], False),
            'other_field': (other_field, [(INITIAL, OTHER)], False),
            'selection_over_word': (selection_over_word, [(INITIAL, OTHER), (CORRECT, OTHER)], False),
            'inflight_cursor_moved': (cursor_moved, [(INITIAL, OTHER), (CORRECT, OTHER)], True),
            'inflight_prefix_inserted': (prefix_inserted, [('QQ' + INITIAL, OTHER), ('QQxx привет', OTHER)], True),
            'inflight_word_changed': (word_changed, [('xx ghbdtx', OTHER)], True),
        }
        selected = os.environ.get('TYPOMORPH_WAYLAND_CASE', '')
        results = []

        def run(case):
            change, safe, inflight = cases[case]
            prepare()
            drain()
            mark = len(trace_events())
            if inflight:
                # The stale request is delivered to the client socket but NOT read, so
                # the field can change before the matching done is dispatched.
                before = unread_bytes(wayland_fd)
                assert fixture('SendFromSnapshot')[0], 'Engine did not send the snapshot request'
                deadline = time.monotonic() + 5
                while unread_bytes(wayland_fd) - before < EXPECTED_BATCH_BYTES:
                    assert time.monotonic() < deadline, 'Request did not reach the client socket'
                    time.sleep(.001)
                change()
                pre = (a.get_text(), b.get_text())
            else:
                state = change()
                if state is not None:
                    assert acknowledged(*state), 'Engine did not acknowledge the changed field'
                pre = (a.get_text(), b.get_text())
                mark = len(trace_events())
                assert fixture('SendFromSnapshot')[0], 'Engine did not send the snapshot request'
            def delivered():
                events = trace_events()[mark:]
                names = [(r, n) for r, n, _ in events]
                if (False, 'commit_string') not in names:
                    return False
                tail = names[names.index((False, 'commit_string')):]
                return (False, 'done') in tail
            assert wait_for(delivered), 'Protocol events not observed'
            drain()

            events = trace_events()
            window_events = events[mark:]
            done = [int(args) for r, n, args in window_events if not r and n == 'done']
            # Serial condition, from the trace: the client's commit count when each
            # done was dispatched, against the serial the compositor reported.
            commits = sum(1 for r, n, _ in events[:mark] if r and n == 'commit')
            condition = []
            for r, n, args in window_events:
                if r and n == 'commit':
                    commits += 1
                if not r and n == 'done':
                    condition.append('match' if int(args) == commits else 'MISMATCH(%s!=%d)' % (args, commits))
            deletes = [args for r, n, args in window_events if not r and n == 'delete_surrounding_text']
            a_text, b_text = a.get_text(), b.get_text()
            outcome = (a_text, b_text)
            first_done = next(i for i, e in enumerate(window_events) if not e[0] and e[1] == 'done')
            queued_before_done = any(r and n == 'commit' for r, n, _ in window_events[:first_done])
            if outcome == pre and pre in safe:
                kind = 'UNCHANGED'
            elif outcome == pre:
                kind = 'CONTROL_NOT_APPLIED'
            elif outcome in safe:
                kind = 'EXACT_REPLACEMENT'
            else:
                kind = 'CORRUPTION'
            order = ' '.join(('>' if r else '<') + n for r, n, _ in window_events
                             if n in ('commit', 'delete_surrounding_text', 'commit_string', 'done'))
            print('CASE %s: serial_condition=%s delete_events=%d done_serials=%s '
                  'client_commit_queued_before_done=%s final_a=%r cursor_a=%d final_b=%r outcome=%s '
                  'order=[%s]' % (case, ','.join(condition), len(deletes), done, queued_before_done,
                                  a_text, a.get_position(), b_text, kind, order), flush=True)
            assert kind not in ('CORRUPTION', 'CONTROL_NOT_APPLIED'), (
                kind + ': a=%r b=%r' % (a_text, b_text))

        for case in cases:
            if selected and selected != case:
                continue
            try:
                run(case)
                results.append(True)
                print('PASS ' + case, flush=True)
            except AssertionError as error:
                results.append(False)
                print('FAIL ' + case + ': ' + str(error), flush=True)
        print('stale_checks_passed=%d; stale_checks_failed=%d' % (sum(results), len(results) - sum(results)),
              flush=True)
        return 0 if all(results) else 1
    finally:
        if window:
            window.destroy()
        try:
            fixture('Quit')
        except GLib.Error:
            pass
        call(name, remote, iface, 'Stop')


if __name__ == '__main__':
    raise SystemExit(main())
