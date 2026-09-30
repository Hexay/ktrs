"""End-to-end: the `ktfmt` binary vs the ktfmt 0.64 jar, run the way users run them. Linux, macOS, Windows.

    python3 tools/bench/e2e.py [--runs N] [--no-build] [--only TEXT]   (corpus: tools/fetch-corpus.sh)

Each scenario alternates the two tools `runs` times and reports the median wall time as a Markdown table.
Needs psutil for the 1-core scenario (skipped without it on Windows).
"""

import argparse
import glob
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

EXE = ".exe" if os.name == "nt" else ""
KOTLIN = (".kt", ".kts")


def find_java():
    bundled = glob.glob(f"tools/jdk/*/bin/java{EXE}")
    return bundled[0] if bundled else shutil.which("java")


def kotlin_files(root):
    return sorted(p for p in Path(root).rglob("*") if p.suffix in KOTLIN and p.is_file())


def copy_files(src_root, files, dst):
    shutil.rmtree(dst, ignore_errors=True)
    for f in files:
        target = dst / f.relative_to(src_root)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(f, target)


def pin_to_one_core():
    """How to start a child already restricted to CPU 0, or None if unsupported. The mask must be in
    place at launch: the JVM sizes its GC and JIT thread pools from the CPUs it sees at startup."""
    if hasattr(os, "sched_setaffinity"):
        return lambda: os.sched_setaffinity(0, {0})
    try:
        import psutil  # noqa: F401
    except ImportError:
        return None
    return "inherit"


def run_once(cmd, stdin_path, one_core):
    stdin = open(stdin_path, "rb") if stdin_path else subprocess.DEVNULL
    preexec = one_core if callable(one_core) else None
    parent, parent_mask = None, None
    if one_core == "inherit":
        # Windows: a child inherits its parent's affinity mask at creation.
        import psutil
        parent = psutil.Process()
        parent_mask = parent.cpu_affinity()
        parent.cpu_affinity([0])
    start = time.perf_counter()
    proc = subprocess.Popen(cmd, stdin=stdin, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, preexec_fn=preexec)
    if parent:
        parent.cpu_affinity(parent_mask)
    proc.wait()
    elapsed = time.perf_counter() - start
    if stdin_path:
        stdin.close()
    return elapsed


ONLY = None


def scenario(label, tools, runs, setup, args, stdin_path=None, one_core=None):
    if ONLY and ONLY not in label:
        return
    # One copy and an untimed warm-up per tool: fresh files cost a virus scan on first open (Windows),
    # which would dominate. After the warm-up, "format in place" re-formats already formatted files.
    setup()
    for base in tools.values():
        run_once(base + args, stdin_path, one_core)
    times = {name: [] for name in tools}
    for _ in range(runs):
        for name, base in tools.items():
            times[name].append(run_once(base + args, stdin_path, one_core))
    ours, jar = statistics.median(times["ktrs"]), statistics.median(times["jar"])
    print(f"| {label} | {fmt_time(ours)} | {fmt_time(jar)} | **{jar / ours:.0f}x** |", flush=True)


def fmt_time(seconds):
    return f"{seconds * 1000:.0f} ms" if seconds < 1 else f"{seconds:.2f} s"


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--runs", type=int, default=5)
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument("--only", help="run only the scenarios whose label contains this text")
    opts = parser.parse_args()
    global ONLY
    ONLY = opts.only
    os.chdir(Path(__file__).resolve().parents[2])

    if not opts.no_build:
        subprocess.run(["cargo", "build", "-q", "--locked", "--profile", "dist", "--bins"], check=True)
    # Windows' CreateProcess does not resolve relative paths with forward slashes.
    ours = str(Path(f"target/dist/ktfmt{EXE}").resolve())
    jar = glob.glob("tools/ktfmt-oracle/lib/ktfmt-*-with-dependencies.jar")[0]
    java = str(Path(find_java()).resolve())
    tools = {"ktrs": [ours], "jar": [java, "-jar", jar]}

    corpus = Path("corpus")
    okhttp = corpus / "okhttp"
    all_files, okhttp_files = kotlin_files(corpus), kotlin_files(okhttp)
    precommit = okhttp_files[:10]
    editor_file = next(f for f in okhttp_files if 8_000 < f.stat().st_size < 16_000)
    mb = sum(f.stat().st_size for f in all_files) / 1e6
    java_version = subprocess.run([java, "-version"], capture_output=True, text=True).stderr.splitlines()[0]
    print(f"{platform.system()} {platform.machine()}, {os.cpu_count()} logical CPUs, {platform.processor()}")
    print(f"{java_version}; corpus {len(all_files)} files, {mb:.1f} MB; okhttp {len(okhttp_files)} files")
    print(f"median of {opts.runs} runs per tool, alternating\n")
    print("| Scenario | ktrs | ktfmt 0.64 (JVM) | Speedup |")
    print("|---|---|---|---|")

    with tempfile.TemporaryDirectory() as tmp:
        tree = Path(tmp) / "tree"

        def fresh(src_root, files):
            return lambda: copy_files(src_root, files, tree)

        scenario(f"Whole corpus ({len(all_files)} files), format in place", tools, opts.runs,
                 fresh(corpus, all_files), ["--quiet", str(tree)])
        scenario(f"One project (okhttp, {len(okhttp_files)} files), format in place", tools, opts.runs,
                 fresh(okhttp, okhttp_files), ["--quiet", str(tree)])
        one_core = pin_to_one_core()
        if one_core:
            scenario("okhttp, format in place, 1 core", tools, opts.runs,
                     fresh(okhttp, okhttp_files), ["--quiet", str(tree)], one_core=one_core)
        scenario("okhttp, CI check (`-n --set-exit-if-changed`)", tools, opts.runs,
                 fresh(okhttp, okhttp_files), ["-n", "--set-exit-if-changed", str(tree)])
        scenario("Pre-commit: 10 changed files", tools, opts.runs, fresh(okhttp, precommit),
                 ["--quiet"] + [str(tree / f.relative_to(okhttp)) for f in precommit])
        scenario(f"Editor: one {editor_file.stat().st_size // 1000} KB file on stdin", tools, opts.runs,
                 lambda: None, ["-"], stdin_path=editor_file)

    print(f"\nbinary {Path(ours).stat().st_size / 1e6:.1f} MB; jar {Path(jar).stat().st_size / 1e6:.1f} MB plus a JRE")


if __name__ == "__main__":
    sys.exit(main())
