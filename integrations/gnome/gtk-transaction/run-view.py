#!/usr/bin/env python3
"""Run the fixed view-refusal probe on private Unix sockets, not the desktop."""
import argparse
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--server', type=Path, required=True)
    parser.add_argument('--probe', type=Path, required=True)
    parser.add_argument('--check-leaks', action='store_true',
                        help='Also check process-exit leaks (font stack currently reports leaks)')
    args = parser.parse_args()
    server_path = args.server.resolve(strict=True)
    probe_path = args.probe.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix='typomorph-view-') as directory:
        root = Path(directory)
        env = {
            'PATH': '/usr/bin:/bin', 'HOME': directory, 'LC_ALL': 'C.UTF-8',
            'XDG_CONFIG_HOME': directory, 'XDG_CACHE_HOME': directory,
            'XDG_DATA_HOME': directory, 'XDG_RUNTIME_DIR': directory,
            'GSETTINGS_BACKEND': 'memory', 'G_DEBUG': 'fatal-warnings',
            'GTK_A11Y': 'none', 'GDK_BACKEND': 'broadway',
            'BROADWAY_DISPLAY': ':91',
            'DBUS_SYSTEM_BUS_ADDRESS': 'unix:path=' + directory + '/absent',
            'ASAN_OPTIONS': f'detect_leaks={int(args.check_leaks)}:halt_on_error=1',
            'UBSAN_OPTIONS': 'halt_on_error=1:print_stacktrace=1',
        }
        config = root / 'bus.conf'
        config.write_text('<busconfig><type>session</type><listen>unix:tmpdir=' +
                          directory + '</listen><auth>EXTERNAL</auth>'
                          '<policy context="default"><allow send_destination="*"/>'
                          '<allow receive_sender="*"/><allow own="*"/>'
                          '</policy></busconfig>')
        children = []
        try:
            server = subprocess.Popen([str(server_path), '-u',
                                       str(root / 'http.socket'), ':91'],
                                      env=env, start_new_session=True)
            children.append(server)
            deadline = time.monotonic() + 10
            while not (root / 'broadway92.socket').exists():
                if server.poll() is not None or time.monotonic() >= deadline:
                    raise RuntimeError('Private Broadway display did not become ready')
                time.sleep(0.05)
            probe = subprocess.Popen(['dbus-run-session', '--config-file=' + str(config),
                                      '--', str(probe_path)], env=env, start_new_session=True)
            children.append(probe)
            return probe.wait(timeout=30)
        finally:
            for child in reversed(children):
                try:
                    os.killpg(child.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    child.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    pass
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                child.wait(timeout=3)


if __name__ == '__main__':
    raise SystemExit(main())
