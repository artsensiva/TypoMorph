"""Private Broadway + IBus real-widget lifecycle experiment; no desktop access."""
import os
from pathlib import Path
import selectors
import signal
import subprocess
import sys
import tempfile
import time
from run_daemon_tests import stop

ROOT = Path(__file__).resolve().parent


def main():
    if sys.argv[1:] in ([], ['--async-client']):
        with tempfile.TemporaryDirectory(prefix='typomorph-gtk-') as tmp:
            env = dict(os.environ)
            for name in list(env):
                if name.startswith(('IBUS_', 'GTK_', 'GDK_', 'DBUS_', 'BROADWAY_')) or name in ('DISPLAY', 'WAYLAND_DISPLAY', 'WAYLAND_SOCKET'):
                    env.pop(name, None)
            env.update(TYPOMORPH_PRIVATE_GTK='1', IBUS_ADDRESS='unix:path='+tmp+'/bus',
                XDG_RUNTIME_DIR=tmp, XDG_CONFIG_HOME=tmp, XDG_CACHE_HOME=tmp,
                XDG_DATA_HOME=tmp, GIO_USE_VFS='local', GTK_A11Y='none',
                GDK_BACKEND='broadway', BROADWAY_DISPLAY=':7', GTK_IM_MODULE='ibus',
                PYTHONDONTWRITEBYTECODE='1',
                IBUS_ENABLE_SYNC_MODE='0' if sys.argv[1:] else '1')
            process = subprocess.Popen(['dbus-run-session', '--', '/usr/bin/python3',
                str(Path(__file__).resolve()), '--inside'], env=env, start_new_session=True)
            try:
                return process.wait(50)
            finally:
                # Kill only this runner's process group, including timeout leftovers.
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    process.wait(3)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()

    if sys.argv[1:] != ['--inside']:
        raise SystemExit('Use no arguments or --async-client')
    check_environment()
    children = []
    try:
        runtime = os.environ['XDG_RUNTIME_DIR']
        children.append(subprocess.Popen(['gtk4-broadwayd', '--unixsocket='+runtime+'/http', ':7'],
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL))
        children.append(subprocess.Popen(['ibus-daemon', '--single', '--panel=disable',
            '--config=disable', '--emoji-extension=disable', '--cache=none',
            '--desktop=typomorph-test', '--address='+os.environ['IBUS_ADDRESS']],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL))
        deadline = time.monotonic()+5
        while not Path(runtime, 'bus').exists() or not Path(runtime, 'broadway8.socket').exists():
            if time.monotonic()>deadline or any(p.poll() is not None for p in children):
                raise RuntimeError('Isolated display/bus unavailable')
            time.sleep(.05)
        factory = subprocess.Popen(['/usr/bin/python3', str(ROOT/'gtk_fixture.py')], stdout=subprocess.PIPE, text=True)
        children.append(factory)
        with selectors.DefaultSelector() as selector:
            selector.register(factory.stdout, selectors.EVENT_READ)
            if not selector.select(5) or factory.stdout.readline().strip() != 'READY':
                raise RuntimeError('GTK fixture unavailable')
        return subprocess.run(['/usr/bin/python3', str(ROOT/'test_gtk_lifecycle.py')], timeout=30).returncode
    finally:
        for child in reversed(children):
            stop(child)
            if child.stdout:
                child.stdout.close()


def check_environment():
    runtime = os.environ.get('XDG_RUNTIME_DIR', '')
    if (os.environ.get('TYPOMORPH_PRIVATE_GTK') != '1' or not runtime.startswith('/tmp/typomorph-gtk-')
        or os.environ.get('IBUS_ADDRESS') != 'unix:path='+runtime+'/bus'
        or os.environ.get('GDK_BACKEND') != 'broadway'
        or os.environ.get('DISPLAY') or os.environ.get('WAYLAND_DISPLAY') or os.environ.get('WAYLAND_SOCKET')):
        raise RuntimeError('Isolated GTK runner required')


if __name__ == '__main__':
    raise SystemExit(main())
