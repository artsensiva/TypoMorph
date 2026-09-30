"""Private IBus daemon + synthetic clients; never replaces the desktop daemon."""
import os
from pathlib import Path
import selectors
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parent


def stop(process):
    process.terminate()
    try:
        process.wait(3)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def main():
    if len(sys.argv) == 1:
        with tempfile.TemporaryDirectory(prefix='typomorph-ibus-daemon-') as tmp:
            env = dict(os.environ, TYPOMORPH_PRIVATE_IBUS_DAEMON='1',
                       IBUS_ADDRESS=f'unix:path={tmp}/bus',
                       XDG_CONFIG_HOME=tmp, XDG_CACHE_HOME=tmp, XDG_DATA_HOME=tmp,
                       XDG_RUNTIME_DIR=tmp, GIO_USE_VFS='local', PYTHONDONTWRITEBYTECODE='1')
            for name in ['DISPLAY', 'WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS',
                         'DBUS_STARTER_ADDRESS', 'DBUS_STARTER_BUS_TYPE']:
                env.pop(name, None)
            return subprocess.run(['dbus-run-session', '--', '/usr/bin/python3',
                                   str(Path(__file__).resolve()), '--inside'],
                                  env=env, timeout=60).returncode
    if sys.argv[1:] != ['--inside'] or os.environ.get('TYPOMORPH_PRIVATE_IBUS_DAEMON') != '1':
        raise SystemExit('Run without arguments')
    address = os.environ['IBUS_ADDRESS']
    if address != 'unix:path=' + os.environ['XDG_RUNTIME_DIR'] + '/bus' or not os.environ['XDG_RUNTIME_DIR'].startswith('/tmp/typomorph-ibus-daemon-'):
        raise SystemExit('Refusing non-test bus')
    daemon = subprocess.Popen(['ibus-daemon', '--single', '--panel=disable',
        '--config=disable', '--emoji-extension=disable', '--cache=none',
        '--desktop=typomorph-test', '--address=' + address],
        stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    factory = None
    try:
        deadline = time.monotonic() + 5
        while not Path(os.environ['XDG_RUNTIME_DIR'], 'bus').exists():
            if daemon.poll() is not None or time.monotonic() > deadline:
                raise RuntimeError('Private daemon startup failed')
            time.sleep(0.05)
        factory = subprocess.Popen(['/usr/bin/python3', str(ROOT / 'private_factory.py')],
                                   stdout=subprocess.PIPE, text=True)
        with selectors.DefaultSelector() as selector:
            selector.register(factory.stdout, selectors.EVENT_READ)
            if not selector.select(5) or factory.stdout.readline().strip() != 'READY':
                raise RuntimeError('Private factory startup failed')
        return subprocess.run(['/usr/bin/python3', str(ROOT / 'test_daemon.py')],
                              timeout=35).returncode
    finally:
        if factory is not None:
            stop(factory)
        stop(daemon)
        # No per-event logging is enabled. Do not dump daemon diagnostics as input evidence.
        daemon.stderr.close()


if __name__ == '__main__':
    raise SystemExit(main())
