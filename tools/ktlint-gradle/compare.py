"""compare.py <scenario-dir> --plugin-ids KTRS UPSTREAM [--slashes] [--known FILE]: diffs <dir>/{upstream,ktrs}: the
console (each run's task blocks and failures in a stable order, Gradle noise dropped) and every project file but build
caches and intermediates. --plugin-ids: the drop-in's plugin id, read as the upstream one; --slashes ignores path
separators and ANSI colors; --known applies tools/parity/known_diffs.py entries.
Exit 1 when anything differs (last line DIFFERENT, else IDENTICAL)."""
import os, re, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "parity"))
from known_diffs import Known, unified

root = os.path.abspath(sys.argv[1])
KNOWN = Known(sys.argv[sys.argv.index("--known") + 1] if "--known" in sys.argv else None, os.path.basename(root))
SLASHES = "--slashes" in sys.argv
KTRS_ID, UPSTREAM_ID = sys.argv[sys.argv.index("--plugin-ids") + 1 :][:2]
SKIP_DIRS = {".gradle", ".kotlin"}
# Under a `build` directory only: `src/main/kotlin` is a source directory.
SKIP_BUILD_DIRS = {"kotlin", "classes", "tmp", "intermediates", "libs", "kotlinToolingMetadata", "resources", "generated"}
NOISE = re.compile(
    r"(Starting a Gradle Daemon|BUILD (SUCCESSFUL|FAILED) in|\d+ actionable task|Configuration cache|Reusing configuration"
    r"|Calculating task graph|Consider enabling|Daemon will be stopped|Deprecated Gradle|You can use '--warning-mode"
    r"|For more on this|See https://docs.gradle.org|w: |Fetching distribution|Downloading https://services\.gradle\.org/|\.+10%|\[Incubating\] Problems report|> Configure project )"
)
# A build's own compile tasks (buildSrc, a rule set project) hit the build cache on whichever side runs second.
COMPILE_FROM_CACHE = re.compile(r"^(> Task \S*:compile\w+) FROM-CACHE$")


def norm(text, side):
    proj = os.path.join(root, side, "project")
    for p in (proj, proj.replace("\\", "/"), proj.replace("\\", "\\\\")):
        text = text.replace(p, "<P>")
    # The plugin id is the one thing a drop-in can't share.
    text = text.replace(f"plugin '{KTRS_ID}'", f"plugin '{UPSTREAM_ID}'")
    text = re.sub(r"([\\/]+)" + side + r"([\\/]+project)", r"\1<side>\2", text)
    if SLASHES:
        text = re.sub(r"\x1b\[\d+m", "", text).replace("\\\\", "/").replace("\\", "/")
    lines = [line for line in text.splitlines() if ":java" not in line and not NOISE.match(line)]
    return [COMPILE_FROM_CACHE.sub(r"\1", line) for line in lines]


def task_order_free(lines):
    out, block, blocks = [], [], []

    def flush():
        nonlocal block
        if block:
            blocks.append([x for x in block if x.strip()])
        block = []

    def drain():
        # A task whose output outlives Gradle's grouping window is printed in two blocks, the outcome on the second.
        by_task, merged = {}, []
        for b in blocks:
            task = re.match(r"> Task (\S+)", b[0]) if b else None
            first = by_task.get(task.group(1)) if task else None
            if first is None:
                merged.append(b)
                if task:
                    by_task[task.group(1)] = b
            else:
                first[0] = max(first[0], b[0], key=len)
                first.extend(b[1:])
        out.extend(x for b in sorted(merged) for x in b)
        blocks.clear()

    for line in lines:
        if line.startswith("== exit"):
            flush(); drain(); out.append(line)
        elif line.startswith("> Task ") or re.match(r"\d+: Task failed", line) or line.startswith("FAILURE:"):
            flush(); block = [re.sub(r"^\d+: ", "N: ", line)]
        else:
            block.append(line)
    flush(); drain()
    return out


def files(side):
    base = os.path.join(root, side, "project")
    result = {}
    for d, dirs, fs in os.walk(base):
        parent = os.path.basename(d)
        skip = SKIP_DIRS | (SKIP_BUILD_DIRS if parent == "build" else {"problems", "configuration-cache"} if parent == "reports" else set())
        dirs[:] = [x for x in dirs if x not in skip]
        for f in fs:
            rel = os.path.relpath(os.path.join(d, f), base).replace("\\", "/")
            if rel not in ("settings.gradle", "build.gradle", "gradle.properties"):
                with open(os.path.join(d, f), encoding="utf-8", errors="replace") as fh:
                    result[rel] = norm(fh.read(), side)
    return result


def diff(name, a, b):
    d = unified(name, *KNOWN.apply(name, a, b))
    if d:
        print("\n".join(d) + "\n")
    return bool(d)


def console(side):
    with open(os.path.join(root, side, "console.txt"), encoding="utf-8", errors="replace") as f:
        return task_order_free(norm(f.read(), side))


different = diff("console.txt", console("upstream"), console("ktrs"))
fu, fk = files("upstream"), files("ktrs")
for name in sorted(set(fu) | set(fk)):
    if name not in fk or name not in fu:
        print(("only upstream: " if name in fu else "only ktrs: ") + name)
        different = True
    else:
        different |= diff(name, fu[name], fk[name])
for line in KNOWN.stale():
    print("stale known diff (remove it): " + line)
    different = True
print("DIFFERENT" if different else "IDENTICAL")
sys.exit(1 if different else 0)
