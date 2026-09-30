"""Installer orchestration tests: all network, signing and install commands are fakes.
These prove refusal/ordering, not real Sigstore verification or package compatibility.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "typomorph_1.0.0_amd64.deb"

@unittest.skipIf(os.geteuid() == 0, "installer deliberately refuses root")
class InstallerTests(unittest.TestCase):
    def exercise(self, *, signature_ok=True, tampered=False, tag="v1.0.0", duplicate=False, install=True):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "bin"
            binary.mkdir()
            fixture = root / "fixture"
            fixture.mkdir()
            payload = b"synthetic package: never installed"
            (fixture / PACKAGE).write_bytes(payload + (b"tampered" if tampered else b""))
            (fixture / "SHA256SUMS").write_text(hashlib.sha256(payload).hexdigest() + "  " + PACKAGE + "\n")
            (fixture / "SHA256SUMS.bundle.json").write_text("synthetic signature result")
            assets = [{"name": x} for x in [PACKAGE, "SHA256SUMS", "SHA256SUMS.bundle.json"]]
            if duplicate:
                assets.append({"name": PACKAGE})
            (fixture / "release.json").write_text(json.dumps({"tag_name": tag, "draft": False,
                "prerelease": False, "assets": assets}))
            driver = f'''#!{sys.executable}
import json, os, pathlib, shutil, sys
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ["CALLS"], "a") as stream:
    stream.write(json.dumps([name, args]) + "\\n")
if name == "dpkg":
    assert args == ["--print-architecture"]
    print("amd64")
elif name == "curl":
    assert "--proto" in args and "--proto-redir" in args and "--max-filesize" in args
    url = args[args.index("--output") - 1]
    asset = "release.json" if url.endswith("/latest") else url.rsplit("/", 1)[1]
    assert url.startswith("https://")
    shutil.copyfile(pathlib.Path(os.environ["FIXTURE"]) / asset, args[args.index("--output") + 1])
elif name == "cosign":
    assert args[0] == "verify-blob"
    assert args[args.index("--certificate-identity") + 1] == "https://github.com/artsensiva/TypoMorph/.github/workflows/release.yml@refs/tags/v1.0.0"
    assert args[args.index("--certificate-oidc-issuer") + 1] == "https://token.actions.githubusercontent.com"
    assert not any("ignore" in arg or "insecure" in arg for arg in args)
    sys.exit(0 if os.environ["SIGNATURE_OK"] == "1" else 1)
elif name == "sudo":
    assert args[:3] == ["apt-get", "install", "--"]
    assert pathlib.Path(args[3]).name == "{PACKAGE}"
else:
    sys.exit("Unexpected external command")
'''
            for name in ["curl", "cosign", "sudo", "apt-get", "dpkg", "systemctl", "udevadm"]:
                path = binary / name
                path.write_text(driver)
                path.chmod(0o755)
            calls = root / "calls"
            env = dict(os.environ, PATH=str(binary) + os.pathsep + os.environ["PATH"],
                FIXTURE=str(fixture), CALLS=str(calls), SIGNATURE_OK="1" if signature_ok else "0")
            result = subprocess.run(["bash", str(ROOT / "landing/install.sh")] + (["--install"] if install else []),
                env=env, capture_output=True, text=True, timeout=15)
            records = [json.loads(x) for x in calls.read_text().splitlines()] if calls.exists() else []
            return result, records

    def test_verified_flow_installs_once_without_starting_or_granting_access(self):
        result, calls = self.exercise()
        self.assertEqual(result.returncode, 0, result.stderr)
        names = [x[0] for x in calls]
        self.assertEqual(names.count("sudo"), 1)
        self.assertLess(names.index("cosign"), names.index("sudo"))
        self.assertNotIn("systemctl", names)
        self.assertNotIn("udevadm", names)

    def test_failed_signature_never_downloads_package_or_installs(self):
        result, calls = self.exercise(signature_ok=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(x[0] == "sudo" for x in calls))
        self.assertFalse(any(x[0] == "curl" and any(a.endswith("/" + PACKAGE) for a in x[1]) for x in calls))

    def test_tampered_package_cannot_reach_installation(self):
        result, calls = self.exercise(tampered=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("checksum verification failed", result.stderr)
        self.assertFalse(any(x[0] == "sudo" for x in calls))

    def test_legacy_or_ambiguous_release_is_refused(self):
        for arguments in [{"tag": "v0.2.2"}, {"tag": "v1.0.0/../../main"}, {"duplicate": True}]:
            result, calls = self.exercise(**arguments)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any(x[0] in ("cosign", "sudo") for x in calls))

    def test_explicit_install_argument_required_before_external_commands(self):
        result, calls = self.exercise(install=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, [])

if __name__ == "__main__":
    unittest.main()
