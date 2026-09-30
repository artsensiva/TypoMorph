"""Test-owned transactions with ordinary keys on a private native Wayland seat."""
import os
import time
import gi

gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, Gio, GLib
from client_transaction import FieldOwner
from test_wayland_route import drain


def main():
    runtime = os.environ.get('XDG_RUNTIME_DIR', '')
    assert runtime.startswith('/tmp/typomorph-wayland-')
    assert os.environ.get('TYPOMORPH_PRIVATE_WAYLAND') == '1'
    assert os.environ.get('TYPOMORPH_CLIENT_TRANSACTION') == '1'
    assert os.environ.get('WAYLAND_DISPLAY') == 'typomorph-test'
    assert not os.environ.get('DISPLAY') and not os.environ.get('GTK_IM_MODULE')
    session = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    def call(name, path, iface, method, args=None):
        return session.call_sync(name, path, iface, method, args, None, 0, 1000, None).unpack()
    name = 'org.gnome.Mutter.RemoteDesktop'
    pid = call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
        'GetConnectionUnixProcessID', GLib.Variant('(s)', (name,)))[0]
    assert pid == int(os.environ['TYPOMORPH_SHELL_PID'])
    assert not call('org.freedesktop.DBus', '/org/freedesktop/DBus',
        'org.freedesktop.DBus', 'NameHasOwner',
        GLib.Variant('(s)', ('org.typomorph.GtkFixture',)))[0]
    remote = call(name, '/org/gnome/Mutter/RemoteDesktop', name, 'CreateSession')[0]
    call(name, remote, name+'.Session', 'Start')
    def key(value):
        for pressed in (True, False):
            call(name, remote, name+'.Session', 'NotifyKeyboardKeysym',
                 GLib.Variant('(ub)', (value, pressed)))
    window = None
    try:
        Gtk.init()
        window = Gtk.Window()
        a, b, secret = Gtk.Text(), Gtk.Text(), Gtk.Text()
        secret.set_visibility(False); secret.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        box = Gtk.Box()
        for w in (a,b,secret): box.append(w)
        window.set_child(box); window.present(); a.grab_focus(); drain(.5)
        key(0xff1b); drain(.3)
        window.present(); a.grab_focus(); drain(.3)
        first, second, guarded = [FieldOwner(a, True), FieldOwner(b, True), FieldOwner(secret, False)]
        assert first.context.get_context_id() == 'wayland'
        print('native_wayland_no_echo_fixture=True', flush=True)
        results = []
        def focus(w):
            window.present(); w.grab_focus(); drain(.08)
            assert w.has_focus() and window.is_active()
        def prepare():
            first.reconnect(); first.set_ordinary(True); first._composition_end()
            focus(a)
            for w in (a,b,secret): w.set_text('')
            for c in 'ghbdtn': key(ord(c))
            drain(.12)
            assert a.get_text() == 'ghbdtn', 'Original input not delivered normally'
            ticket = first.request()
            assert ticket is not None, 'Ordinary request refused'
            assert a.get_text() == 'ghbdtn', 'Request altered original input'
            return ticket
        def unchanged(ticket):
            assert not first.complete(ticket), 'Stale correction accepted'
            assert not first.pending, 'Snapshot retained after invalidation'
        def run(label, test):
            try:
                test()
                results.append(True); print('PASS '+label, flush=True)
            except (AssertionError, RuntimeError) as error:
                results.append(False); print('FAIL '+label+': '+str(error), flush=True)
        def stable():
            t = prepare(); delivered = []
            GLib.timeout_add(50, lambda: (delivered.append(first.complete(t)), False)[1])
            assert a.get_text() == 'ghbdtn'
            drain(.15)
            assert delivered == [True] and a.get_text() == 'привет'
            assert not first.complete(t), 'Duplicate completion accepted'
        def switch():
            t = prepare(); focus(b); unchanged(t)
            assert (a.get_text(), b.get_text()) == ('ghbdtn', '')
        def roundtrip():
            t = prepare(); focus(b); focus(a); unchanged(t)
            assert (a.get_text(), b.get_text()) == ('ghbdtn', '')
        def typing():
            t = prepare(); key(ord('x')); key(ord('y')); drain(.1)
            unchanged(t); assert a.get_text() == 'ghbdtnxy'
        def backspace():
            t = prepare(); key(0xff08); drain(.1)
            unchanged(t); assert a.get_text() == 'ghbdt'
        def selection():
            t = prepare(); a.select_region(1,3); a.set_position(6)
            unchanged(t); assert a.get_text() == 'ghbdtn'
        def protected():
            t = prepare(); focus(secret)
            before = (guarded.snapshots, guarded.requests)
            key(ord('x')); drain(.1)
            assert guarded.request() is None
            assert (guarded.snapshots, guarded.requests) == before
            unchanged(t)
            assert (a.get_text(), secret.get_text()) == ('ghbdtn','x')
        def unknown():
            t = prepare(); first.set_ordinary(False); before = first.snapshots
            assert first.request() is None and first.snapshots == before
            unchanged(t); assert a.get_text() == 'ghbdtn'
        def reset():
            t = prepare(); first.reset(); unchanged(t)
            assert a.get_text() == 'ghbdtn'
        def composition():
            t = prepare()
            # Explicit lifecycle signal fixture, not a full native IME test.
            first.context.emit('preedit-start'); before = first.snapshots
            assert first.request() is None and first.snapshots == before
            unchanged(t); assert a.get_text() == 'ghbdtn'
            first.context.emit('preedit-end')
        def cancel():
            t = prepare(); first.cancel(t); unchanged(t)
            assert a.get_text() == 'ghbdtn'
        def disconnect():
            t = prepare(); first.disconnect(); first.reconnect(); unchanged(t)
            assert a.get_text() == 'ghbdtn'
        def surfaces():
            t = prepare(); other_window = Gtk.Window(); other = Gtk.Text()
            other_window.set_child(other)
            try:
                other_window.present(); other.grab_focus(); drain(.2)
                assert other_window.is_active() and other.has_focus()
                unchanged(t)
                assert a.get_text() == 'ghbdtn' and other.get_text() == ''
            finally: other_window.destroy(); drain(.1)
        def destroyed():
            extra = Gtk.Text(); box.append(extra); focus(extra)
            owner = FieldOwner(extra, True)
            for c in 'ghbdtn': key(ord(c))
            drain(.1)
            t = owner.request(); assert t is not None
            owner.dispose(); box.remove(extra)
            new = Gtk.Text(); box.append(new); focus(new)
            assert not owner.complete(t) and new.get_text() == ''
            box.remove(new)
        def reentrant():
            t = prepare(); observations = []
            def callback(w, unused):
                observations.append((w.get_text(), first.complete(t), first.request()))
            handler = a.connect('notify::buffer', callback)
            try: assert first.complete(t)
            finally: a.disconnect(handler)
            assert observations == [('привет', False, None)], repr(observations)
            assert a.get_text() == 'привет'
        def timeout():
            prepare(); t = first.request(timeout_ms=30)
            assert t is not None
            drain(.08); unchanged(t)
            assert a.get_text() == 'ghbdtn'
        def timeout_before_timer_dispatch():
            prepare(); t = first.request(timeout_ms=10)
            assert t is not None
            time.sleep(.02)  # Deliberately do not dispatch the GLib timeout.
            unchanged(t); assert a.get_text() == 'ghbdtn'
        def purpose_change():
            t = prepare(); a.set_input_purpose(Gtk.InputPurpose.PASSWORD)
            before = first.snapshots
            try:
                assert first.request() is None and first.snapshots == before
                unchanged(t); assert a.get_text() == 'ghbdtn'
            finally: a.set_input_purpose(Gtk.InputPurpose.FREE_FORM)
        def foreign_ticket():
            t = prepare(); focus(b)
            for c in 'ghbdtn': key(ord(c))
            drain(.1); own = second.request(); assert own is not None
            assert not second.complete(t)
            assert b.get_text() == 'ghbdtn'
            assert second.complete(own) and b.get_text() == 'привет'
        def bounded():
            prepare(); a.set_text('x'*33); a.set_position(33); before = first.snapshots
            assert first.request() is None and first.snapshots == before
        for label, test in [('stable_delayed_correction', stable), ('focus_change', switch),
            ('round_trip', roundtrip), ('ordered_typing', typing), ('backspace', backspace),
            ('selection_change_and_return', selection), ('protected_exclusion', protected),
            ('unknown_exclusion', unknown), ('reset', reset),
            ('composition_signal_fixture', composition), ('cancellation', cancel),
            ('disconnect_and_reconnect', disconnect), ('two_surfaces', surfaces),
            ('destroyed_and_recreated_field', destroyed), ('nested_callback', reentrant),
            ('bounded_snapshot', bounded), ('timeout', timeout),
            ('timeout_before_timer_dispatch', timeout_before_timer_dispatch),
            ('purpose_change', purpose_change), ('foreign_ticket', foreign_ticket)]:
            run(label, test)
        print('transaction_checks_passed='+str(sum(results))+'; failed='+str(len(results)-sum(results)), flush=True)
        return 0 if all(results) else 1
    finally:
        if window: window.destroy()
        call(name, remote, name+'.Session', 'Stop')


if __name__ == '__main__':
    raise SystemExit(main())
