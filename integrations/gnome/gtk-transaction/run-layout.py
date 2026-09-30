#!/usr/bin/env python3
"""Build and run fixed layout-refusal tests against the disposable GTK archive."""
import argparse
from pathlib import Path
import shlex
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    args = parser.parse_args()
    build = args.build.resolve(strict=True)
    source = args.source.resolve(strict=True)
    if not all(path.is_relative_to('/tmp') for path in (build, source)):
        parser.error('Only disposable source/build directories under /tmp are accepted')
    with tempfile.TemporaryDirectory(prefix='typomorph-layout-') as directory:
        root = Path(directory)
        obj = root / 'test.o'
        binary = root / 'test-layout'
        flags = shlex.split(subprocess.check_output(
            ['pkg-config', '--cflags', 'gtk4'], text=True, timeout=10))
        subprocess.run([
            '/usr/bin/gcc', '-std=c11', '-Wall', '-Wextra', '-Werror',
            '-fsanitize=address,undefined', '-fno-omit-frame-pointer',
            '-I' + str(build), '-I' + str(source), *flags, '-c',
            str(Path(__file__).with_name('test-layout.c')), '-o', str(obj),
        ], check=True, timeout=30)
        # Private GTK symbols require the same archive dependencies as its tests.
        commands = subprocess.check_output(
            ['ninja', '-t', 'commands', 'testsuite/gtk/textbuffer'],
            cwd=build, text=True, timeout=10)
        link = shlex.split(commands.splitlines()[-1])
        link[link.index('-o') + 1] = str(binary)
        link[link.index('testsuite/gtk/textbuffer.p/textbuffer.c.o')] = str(obj)
        subprocess.run(link, cwd=build, check=True, timeout=30)
        env = {
            'PATH': '/usr/bin:/bin', 'HOME': directory, 'LC_ALL': 'C.UTF-8',
            'XDG_CONFIG_HOME': directory, 'XDG_CACHE_HOME': directory,
            'XDG_DATA_HOME': directory, 'GSETTINGS_BACKEND': 'memory',
            'G_DEBUG': 'fatal-warnings',
            # Font-stack process-exit allocations remain a recorded limitation.
            'ASAN_OPTIONS': 'detect_leaks=0:halt_on_error=1',
            'UBSAN_OPTIONS': 'halt_on_error=1:print_stacktrace=1',
        }
        return subprocess.run([str(binary)], env=env, timeout=30).returncode


if __name__ == '__main__':
    raise SystemExit(main())
