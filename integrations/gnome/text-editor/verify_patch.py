#!/usr/bin/env python3
"""Verify pinned input files and round-trip the patch in a disposable directory.

No network, compilation, source-tree mutation or package installation.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    if len(sys.argv) != 2:
        raise SystemExit('Usage: verify_patch.py /path/to/gnome-text-editor-50.1')
    source = Path(sys.argv[1]).resolve()
    here = Path(__file__).resolve().parent
    hashes = json.loads((here / 'upstream-files.sha256.json').read_text())
    patches = [here / '0001-private-owner-range-probe.patch',
               here / '0002-subclass-boundary-probe.patch']
    originals = {}
    for name, expected in hashes.items():
        data = (source / name).read_bytes()
        if hashlib.sha256(data).hexdigest() != expected:
            raise SystemExit('Source mismatch: ' + name)
        originals[name] = data
    with tempfile.TemporaryDirectory(prefix='typomorph-patch-check-') as directory:
        root = Path(directory)
        for name, data in originals.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        def apply(patch, *options):
            subprocess.run(['git', '-C', str(root), 'apply', *options, str(patch)], check=True)
        # Patches are an ordered series: 0002 modifies a file created by 0001.
        for patch in patches:
            apply(patch, '--check', '--whitespace=error-all')
            apply(patch, '--whitespace=error-all')
        for name in ('editor-typomorph-owner.c', 'editor-typomorph-owner.h'):
            if not (root / 'src' / name).is_file():
                raise SystemExit('Patch did not create: ' + name)
        for patch in reversed(patches):
            apply(patch, '--reverse', '--check')
            apply(patch, '--reverse')
        restored = {str(p.relative_to(root)): p.read_bytes()
                    for p in root.rglob('*') if p.is_file()}
        if restored != originals:
            raise SystemExit('Reverse application did not restore source exactly')
    print('Pinned files verified; forward/reverse application and exact restoration passed.')
    print('C compilation, application startup and runtime safety were NOT tested.')


if __name__ == '__main__':
    main()
