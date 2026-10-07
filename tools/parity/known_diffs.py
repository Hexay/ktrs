"""Reviewed, accepted differences of the build-tool parity scripts (tools/parity/known-diffs/<tool>.tsv).

Entries: `<scenario>\t<file glob>\t<kind>[\t<regex>]`, `#` comment lines above them give the reason. The glob matches
the compared file's project-relative path (or `console.txt`). Kinds:
  order          the same lines in any order
  upstream-only  upstream lines matching <regex> are dropped before comparing
  ktrs-only      ktrs lines matching <regex> are dropped before comparing
An entry its scenario no longer needs is reported as stale and fails the run, so the list stays exact.

As a script, for parity scripts without a compare.py of their own:
  known_diffs.py KNOWN SCENARIO NAME UPSTREAM_FILE KTRS_FILE
prints the unified diff left after the entries apply (and stale entries); exit 1 when it printed anything.
"""
import difflib, fnmatch, re, sys

KINDS = ("order", "upstream-only", "ktrs-only")


class Known:
    def __init__(self, path, scenario):
        self.entries = []
        if not path:
            return
        with open(path, encoding="utf-8") as f:
            for line in f:
                line = line.rstrip("\r\n")
                if not line or line.startswith("#"):
                    continue
                parts = line.split("\t")
                if len(parts) < 3 or parts[2] not in KINDS or (parts[2] != "order") != (len(parts) == 4):
                    raise SystemExit(f"{path}: bad entry {line!r}")
                if parts[0] == scenario:
                    regex = re.compile(parts[3]) if len(parts) == 4 else None
                    self.entries.append({"line": line, "glob": parts[1], "kind": parts[2], "re": regex, "used": False})

    def apply(self, name, upstream, ktrs):
        """The two line lists with the matching entries applied; marks them used when they made the lists equal."""
        matching = [e for e in self.entries if fnmatch.fnmatchcase(name, e["glob"])]
        a, b = upstream, ktrs
        for e in matching:
            if e["kind"] == "order":
                a, b = sorted(a), sorted(b)
            elif e["kind"] == "upstream-only":
                a = [x for x in a if not e["re"].search(x)]
            else:
                b = [x for x in b if not e["re"].search(x)]
        if upstream != ktrs and a == b:
            for e in matching:
                e["used"] = True
        return a, b

    def stale(self):
        return [e["line"] for e in self.entries if not e["used"]]


def unified(name, a, b):
    return list(difflib.unified_diff(a, b, "upstream/" + name, "ktrs/" + name, lineterm="", n=1))


def main(known_path, scenario, name, upstream_path, ktrs_path):
    known = Known(known_path, scenario)
    read = lambda p: open(p, encoding="utf-8", errors="replace").read().splitlines()
    out = unified(name, *known.apply(name, read(upstream_path), read(ktrs_path)))
    out += [f"stale known diff (remove from {known_path}): {line}" for line in known.stale()]
    if out:
        print("\n".join(out))
    return 1 if out else 0


if __name__ == "__main__":
    if len(sys.argv) != 6:
        raise SystemExit(__doc__)
    sys.exit(main(*sys.argv[1:]))
