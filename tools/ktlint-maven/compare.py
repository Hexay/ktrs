"""compare.py <scenario-dir> [--known FILE]: diffs <dir>/{upstream,ktrs}: the console (Maven noise and the plugin coordinates
normalized) and every project file but the POMs; --known applies tools/parity/known_diffs.py entries.
Exit 1 when anything differs (last line DIFFERENT, else IDENTICAL)."""
import os, re, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "parity"))
from known_diffs import Known, unified

root = os.path.abspath(sys.argv[1])
KNOWN = Known(sys.argv[sys.argv.index("--known") + 1] if "--known" in sys.argv else None, os.path.basename(root))
NOISE = re.compile(r"(\[INFO\] (Total time|Finished at)|WARNING: |\[WARNING\] (Using platform encoding|File encoding))")
# Deviation 2 of research/31: `format` logs an engine exception without its JVM stack trace.
STACK_TRACE = re.compile(r"(\s+at |[\w.$]+(Exception|Error): )")
DURATION = re.compile(r"\[ *[\d.]+ (s|min)\]$")
PLUGIN = re.compile(r"(com\.github\.gantsign\.maven:ktlint-maven-plugin:3\.7\.1|io\.github\.hexay:ktrs-ktlint-maven-plugin:[\w.-]+)")
GOAL = re.compile(r"--- ktlint:[\w.-]+:(\w+)")
SKIP = {"pom.xml", ".mvn/jvm.config"}


def norm(text, side):
    proj = os.path.join(root, side, "project")
    for p in (proj, proj.replace("\\", "/"), proj.replace("\\", "\\\\")):
        text = text.replace(p, "<P>")
    text = PLUGIN.sub("<plugin>", text)
    text = GOAL.sub(r"--- ktlint:<version>:\1", text)
    return [DURATION.sub("[<t>]", line) for line in text.splitlines()
            if not NOISE.match(line) and not STACK_TRACE.match(line)]


def files(side):
    base = os.path.join(root, side, "project")
    result = {}
    for d, _, fs in os.walk(base):
        for f in fs:
            rel = os.path.relpath(os.path.join(d, f), base).replace("\\", "/")
            if rel in SKIP or rel.endswith("/pom.xml") or rel.endswith(".png") or rel.endswith(".gif"):
                continue
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
        return norm(f.read(), side)


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
