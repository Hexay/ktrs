"""rows.py <scenario-dir>: the console error rows of both sides as multisets (paths project-relative): ktlint-gradle's
`<file>:<line>:<col> <detail>` and kotlinter's `<file>:<line>:<col>: <Lint error|Format fixed|...> > [<rule>] <detail>`."""
import collections, re, sys


def rows(side):
    text = open(f"{sys.argv[1]}/{side}/console.txt", encoding="utf-8", errors="replace").read()
    out = []
    for line in text.splitlines():
        m = re.search(r"[\\/]project([\\/].*:\d+:\d+:? .*)$", line)
        if m:
            out.append(m.group(1).replace("\\", "/"))
    return out


u, k = rows("upstream"), rows("ktrs")
same_set = collections.Counter(u) == collections.Counter(k)
# Order is compare.py's business: tasks interleave differently, rows within a task's block are compared there.
print(f"rows: upstream {len(u)}, ktrs {len(k)}; same rows: {same_set}")
for r in (collections.Counter(u) - collections.Counter(k)).elements():
    print("only upstream:", r)
for r in (collections.Counter(k) - collections.Counter(u)).elements():
    print("only ktrs:", r)
