"""compare.py <scenario-dir> [--slashes] [--known FILE]: diffs <dir>/{upstream,ktrs}: the console (each run's task blocks and
failures in a stable order, Gradle noise dropped) and every project file but build caches and intermediates.
--slashes ignores path separators and ANSI colors; --known applies tools/parity/known_diffs.py entries.
Exit 1 when anything differs (last line DIFFERENT, else IDENTICAL)."""
import os, re, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "parity"))
from known_diffs import Known, unified

root = os.path.abspath(sys.argv[1])
KNOWN = Known(sys.argv[sys.argv.index("--known") + 1] if "--known" in sys.argv else None, os.path.basename(root))
SLASHES = "--slashes" in sys.argv
SKIP_DIRS = {".gradle", ".kotlin", "kotlin", "classes", "tmp", "intermediates", "libs", "kotlinToolingMetadata", "problems"}
NOISE = re.compile(
    r"(Starting a Gradle Daemon|BUILD (SUCCESSFUL|FAILED) in|\d+ actionable task|Configuration cache|Reusing configuration"
    r"|Calculating task graph|Consider enabling|Daemon will be stopped|Deprecated Gradle|You can use '--warning-mode"
    r"|For more on this|See https://docs.gradle.org|w: |Fetching distribution|Downloading https://services\.gradle\.org/|\.+10%|\[Incubating\] Problems report|> Configure project :java)"
)


def norm(text, side):
    proj = os.path.join(root, side, "project")
    for p in (proj, proj.replace("\\", "/"), proj.replace("\\", "\\\\")):
        text = text.replace(p, "<P>")
    # The plugin id is the one thing a drop-in can't share.
    text = text.replace("plugin 'io.github.hexay.ktrs.ktlint'", "plugin 'org.jlleitschuh.gradle.ktlint'")
    text = re.sub(r"([\\/]+)" + side + r"([\\/]+project)", r"\1<side>\2", text)
    if SLASHES:
        text = re.sub(r"\x1b\[\d+m", "", text).replace("\\\\", "/").replace("\\", "/")
    return [line for line in text.splitlines() if ":java" not in line and not NOISE.match(line)]


def task_order_free(lines):
    out, block, blocks = [], [], []

    def flush():
        nonlocal block
        if block:
            blocks.append([x for x in block if x.strip()])
        block = []

    def drain():
        out.extend(x for b in sorted(blocks) for x in b)
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
        dirs[:] = [x for x in dirs if x not in SKIP_DIRS]
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
