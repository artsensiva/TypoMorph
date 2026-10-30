#!/usr/bin/env python3
"""Compile and test fixed replacement notifications on private Wayland/AT-SPI."""
import argparse
import os
from pathlib import Path
import runpy
import signal
import subprocess
import sys
import tempfile
import time


def stop(child):
    if child.poll() is None:
        child.terminate()
        try:
            child.wait(timeout=2)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()


def wait_socket(child, path):
    deadline = time.monotonic() + 15
    while not path.exists():
        if child.poll() is not None or time.monotonic() >= deadline:
            raise RuntimeError('Private service did not become ready')
        time.sleep(0.05)


def inside(probe, scenario):
    runtime = os.environ.get('XDG_RUNTIME_DIR', '')
    address = os.environ.get('DBUS_SESSION_BUS_ADDRESS', '')
    if (not runtime.startswith('/tmp/typomorph-a11y-') or runtime not in address
            or any(os.environ.get(k) for k in ('DISPLAY', 'WAYLAND_DISPLAY', 'WAYLAND_SOCKET'))):
        raise RuntimeError('Only the private probe runner is supported')
    env = dict(os.environ, AT_SPI_BUS_ADDRESS=address, DBUS_SYSTEM_BUS_ADDRESS=address)
    children = []
    try:
        ibus = subprocess.Popen([
            'ibus-daemon', '--single', '--panel=disable', '--config=disable',
            '--emoji-extension=disable', '--cache=none', '--address=' + env['IBUS_ADDRESS'],
        ], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        children.append(ibus)
        wait_socket(ibus, Path(runtime) / 'ibus')
        shell = subprocess.Popen([
            'gnome-shell', '--headless', '--no-x11', '--virtual-monitor=1024x768',
            '--wayland-display=typomorph-a11y',
        ], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        children.append(shell)
        wait_socket(shell, Path(runtime) / 'typomorph-a11y')
        registry = subprocess.Popen(['/usr/libexec/at-spi2-registryd'], env=env)
        children.append(registry)
        deadline = time.monotonic() + 5
        while True:
            reply = subprocess.run([
                'gdbus', 'call', '--session', '--dest', 'org.freedesktop.DBus',
                '--object-path', '/org/freedesktop/DBus', '--method',
                'org.freedesktop.DBus.NameHasOwner', 'org.a11y.atspi.Registry',
            ], env=env, capture_output=True, text=True, timeout=2, check=True)
            if 'true' in reply.stdout:
                break
            if registry.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError('Private accessibility registry did not become ready')
            time.sleep(0.05)
        probe_env = dict(env, GTK_A11Y='atspi', GDK_BACKEND='wayland', G_DEBUG='fatal-warnings',
                         WAYLAND_DISPLAY='typomorph-a11y')
        argv = [str(probe)] + ([scenario] if scenario else [])
        return subprocess.run(argv, env=probe_env, timeout=20).returncode
    finally:
        for child in reversed(children):
            stop(child)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build', type=Path)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--test', choices=('test-a11y.c', 'test-a11y-view.c', 'test-a11y-natural.c'),
                        default='test-a11y.c', help='Fixed fixture in this directory')
    parser.add_argument('--scenario', choices=('primary', 'late-added', 'listener-gap', 'listener-gap-timeout',
                                    'listener-gap-settled'),
                        help='Fresh-process scenario for test-a11y-natural.c')
    parser.add_argument('--inside', type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    if (args.test == 'test-a11y-natural.c') != (args.scenario is not None) and not args.inside:
        parser.error('--scenario is required for, and only valid with, test-a11y-natural.c')
    if args.inside:
        return inside(args.inside.resolve(strict=True), args.scenario)
    if args.build is None or args.source is None:
        parser.error('--build and --source are required')
    build, source = args.build.resolve(strict=True), args.source.resolve(strict=True)
    if not all(path.is_relative_to('/tmp') for path in (build, source)):
        parser.error('Only disposable source/build directories under /tmp are accepted')
    here = Path(__file__).resolve().parent
    compiler = runpy.run_path(str(here / 'run-layout.py'))['build_probe']
    with tempfile.TemporaryDirectory(prefix='typomorph-a11y-') as temp:
        root = Path(temp)
        probe = root / 'test-a11y'
        compiler(build, source, root / 'test.o', probe, here / args.test)
        env = dict(PATH='/usr/bin:/bin', HOME=temp, LANG='C.UTF-8',
                   XDG_RUNTIME_DIR=temp, XDG_CONFIG_HOME=temp, XDG_CACHE_HOME=temp,
                   XDG_DATA_HOME=temp, XDG_STATE_HOME=temp, GSETTINGS_BACKEND='memory',
                   GIO_USE_VFS='local', GTK_A11Y='none', LIBGL_ALWAYS_SOFTWARE='1',
                   GNOME_SHELL_SESSION_MODE='user', IBUS_ADDRESS='unix:path=' + temp + '/ibus',
                   ASAN_OPTIONS='detect_leaks=0:halt_on_error=1',
                   UBSAN_OPTIONS='halt_on_error=1:print_stacktrace=1')
        config = root / 'bus.conf'
        config.write_text('<busconfig><type>session</type><listen>unix:tmpdir=' + temp +
                          '</listen><auth>EXTERNAL</auth><policy context="default">'
                          '<allow send_destination="*"/><allow receive_sender="*"/>'
                          '<allow own="*"/></policy></busconfig>')
        child = subprocess.Popen([
            'dbus-run-session', '--config-file=' + str(config), '--', sys.executable,
            str(Path(__file__).resolve()), '--inside', str(probe),
            *(['--scenario', args.scenario] if args.scenario else []),
        ], env=env, start_new_session=True)
        try:
            return child.wait(timeout=55)
        finally:
            for sig in (signal.SIGTERM, signal.SIGKILL):
                try:
                    os.killpg(child.pid, sig)
                except ProcessLookupError:
                    pass
                try:
                    child.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    pass
            child.wait(timeout=2)


if __name__ == '__main__':
    raise SystemExit(main())
