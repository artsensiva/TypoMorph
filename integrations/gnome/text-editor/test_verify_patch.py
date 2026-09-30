#!/usr/bin/env python3
"""Integrity-check regressions; these do not execute or validate the C probe."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def main():
    if len(sys.argv) != 2:
        raise SystemExit('Usage: test_verify_patch.py /path/to/pinned/upstream')
    source = Path(sys.argv[1]).resolve()
    here = Path(__file__).resolve().parent
    hashes = json.loads((here / 'upstream-files.sha256.json').read_text())
    def check(condition, message):
        if not condition:
            raise RuntimeError(message)
    def contents(root):
        return {str(p.relative_to(root)): p.read_bytes()
                for p in root.rglob('*') if p.is_file()}
    with tempfile.TemporaryDirectory(prefix='typomorph-verifier-tests-') as tmp:
        tree = Path(tmp) / 'source'
        for name, digest in hashes.items():
            data = (source / name).read_bytes()
            check(hashlib.sha256(data).hexdigest() == digest, 'Unpinned test input')
            target = tree / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        tools = Path(tmp) / 'tools'
        tools.mkdir()
        for name in ('verify_patch.py', 'upstream-files.sha256.json',
                     '0001-private-owner-range-probe.patch'):
            shutil.copyfile(here / name, tools / name)
        def run(success, expected=''):
            before = contents(tree)
            result = subprocess.run([sys.executable, '-O', str(tools / 'verify_patch.py'), str(tree)],
                                    capture_output=True, text=True)
            check((result.returncode == 0) == success, 'Unexpected verifier exit: ' + result.stderr)
            check(expected in result.stdout + result.stderr, 'Missing expected diagnostic')
            check(contents(tree) == before, 'Verifier mutated its input source tree')
        run(True, 'exact restoration passed')
        print('PASS valid patch and immutable source under Python -O')
        target = tree / 'meson_options.txt'
        original = target.read_bytes()
        target.write_bytes(original + b'\n# altered fixture\n')
        run(False, 'Source mismatch: meson_options.txt')
        target.write_bytes(original)
        print('PASS changed upstream refused without mutation')
        patch = tools / '0001-private-owner-range-probe.patch'
        original_patch = patch.read_bytes()
        patch.write_text('invalid patch fixture\n')
        run(False)
        patch.write_bytes(original_patch)
        print('PASS corrupt patch refused without mutation')
    print('Three verifier regression checks passed; no C build/runtime checks performed.')


if __name__ == '__main__':
    main()
