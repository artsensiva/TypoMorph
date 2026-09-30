#!/usr/bin/env python3
"""Run one fixed Text Editor case on a disposable native Wayland compositor."""
import argparse
import os
from pathlib import Path
import selectors
import signal
import subprocess
import sys
import tempfile
import time

from assess_probe import EXPECTED, MAX_OUTPUT_BYTES, Outcome, assess


def safe_markers(output):
    allowed = {marker.encode() for markers in EXPECTED.values() for marker in markers}
    allowed.update({b'typomorph-harness: READY', b'typomorph-harness: FINISHED',
        b'typomorph-harness: FAILED', b'typomorph-probe: STALE',
        b'typomorph-probe: INELIGIBLE', b'typomorph-probe: BUSY',
        b'typomorph-check: NOT_READY', b'typomorph-check: WRONG_CASE',
        b'typomorph-check: CHECK_FAILED',
        b'typomorph-diagnostic: BEFORE_MUTATION_REFUSED',
        b'typomorph-diagnostic: BEGIN_ACTION_REFUSED'})
    return [line.decode('ascii') for line in output.splitlines() if line in allowed]


def stop(child):
    if child is None:
        return
    if child.poll() is None:
        child.terminate()
        try:
            child.wait(3)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()


def private_environment(root):
    return dict(PATH='/usr/bin:/bin', LANG='C.UTF-8',
        XDG_RUNTIME_DIR=str(root), XDG_CONFIG_HOME=str(root / 'config'),
        XDG_CACHE_HOME=str(root / 'cache'), XDG_DATA_HOME=str(root / 'data'),
        XDG_STATE_HOME=str(root / 'state'), XDG_DATA_DIRS='/usr/local/share:/usr/share',
        GSETTINGS_BACKEND='memory', GIO_USE_VFS='local', GTK_A11Y='none',
        IBUS_ADDRESS='unix:path=' + str(root / 'ibus'),
        LIBGL_ALWAYS_SOFTWARE='1', GNOME_SHELL_SESSION_MODE='user',
        TYPOMORPH_PRIVATE_WAYLAND='1', TYPOMORPH_TEXT_EDITOR_SYNTHETIC='1',
        PYTHONDONTWRITEBYTECODE='1')


