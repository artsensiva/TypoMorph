#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v cargo-deb >/dev/null 2>&1; then
    echo "cargo-deb is not installed; installing it with cargo..." >&2
    cargo install cargo-deb
fi

cargo build --release -p daemon -p native-host
cargo deb -p daemon

shopt -s nullglob
packages=(target/debian/typomorph_*.deb)
if ((${#packages[@]} == 0)); then
    echo "cargo-deb did not produce target/debian/typomorph_*.deb" >&2
    exit 1
fi

printf 'Built package: %s\n' "${packages[@]}"
