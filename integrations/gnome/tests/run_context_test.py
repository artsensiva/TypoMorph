"""Exercise the metadata reader against a synthetic AT-SPI tree on a private bus."""
import os
from pathlib import Path
import selectors
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]

if __name__ == "__main__":
    if len(sys.argv) == 1:
        env = dict(os.environ, TYPOMORPH_PRIVATE_CONTEXT_TEST="1", GIO_USE_VFS="local")
        result = subprocess.run(
            ["dbus-run-session", "--", sys.executable, str(Path(__file__).resolve()), "--inside"],
            cwd=ROOT, env=env, timeout=120,
        )
        raise SystemExit(result.returncode)

    if sys.argv[1:] != ["--inside"] or os.environ.get("TYPOMORPH_PRIVATE_CONTEXT_TEST") != "1":
        raise SystemExit("Use the runner without arguments to create a private bus.")
    process = subprocess.Popen(
        ["gjs", "-m", str(Path(__file__).with_name("fake-context.mjs"))],
        cwd=ROOT, stdout=subprocess.PIPE, text=True,
    )
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            if not selector.select(timeout=10) or process.stdout.readline().strip() != "READY":
                raise RuntimeError("Synthetic GJS bridge did not become ready")
        result = subprocess.run(
            ["cargo", "test", "--offline", "-p", "platform-linux", "--test",
             "input_context", "--", "--ignored", "--test-threads=1"],
            cwd=ROOT, timeout=90,
        )
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
    raise SystemExit(result.returncode)
