#!/usr/bin/env bash
set -euo pipefail
project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "$project_dir/target/gnome"
gnome-extensions pack --force --extra-source=service.mjs \
    --out-dir="$project_dir/target/gnome" \
    "$project_dir/integrations/gnome/layout-bridge@typomorph.com"
