"""Private native Wayland client and compositor-confined synthetic key probe."""
import os
import sys
import time
from pathlib import Path
import gi

gi.require_version('Gtk', '4.0')
gi.require_version('IBus', '1.0')
from gi.repository import Gtk, Gio, GLib, IBus
sys.path.insert(0, str(Path(__file__).resolve().parents[2]/'ibus'))


def drain(seconds=.2):
    end = time.monotonic()+seconds
    while time.monotonic()<end:
        while GLib.MainContext.default().pending(): GLib.MainContext.default().iteration(False)
        time.sleep(.005)


def main():
    runtime = os.environ.get('XDG_RUNTIME_DIR', '')
    assert runtime.startswith('/tmp/typomorph-wayland-')
    assert os.environ.get('TYPOMORPH_PRIVATE_WAYLAND') == '1'
    assert os.environ.get('WAYLAND_DISPLAY') == 'typomorph-test'
    assert os.environ.get('IBUS_ADDRESS') == 'unix:path='+runtime+'/ibus'
    assert not os.environ.get('DISPLAY') and not os.environ.get('GTK_IM_MODULE')
    session = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    def call(name, path, interface, method, value=None):
        return session.call_sync(name, path, interface, method, value, None, 0, 1000, None).unpack()
    # Refuse input unless the private RemoteDesktop service belongs to our child.
    name = 'org.gnome.Mutter.RemoteDesktop'
    owner = call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
                 'GetConnectionUnixProcessID', GLib.Variant('(s)', (name,)))[0]
    assert owner == int(os.environ['TYPOMORPH_SHELL_PID'])
    remote = call(name, '/org/gnome/Mutter/RemoteDesktop', name, 'CreateSession')[0]
    iface = name+'.Session'
    call(name, remote, iface, 'Start')
    def key(value):
        for pressed in (True, False):
            call(name, remote, iface, 'NotifyKeyboardKeysym', GLib.Variant('(ub)', (value, pressed)))
    window = None
    try:
        Gtk.init(); IBus.init()
        window = Gtk.Window()
        a, b, secret = Gtk.Text(), Gtk.Text(), Gtk.Text()
        secret.set_visibility(False); secret.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        box = Gtk.Box()
        for w in (a,b,secret): box.append(w)
        window.set_child(box); window.present(); a.grab_focus()
        drain(1)
        key(0xff1b)  # Escape only on the private compositor: leave initial overview.
        drain(.5)
        window.present(); a.grab_focus(); drain(.5)
        assert window.is_active() and a.has_focus(), 'Private widget focus unavailable'
        controllers = a.observe_controllers()
        contexts = [controllers.get_item(i).get_im_context() for i in range(controllers.get_n_items())
                    if isinstance(controllers.get_item(i), Gtk.EventControllerKey)]
        context = next(c for c in contexts if c is not None)
        assert context.get_context_id() == 'wayland', 'Not native Wayland input'
        bus = IBus.Bus.new(); assert bus.is_connected()
        assert bus.set_global_engine('typomorph-gtk-fixture')
        drain(.4)
        def fixture(method):
            return call('org.typomorph.GtkFixture', '/org/typomorph/GtkFixture', 'org.typomorph.GtkFixture', method)
        assert fixture('State')[0], 'No Shell focus reached fixture'
        key(ord('x')); drain(.4)
        assert fixture('State')[3] > 0, 'Key bypassed fixture'
        assert a.get_text() == 'x' and b.get_text() == '', 'Native echo target mismatch'
        print('native_wayland_route_and_private_input_verified=True', flush=True)
        results = []
        def focus(widget):
            window.present()
            widget.grab_focus()
            drain(.3)
            assert widget.has_focus(), 'Test field did not gain focus'
        def clear():
            focus(a)
            context.reset()
            drain()
            for widget in (a, b, secret): widget.set_text('')
            drain()
        def run(name, test):
            clear()
            try:
                test()
                results.append(True)
                print('PASS '+name, flush=True)
            except AssertionError as error:
                results.append(False)
                print('FAIL '+name+': '+str(error), flush=True)
        started = [False]
        subscription = session.signal_subscribe('org.typomorph.GtkFixture',
            'org.typomorph.GtkFixture', 'KeyStarted', '/org/typomorph/GtkFixture',
            None, 0, lambda *args: started.__setitem__(0, True))
        def outstanding_key():
            started[0] = False
            key(ord('x'))
            deadline = time.monotonic()+2
            while not started[0] and time.monotonic()<deadline:
                drain(.005)
            assert started[0], 'Fixture key entry was not observed'
        def contexts():
            first = fixture('State')[0]
            focus(b)
            assert fixture('State')[0] == first, 'Expected shared Shell context'
        def draft():
            assert fixture('Seed')[0], 'Fixture refused preedit seed'
            drain(.3)
            focus(b)
            assert (a.get_text(), b.get_text()) == ('ghb', ''), 'Old draft mismatch: '+repr((a.get_text(), b.get_text()))
        def delayed():
            outstanding_key()
            b.grab_focus()
            drain(.7)
            assert (a.get_text(), b.get_text()) == ('x', ''), 'Delayed target mismatch: '+repr((a.get_text(), b.get_text()))
        def round_trip():
            outstanding_key()
            b.grab_focus()
            a.grab_focus()
            drain(.7)
            assert (a.get_text(), b.get_text()) == ('x', ''), 'Round-trip input changed or lost'
        def selected():
            a.set_text('abc'); a.select_region(1, 2); drain()
            key(ord('x')); drain(.5)
            assert a.get_text() == 'axc', 'Selection range was not replaced correctly'
        def protected():
            assert fixture('Seed')[0], 'Fixture refused preedit seed'; drain(.3)
            focus(secret)
            state = fixture('State')
            print('protected_fixture_active='+str(bool(state[0]))+'; protected_fixture_purpose='+str(state[1]), flush=True)
            # An unfocused engine is a valid refusal path; 999 is our sentinel,
            # not an OS content-purpose value. Do not require password delivery.
            assert not state[0] or state[1] == int(IBus.InputPurpose.PASSWORD), 'Active fixture lacks protected purpose'
            assert not fixture('Seed')[0], 'Protected preedit stimulus accepted'
            key(ord('x')); drain(.5)
            assert fixture('State')[3] == state[3], 'Protected field received fixture echo'
            assert (a.get_text(), secret.get_text()) == ('ghb', 'x'), 'Protected transition mismatch: '+repr((a.get_text(), secret.get_text()))
        def reset():
            assert fixture('Seed')[0], 'Fixture refused preedit seed'; drain(.3)
            context.reset(); drain(.3)
            assert a.get_text() == 'ghb', 'Reset draft: '+repr(a.get_text())+'; preedit='+repr(context.get_preedit_string()[0])
        def surfaces():
            other_window = Gtk.Window()
            other = Gtk.Text(); other_window.set_child(other)
            # Realize and return focus before sending the delayed input.
            other_window.present(); drain(.4)
            window.present(); a.grab_focus(); drain(.4)
            assert a.has_focus() and window.is_active()
            try:
                outstanding_key()
                other_window.present(); other.grab_focus(); drain(.7)
                assert other_window.is_active(), 'Second surface did not become active'
                assert (a.get_text(), other.get_text()) == ('x', ''), 'Cross-surface mismatch: '+repr((a.get_text(), other.get_text()))
            finally:
                other_window.destroy(); drain()
        def disconnected():
            assert fixture('Seed')[0], 'Fixture refused preedit seed'; drain(.3)
            fixture('Quit'); drain(.3)
            b.grab_focus(); drain(.4)
            assert (a.get_text(), b.get_text()) == ('ghb', ''), 'Disconnect draft mismatch: '+repr((a.get_text(), b.get_text()))
        for case_name, test in [('shared_context', contexts), ('focus_draft', draft),
                           ('delayed_same_surface', delayed), ('round_trip', round_trip),
                           ('selection', selected), ('protected_transition', protected),
                           ('reset_draft', reset), ('delayed_two_surfaces', surfaces),
                           ('disconnect_draft', disconnected)]:
            selected_case = os.environ.get('TYPOMORPH_WAYLAND_CASE', '')
            if not selected_case or selected_case == case_name:
                run(case_name, test)
        session.signal_unsubscribe(subscription)
        print('native_checks_passed='+str(sum(results))+'; native_checks_failed='+str(len(results)-sum(results)), flush=True)
        return 0 if all(results) else 1
    finally:
        if window: window.destroy()
        call(name, remote, iface, 'Stop')


if __name__ == '__main__':
    raise SystemExit(main())
