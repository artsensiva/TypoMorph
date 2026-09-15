#!/usr/bin/env bash
set -euo pipefail

REPO="artsensiva/TypoMorph"
GITHUB_API="https://api.github.com/repos/${REPO}"
GITHUB_RELEASES="https://github.com/${REPO}/releases/download"

TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

BIN_DIR="${HOME}/.local/bin"

fail_to_source_build() {
  cat >&2 <<EOF

Could not install a prebuilt package automatically.
Build TypoMorph from source instead:

  git clone https://github.com/${REPO}.git
  cd TypoMorph
  cargo build --release -p daemon
  ./scripts/build-deb.sh   # optional: produces target/debian/typomorph_*.deb
  sudo apt install ./target/debian/typomorph_*.deb

EOF
  exit 1
}

if [[ "${EUID}" -eq 0 ]]; then
  echo "Run this installer as your normal user; it configures a systemd user service." >&2
  exit 1
fi

command -v curl >/dev/null 2>&1 || { echo "Missing dependency: curl" >&2; exit 1; }
command -v systemctl >/dev/null 2>&1 || { echo "Missing dependency: systemd/systemctl" >&2; exit 1; }

if [[ ! -r /dev/uinput ]]; then
  echo "Warning: /dev/uinput is not readable yet. Install the package to apply udev rules, then log in again if needed." >&2
fi

TAG="${TYPO_TAG:-}"
if [[ -z "$TAG" ]]; then
  echo "Looking up latest release..."
  TAG="$(curl --fail --silent --show-error "${GITHUB_API}/releases/latest" \
    | grep -o '"tag_name" *: *"[^"]*"' | head -1 | cut -d'"' -f4)" || fail_to_source_build
fi
[[ -n "$TAG" ]] || fail_to_source_build

VERSION="${TAG#v}"
DEB_NAME="typomorph_${VERSION}_amd64.deb"
DEB_PATH="${TMP_DIR}/${DEB_NAME}"
SUMS_PATH="${TMP_DIR}/SHA256SUMS"

echo "Downloading TypoMorph ${TAG}..."
curl --fail --location --silent --show-error \
  "${GITHUB_RELEASES}/${TAG}/${DEB_NAME}" --output "$DEB_PATH" || fail_to_source_build
curl --fail --location --silent --show-error \
  "${GITHUB_RELEASES}/${TAG}/SHA256SUMS" --output "$SUMS_PATH" || fail_to_source_build

echo "Verifying checksum..."
(cd "$TMP_DIR" && grep " ${DEB_NAME}\$" SHA256SUMS | sha256sum -c -) || {
  echo "Checksum verification failed; aborting." >&2
  exit 1
}

if command -v apt-get >/dev/null 2>&1 && command -v dpkg >/dev/null 2>&1; then
  echo "Installing Debian package..."
  if sudo -n true 2>/dev/null; then
    sudo apt-get install -y "$DEB_PATH"
  else
    echo "sudo access is required to install the .deb and udev rule." >&2
    sudo apt-get install -y "$DEB_PATH"
  fi
elif command -v dpkg-deb >/dev/null 2>&1; then
  echo "apt/dpkg unavailable; installing the binary to ${BIN_DIR}..."
  mkdir -p "$BIN_DIR"
  dpkg-deb --extract "$DEB_PATH" "$TMP_DIR/root"
  install -m 0755 "$TMP_DIR/root/usr/bin/typomorph" "$BIN_DIR/typomorph"
  echo "Add ${BIN_DIR} to PATH if it is not already present."
else
  fail_to_source_build
fi

systemctl --user daemon-reload
systemctl --user enable --now typomorph.service
printf 'TypoMorph is installed and the user service is active.\n'
