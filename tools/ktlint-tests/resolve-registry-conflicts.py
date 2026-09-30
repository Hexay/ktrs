"""Resolve merge conflicts between parallel rule-porting branches in rules/mod.rs and golden-passing.txt.

Both files are sorted lists that each branch only appends to, so every conflict hunk resolves to the sorted
union of its two sides: `mod` lines, then a blank line, then `pub use` lines; anything else sorts by text.

usage: py -3 tools/ktlint-tests/resolve-registry-conflicts.py <file>...
"""
import re
import sys

MOD = re.compile(r"^(pub )?mod (\w+);$")
USE = re.compile(r"^pub use (\w+)")


def key(line):
    s = line.strip()
    if m := MOD.match(s):
        return (0, m.group(2))
    if m := USE.match(s):
        return (1, m.group(1))
    return (2, s)


def resolve(lines):
    out, i = [], 0
    while i < len(lines):
        if not lines[i].startswith("<<<<<<<"):
            out.append(lines[i])
            i += 1
            continue
        side = []
        i += 1
        while not lines[i].startswith(">>>>>>>"):
            if not lines[i].startswith("=======") and lines[i].strip():
                side.append(lines[i])
            i += 1
        i += 1
        merged = sorted(dict.fromkeys(side), key=key)
        for prev, cur in zip([None] + merged, merged):
            if prev is not None and key(prev)[0] == 0 and key(cur)[0] == 1:
                out.append("\n")
            out.append(cur)
    return out


for path in sys.argv[1:]:
    with open(path, encoding="utf-8", newline="") as f:
        lines = f.readlines()
    resolved = resolve(lines)
    with open(path, "w", encoding="utf-8", newline="") as f:
        f.writelines(resolved)
    left = sum(l.startswith(("<<<<<<<", ">>>>>>>")) for l in resolved)
    print(f"{path}: {'resolved' if not left else f'{left} markers left'}")
