#!/usr/bin/env bash
# Development source for the v1 installer. No public v1 artifact is certified yet.
set -euo pipefail

if [[ "${1:-}" != "--install" || $# -ne 1 ]]; then
  echo "Usage: bash install.sh --install" >&2
  echo "Downloads and verifies a stable v1 package before requesting installation." >&2
  echo "Does not start TypoMorph, enable autostart, or grant input-device access." >&2
  exit 2
fi
if [[ "${EUID}" -eq 0 ]]; then
  echo "Run as your normal user; apt will request elevation after verification." >&2
  exit 1
fi
for dependency in curl python3 cosign dpkg apt-get sudo; do
  command -v "$dependency" >/dev/null 2>&1 || { echo "Missing dependency: $dependency" >&2; exit 1; }
done
[[ "$(dpkg --print-architecture)" == "amd64" ]] || { echo "This installer requires Debian/Ubuntu amd64." >&2; exit 1; }
REPO="artsensiva/TypoMorph"
API="https://api.github.com/repos/${REPO}/releases/latest"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
fetch() {
  curl --fail --location --silent --show-error --proto '=https' --proto-redir '=https' \
    --connect-timeout 15 --max-time 300 --max-filesize "$3" "$1" --output "$2"
}
fetch "$API" "$STAGE/release.json" 1048576
python3 - "$STAGE/release.json" > "$STAGE/selection" <<'PY'
import json, re, sys
try:
    with open(sys.argv[1], encoding="utf-8") as stream:
        release = json.load(stream)
    tag = release["tag_name"]
    if release.get("draft") is not False or release.get("prerelease") is not False:
        raise ValueError()
    if not isinstance(tag, str) or not re.fullmatch(r"v[1-9][0-9]*\.[0-9]+\.[0-9]+", tag):
        raise ValueError()
    expected = f"typomorph_{tag[1:]}_amd64.deb"
    names = [asset["name"] for asset in release["assets"]]
    for name in (expected, "SHA256SUMS", "SHA256SUMS.bundle.json"):
        if names.count(name) != 1:
            raise ValueError()
    print(tag)
    print(expected)
except (ValueError, KeyError, TypeError, OSError):
    sys.exit("No supported stable v1 package with signing evidence is available.")
PY
mapfile -t selection < "$STAGE/selection"
TAG="${selection[0]}"
PACKAGE="${selection[1]}"
BASE="https://github.com/${REPO}/releases/download/${TAG}"
fetch "${BASE}/SHA256SUMS" "$STAGE/SHA256SUMS" 1048576
fetch "${BASE}/SHA256SUMS.bundle.json" "$STAGE/SHA256SUMS.bundle.json" 1048576
# Exact repository, workflow, tag, and issuer; no unsigned or verification-skip fallback.
cosign verify-blob "$STAGE/SHA256SUMS" --bundle "$STAGE/SHA256SUMS.bundle.json" \
  --certificate-identity "https://github.com/${REPO}/.github/workflows/release.yml@refs/tags/${TAG}" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com"
fetch "${BASE}/${PACKAGE}" "$STAGE/$PACKAGE" 268435456
python3 - "$STAGE" "$PACKAGE" <<'PY'
import hashlib, pathlib, re, sys
try:
    root, name = pathlib.Path(sys.argv[1]), sys.argv[2]
    lines = (root / "SHA256SUMS").read_text(encoding="ascii").splitlines()
    matches = [line[:64] for line in lines
               if re.fullmatch(r"[0-9a-f]{64}  " + re.escape(name), line)]
    if len(matches) != 1:
        raise ValueError()
    digest = hashlib.sha256()
    with (root / name).open("rb") as stream:
        for block in iter(lambda: stream.read(65536), b""):
            digest.update(block)
    if digest.hexdigest() != matches[0]:
        raise ValueError()
except (ValueError, OSError, UnicodeError):
    sys.exit("Package checksum verification failed; installation refused.")
PY
# --install is consent to this package only. apt still handles its normal confirmation.
sudo apt-get install -- "$STAGE/$PACKAGE"
echo "Package installed. TypoMorph has not been started; autostart remains opt-in."