def inside(args):
    root = Path(os.environ.get('XDG_RUNTIME_DIR', ''))
    bus = os.environ.get('DBUS_SESSION_BUS_ADDRESS', '')
    if (not str(root).startswith('/tmp/typomorph-wayland-') or
        str(root) not in bus or os.environ.get('TYPOMORPH_PRIVATE_WAYLAND') != '1' or
        any(os.environ.get(k) for k in ('DISPLAY', 'WAYLAND_DISPLAY', 'WAYLAND_SOCKET'))):
        raise RuntimeError('Private session required')
    os.environ['DBUS_SYSTEM_BUS_ADDRESS'] = bus
    ibus = shell = app = None
    try:
        ibus = subprocess.Popen(['ibus-daemon', '--single', '--panel=disable',
            '--config=disable', '--emoji-extension=disable', '--cache=none',
            '--address=' + os.environ['IBUS_ADDRESS']],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        deadline = time.monotonic() + 5
        while not (root / 'ibus').exists():
            if ibus.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError('Private input service failed')
            time.sleep(.05)
        shell = subprocess.Popen(['gnome-shell', '--headless', '--no-x11',
            '--virtual-monitor=1024x768', '--wayland-display=typomorph-test'],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        deadline = time.monotonic() + 15
        while not (root / 'typomorph-test').exists():
            if shell.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError('Private compositor failed')
            time.sleep(.1)
        import gi
        gi.require_version('Gio', '2.0')
        from gi.repository import Gio, GLib
        session = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        name = 'org.gnome.Mutter.RemoteDesktop'
        def call(destination, path, interface, method, value=None):
            return session.call_sync(destination, path, interface, method,
                value, None, 0, 1500, None).unpack()
        # Socket existence alone does not authorize synthetic input. Require
        # the D-Bus service to belong to the compositor child we just launched.
        deadline = time.monotonic() + 8
        while True:
            try:
                owner = call('org.freedesktop.DBus', '/org/freedesktop/DBus',
                    'org.freedesktop.DBus', 'GetConnectionUnixProcessID',
                    GLib.Variant('(s)', (name,)))[0]
                if owner != shell.pid:
                    raise RuntimeError('Compositor identity mismatch')
                break
            except GLib.Error:
                if shell.poll() is not None or time.monotonic() >= deadline:
                    raise RuntimeError('Private input endpoint unavailable')
                time.sleep(.1)
        remote = call(name, '/org/gnome/Mutter/RemoteDesktop', name, 'CreateSession')[0]
        interface = name + '.Session'
        call(name, remote, interface, 'Start')
        def key(value):
            if shell.poll() is not None:
                raise RuntimeError('Compositor exited before input')
            for pressed in (True, False):
                call(name, remote, interface, 'NotifyKeyboardKeysym',
                    GLib.Variant('(ub)', (value, pressed)))
        env = dict(os.environ, WAYLAND_DISPLAY='typomorph-test', GDK_BACKEND='wayland',
            GSETTINGS_SCHEMA_DIR=str(args.schemas), TYPOMORPH_TEXT_EDITOR_CASE=args.case)
        app = subprocess.Popen([str(args.binary), '--standalone'], env=env,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        output = bytearray()
        sent = False
        escaped = False
        start = time.monotonic()
        with selectors.DefaultSelector() as selector:
            selector.register(app.stdout, selectors.EVENT_READ)
            while time.monotonic() - start < 30:
                if not escaped and time.monotonic() - start >= 2:
                    key(0xff1b)  # Dismiss overview only on our private compositor.
                    escaped = True
                for entry, _ in selector.select(.1):
                    block = os.read(entry.fd, 4096)
                    if not block:
                        selector.unregister(entry.fd)
                        continue
                    output.extend(block)
                    if len(output) > MAX_OUTPUT_BYTES:
                        raise RuntimeError('Application output limit exceeded')
                if args.case == 'baseline' and time.monotonic() - start >= 5:
                    if app.poll() is not None:
                        raise RuntimeError('Baseline exited during startup')
                    # A smoke check only, not evidence of window focus or editing.
                    print('baseline_process_survived_startup=True; editing_not_tested=True')
                    return 0
                if not sent and b'typomorph-harness: READY\n' in output:
                    sent = True
                    for character in 'ghbdtn':
                        key(ord(character))
                if app.poll() is not None and not selector.get_map():
                    break
            else:
                raise RuntimeError('Application test timed out')
        result = assess(args.case, bytes(output), app.returncode)
        # Fixed markers only: never print arbitrary app errors or document text.
        for marker in safe_markers(bytes(output)):
            print(marker)
        print('case=' + args.case + ' result=' + result.name)
        return int(result)
    finally:
        for child in (app, shell, ibus):
            stop(child)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--schemas', type=Path, required=True)
    parser.add_argument('--case', choices=['baseline', *EXPECTED], required=True)
    parser.add_argument('--inside', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    args.binary = args.binary.resolve(strict=True)
    args.schemas = args.schemas.resolve(strict=True)
    if not args.binary.is_file() or not (args.schemas / 'gschemas.compiled').is_file():
        parser.error('A development executable and compiled local schemas are required')
    if args.inside:
        return inside(args)
    with tempfile.TemporaryDirectory(prefix='typomorph-wayland-') as directory:
        root = Path(directory)
        for name in ('config', 'cache', 'data', 'state'):
            (root / name).mkdir()
        config = root / 'bus.conf'
        config.write_text('<busconfig><type>session</type><listen>unix:tmpdir=' + str(root) +
            '</listen><auth>EXTERNAL</auth><policy context="default">'
            '<allow send_destination="*"/><allow receive_sender="*"/>'
            '<allow own="*"/></policy></busconfig>')
        child = subprocess.Popen(['dbus-run-session', '--config-file=' + str(config), '--',
            '/usr/bin/python3', str(Path(__file__).resolve()), '--inside',
            '--binary', str(args.binary), '--schemas', str(args.schemas), '--case', args.case],
            env=private_environment(root), start_new_session=True)
        try:
            return child.wait(60)
        finally:
            try:
                os.killpg(child.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            stop(child)
            # The session leader may exit before a descendant; kill any residual
            # process group as well, before removing its disposable XDG state.
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (RuntimeError, OSError, subprocess.TimeoutExpired):
        # Exception details may contain application output; no raw diagnostic echo.
        print('Isolated application test failed; no safety result established.', file=sys.stderr)
        raise SystemExit(int(Outcome.FAILED))
