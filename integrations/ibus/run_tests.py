"""Run actual IBus.Engine D-Bus methods on a private bus; no desktop daemon."""
import os
from pathlib import Path
import selectors
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent


def main():
    if len(sys.argv) == 1:
        with tempfile.TemporaryDirectory(prefix='typomorph-ibus-') as tmp:
            env = dict(os.environ, TYPOMORPH_PRIVATE_IBUS_TEST='1',
                       IBUS_ADDRESS='unix:path=/nonexistent/typomorph-isolated',
                       XDG_CONFIG_HOME=tmp, XDG_CACHE_HOME=tmp, XDG_DATA_HOME=tmp,
                       XDG_RUNTIME_DIR=tmp, GIO_USE_VFS='local', PYTHONDONTWRITEBYTECODE='1')
            for name in ['DISPLAY', 'WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS',
                         'DBUS_STARTER_ADDRESS', 'DBUS_STARTER_BUS_TYPE']:
                env.pop(name, None)
            return subprocess.run(['dbus-run-session', '--', '/usr/bin/python3',
                                   str(Path(__file__).resolve()), '--inside'],
                                  env=env, timeout=45).returncode
    if sys.argv[1:] != ['--inside'] or os.environ.get('TYPOMORPH_PRIVATE_IBUS_TEST') != '1':
        raise SystemExit('Use the runner without arguments')
    process = subprocess.Popen(['/usr/bin/python3', str(ROOT / 'engine.py')],
                               stdout=subprocess.PIPE, text=True)
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            if not selector.select(5) or process.stdout.readline().strip() != 'READY':
                raise RuntimeError('Private engine failed to start')
        return subprocess.run(['/usr/bin/python3', str(ROOT / 'test_protocol.py')],
                              timeout=30).returncode
    finally:
        process.terminate()
        try:
            process.wait(3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


if __name__ == '__main__':
    raise SystemExit(main())
