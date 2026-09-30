"""Headless native-route preflight. Never uses the owner's display or buses."""
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time


CASES = ('shared_context', 'focus_draft', 'delayed_same_surface', 'round_trip',
         'selection', 'protected_transition', 'reset_draft', 'delayed_two_surfaces',
         'disconnect_draft')

def main():
    selected = sys.argv[2] if len(sys.argv) == 3 and sys.argv[1] == '--case' else ''
    transaction = sys.argv[1:] == ['--client-transaction']
    if len(sys.argv) == 1 or selected in CASES or transaction:
        with tempfile.TemporaryDirectory(prefix='typomorph-wayland-') as tmp:
            env = dict(PATH='/usr/bin:/bin', LANG='C.UTF-8',
                XDG_RUNTIME_DIR=tmp, XDG_CONFIG_HOME=tmp+'/config',
                XDG_CACHE_HOME=tmp+'/cache', XDG_DATA_HOME=tmp+'/data',
                XDG_STATE_HOME=tmp+'/state', XDG_DATA_DIRS='/usr/local/share:/usr/share',
                GSETTINGS_BACKEND='memory', GIO_USE_VFS='local',
                DBUS_SYSTEM_BUS_ADDRESS='unix:path='+tmp+'/no-system-bus',
                IBUS_ADDRESS='unix:path='+tmp+'/ibus', GTK_A11Y='none',
                LIBGL_ALWAYS_SOFTWARE='1', GNOME_SHELL_SESSION_MODE='user',
                TYPOMORPH_PRIVATE_WAYLAND='1', TYPOMORPH_WAYLAND_CASE=selected,
                TYPOMORPH_CLIENT_TRANSACTION='1' if transaction else '0',
                PYTHONDONTWRITEBYTECODE='1')
            for name in ('config', 'cache', 'data', 'state'):
                Path(tmp, name).mkdir()
            config = Path(tmp, 'bus.conf')
            config.write_text('<busconfig><type>session</type><listen>unix:tmpdir='+tmp+
                '</listen><auth>EXTERNAL</auth><policy context="default">'
                '<allow send_destination="*"/><allow receive_sender="*"/>'
                '<allow own="*"/></policy></busconfig>')
            process = subprocess.Popen(['dbus-run-session', '--config-file='+str(config), '--', '/usr/bin/python3',
                str(Path(__file__).resolve()), '--inside'], env=env, start_new_session=True)
            try:
                return process.wait(65)
            finally:
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
        raise SystemExit('Run without arguments')
    check_environment()
    runtime = os.environ['XDG_RUNTIME_DIR']
    # Both bus roles stay private; no real system services or activation dirs.
    os.environ['DBUS_SYSTEM_BUS_ADDRESS'] = os.environ['DBUS_SESSION_BUS_ADDRESS']
    daemon = subprocess.Popen(['ibus-daemon', '--single', '--panel=disable',
        '--config=disable', '--emoji-extension=disable', '--cache=none',
        '--address='+os.environ['IBUS_ADDRESS']], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    factory = None
    shell = None
    try:
        deadline = time.monotonic()+5
        while not Path(runtime, 'ibus').exists():
            if time.monotonic()>deadline or daemon.poll() is not None:
                raise RuntimeError('Private IBus unavailable')
            time.sleep(.05)
        if os.environ.get('TYPOMORPH_CLIENT_TRANSACTION') != '1':
            factory = subprocess.Popen(['/usr/bin/python3', str(Path(__file__).resolve().parents[2]/'ibus'/'gtk_fixture.py')],
                                       stdout=subprocess.PIPE, text=True)
            import selectors
            with selectors.DefaultSelector() as selector:
                selector.register(factory.stdout, selectors.EVENT_READ)
                if not selector.select(5) or factory.stdout.readline().strip() != 'READY':
                    raise RuntimeError('Native fixture unavailable')
        return run_shell(runtime)
    finally:
        for child in (factory, daemon):
            if child is not None:
                child.terminate()
                try:
                    child.wait(3)
                except subprocess.TimeoutExpired:
                    child.kill(); child.wait()
                if child.stdout:
                    child.stdout.close()


def run_shell(runtime):
    shell = subprocess.Popen(['gnome-shell', '--headless', '--no-x11',
        '--virtual-monitor=1024x768', '--wayland-display=typomorph-test'],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        deadline = time.monotonic()+12
        while shell.poll() is None and time.monotonic()<deadline:
            if Path(runtime, 'typomorph-test').exists():
                print('private_wayland_socket_ready=True', flush=True)
                time.sleep(5)
                print('compositor_still_running=' + str(shell.poll() is None), flush=True)
                if shell.poll() is not None:
                    return 2
                env = dict(os.environ, WAYLAND_DISPLAY='typomorph-test', GDK_BACKEND='wayland',
                           TYPOMORPH_SHELL_PID=str(shell.pid))
                return subprocess.run(['/usr/bin/python3', str(Path(__file__).with_name(
                    'test_client_transaction.py' if os.environ.get('TYPOMORPH_CLIENT_TRANSACTION') == '1'
                    else 'test_wayland_route.py'))],
                                      env=env, timeout=40).returncode
            time.sleep(.1)
        print('private_wayland_socket_ready=False', flush=True)
        return 2
    finally:
        shell.terminate()
        try:
            output, _ = shell.communicate(timeout=3)
        except subprocess.TimeoutExpired:
            shell.kill()
            output, _ = shell.communicate()
        # Inspect compositor startup diagnostics; never print input content.
        print('shell_startup_completed=' + str('GNOME Shell started at' in output), flush=True)
        print('optional_private_services_missing=' + str('ServiceUnknown' in output), flush=True)
        if 'GNOME Shell started at' not in output:
            print(output[-3000:])
            raise RuntimeError('Shell startup marker absent')
        if 'Execution of main.js threw exception' in output or 'JS ERROR' in output:
            raise RuntimeError('Shell initialization failed; socket readiness is insufficient')


def check_environment():
    runtime = os.environ.get('XDG_RUNTIME_DIR', '')
    if (os.environ.get('TYPOMORPH_PRIVATE_WAYLAND') != '1'
        or not runtime.startswith('/tmp/typomorph-wayland-')
        or os.environ.get('DBUS_SYSTEM_BUS_ADDRESS') not in ('unix:path='+runtime+'/no-system-bus', os.environ.get('DBUS_SESSION_BUS_ADDRESS'))
        or os.environ.get('DISPLAY') or os.environ.get('WAYLAND_DISPLAY')
        or os.environ.get('WAYLAND_SOCKET') or os.environ.get('GSETTINGS_BACKEND') != 'memory'):
        raise SystemExit('Private preflight required')


if __name__ == '__main__':
    raise SystemExit(main())
