#!/usr/bin/env python3
"""Verify pinned-source patch reproducibility and refusal without building GTK."""
import argparse
import hashlib
from pathlib import Path
import runpy
import subprocess
import sys
import tarfile
import tempfile

ARCHIVE_SHA256 = '51bd9f60c7d23a665a556c7364c21fb2e4e282566b3e7e092455e8f910330893'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--compare-source', type=Path)
    args = parser.parse_args()
    with args.archive.open('rb') as stream:
        if hashlib.file_digest(stream, 'sha256').hexdigest() != ARCHIVE_SHA256:
            parser.error('Release archive identity mismatch')
    script = Path(__file__).with_name('prepare.py')
    names = runpy.run_path(str(script))['HASHES']
    with tempfile.TemporaryDirectory(prefix='typomorph-repro-') as directory:
        roots = [Path(directory) / name for name in ('first', 'second', 'modified')]
        with tarfile.open(args.archive) as archive:
            for root in roots:
                for name in names:
                    path = root / 'gtk' / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(archive.extractfile('gtk-4.22.4/gtk/' + name).read())

        def snapshot(root):
            return {name: (root / 'gtk' / name).read_bytes() for name in names}

        def patch(root, expected_success):
            result = subprocess.run([sys.executable, str(script), str(root)],
                                    capture_output=True, timeout=15)
            if (result.returncode == 0) != expected_success:
                raise RuntimeError('Unexpected source-preparer result')

        for root in roots[:2]:
            patch(root, True)
        if snapshot(roots[0]) != snapshot(roots[1]):
            raise RuntimeError('Patched output is not reproducible')
        before = snapshot(roots[0])
        patch(roots[0], False)
        if snapshot(roots[0]) != before:
            raise RuntimeError('Repeated patch changed source')
        changed = roots[2] / 'gtk' / next(iter(names))
        changed.write_bytes(changed.read_bytes() + b'\n/* modified fixture */\n')
        before_modified = snapshot(roots[2])
        patch(roots[2], False)
        if snapshot(roots[2]) != before_modified:
            raise RuntimeError('Identity refusal changed source')
        if args.compare_source and snapshot(args.compare_source) != before:
            raise RuntimeError('Build source differs from reproducible output')
        print(f'{len(names)} files: reproducible; repeat and modified-source refusals preserve bytes')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
