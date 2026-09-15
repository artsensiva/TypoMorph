#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="${ROOT_DIR}/dist"

mkdir -p "$DIST_DIR"

for browser in chrome firefox; do
    src="${ROOT_DIR}/${browser}"
    out="${DIST_DIR}/typomorph-${browser}.zip"
    rm -f "$out"
    (cd "$src" && zip -qr "$out" .)
    echo "Built ${out}"
done

echo "Safari has no build output — see extensions/safari/README.md"
