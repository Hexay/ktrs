"""rows.py <scenario-dir>: console error rows of both sides as multisets, ignoring order and the lint-mode suffix."""
import collections, re, sys

SUFFIX = " (cannot be auto-corrected)"


def rows(side):
    text = open(f"{sys.argv[1]}/{side}/console.txt", encoding="utf-8", errors="replace").read()
    out = []
    for line in text.splitlines():
        m = re.search(r"[\\/]project([\\/].*:\d+:\d+ .*)$", line)
        if m:
            out.append(m.group(1).replace("\\", "/"))
    return out


u, k = rows("upstream"), rows("ktrs")
suffixed = sum(1 for r in u if SUFFIX in r)
cu = collections.Counter(r.replace(SUFFIX, "") for r in u)
ck = collections.Counter(r.replace(SUFFIX, "") for r in k)
print(f"upstream rows {len(u)} (with suffix {suffixed}), ktrs rows {len(k)}, same multiset ignoring suffix: {cu == ck}")
for r in (cu - ck).elements():
    print("only upstream:", r)
for r in (ck - cu).elements():
    print("only ktrs:", r)
