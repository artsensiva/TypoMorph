#!/usr/bin/env bash
set -euo pipefail

VERSION="${TYPO_VERSION:-0.1.0}"
BASE_URL="${TYPO_BASE_URL:-https://vintage-turntables.com/downloads}"
PACKAGE_URL="${BASE_URL}/typomorph_${VERSION}_amd64.deb"
TMP_DIR="$(mktemp -d)"
PACKAGE_PATH="${TMP_DIR}/typomorph_${VERSION}_amd64.deb"
BIN_DIR="${HOME}/.local/bin"

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

if [[ "${EUID}" -eq 0 ]]; then
  echo "Run this installer as your normal user; it configures a systemd user service." >&2
  exit 1
fi

command -v curl >/dev/null 2>&1 || { echo "Missing dependency: curl" >&2; exit 1; }
command -v systemctl >/dev/null 2>&1 || { echo "Missing dependency: systemd/systemctl" >&2; exit 1; }

if [[ ! -r /dev/uinput ]]; then
  echo "Warning: /dev/uinput is not readable yet. Install the package to apply udev rules, then log in again if needed." >&2
fi

mkdir -p "$TMP_DIR"
echo "Downloading TypoMorph ${VERSION}..."
curl --fail --location --silent --show-error "$PACKAGE_URL" --output "$PACKAGE_PATH"

if command -v apt-get >/dev/null 2>&1 && command -v dpkg >/dev/null 2>&1; then
  echo "Installing Debian package..."
  if sudo -n true 2>/dev/null; then
    sudo apt-get install -y "$PACKAGE_PATH"
  else
    echo "sudo access is required to install the .deb and udev rule." >&2
    sudo apt-get install -y "$PACKAGE_PATH"
  fi
elif command -v dpkg-deb >/dev/null 2>&1; then
  echo "apt/dpkg unavailable; installing the binary to ${BIN_DIR}..."
  mkdir -p "$BIN_DIR"
  dpkg-deb --extract "$PACKAGE_PATH" "$TMP_DIR/root"
  install -m 0755 "$TMP_DIR/root/usr/bin/typomorph" "$BIN_DIR/typomorph"
  echo "Add ${BIN_DIR} to PATH if it is not already present."
else
  echo "Neither apt/dpkg nor dpkg-deb is available; cannot install TypoMorph." >&2
  exit 1
fi

systemctl --user daemon-reload
systemctl --user enable --now typomorph.service
printf 'TypoMorph is installed and the user service is active.\n'
