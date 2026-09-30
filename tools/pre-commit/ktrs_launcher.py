"""`ktrs`, `ktfmt` and `ktlint` commands that run the release binaries for this package's version, downloaded
(and checked against SHA256SUMS) into the Python environment on first use. pre-commit installs this
package from the repository root for the hooks in .pre-commit-hooks.yaml; no Rust toolchain needed.
KTRS_VERSION (a tag, e.g. v0.2.0) overrides the version.
"""

import hashlib
import io
import os
import platform
import subprocess
import sys
import tarfile
import urllib.request
import zipfile
from importlib.metadata import version as package_version
from pathlib import Path

RELEASES = "https://github.com/Hexay/ktrs/releases/download"
OS_TARGETS = {"Linux": "unknown-linux-musl", "Darwin": "apple-darwin", "Windows": "pc-windows-msvc"}


def target():
    system = platform.system()
    if system not in OS_TARGETS:
        sys.exit(f"ktrs: no prebuilt binary for {system}; install it with `cargo install ktrs`")
    arch = "aarch64" if platform.machine().lower() in ("arm64", "aarch64") else "x86_64"
    return f"{arch}-{OS_TARGETS[system]}"


def fetch(url):
    with urllib.request.urlopen(url, timeout=60) as response:
        return response.read()


def install(tag, home):
    triple = target()
    archive = f"ktrs-{tag}-{triple}.{'zip' if 'windows' in triple else 'tar.gz'}"
    data = fetch(f"{RELEASES}/{tag}/{archive}")
    sums = fetch(f"{RELEASES}/{tag}/SHA256SUMS").decode()
    expected = next((line.split()[0] for line in sums.splitlines() if line.split()[1:] == [archive]), None)
    if expected != hashlib.sha256(data).hexdigest():
        sys.exit(f"ktrs: checksum mismatch for {archive}")
    if archive.endswith(".zip"):
        with zipfile.ZipFile(io.BytesIO(data)) as zf:
            members = {Path(n).name: zf.read(n) for n in zf.namelist()}
    else:
        with tarfile.open(fileobj=io.BytesIO(data)) as tf:
            members = {Path(m.name).name: tf.extractfile(m).read() for m in tf.getmembers() if m.isfile()}
    home.mkdir(parents=True, exist_ok=True)
    for name in ("ktrs", "ktfmt", "ktlint", "ktrs.exe", "ktfmt.exe", "ktlint.exe"):
        if name in members:
            # Write then rename: a concurrent hook run never sees a partial binary.
            temp = home / f".{name}.{os.getpid()}"
            temp.write_bytes(members[name])
            temp.chmod(0o755)
            os.replace(temp, home / name)


def run(name):
    tag = os.environ.get("KTRS_VERSION") or "v" + package_version("ktrs-launcher")
    home = Path(sys.prefix) / "ktrs-bin" / tag
    path = home / (name + (".exe" if os.name == "nt" else ""))
    if not path.exists():
        install(tag, home)
    args = [str(path), *sys.argv[1:]]
    if os.name == "nt":
        sys.exit(subprocess.call(args))
    os.execv(path, args)


def ktrs():
    run("ktrs")


def ktfmt():
    run("ktfmt")


def ktlint():
    run("ktlint")
