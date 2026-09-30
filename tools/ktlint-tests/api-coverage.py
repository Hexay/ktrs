"""Coverage of the upstream API the ktlint standard ruleset uses, by the Rust port.

    py -3 tools/ktlint-tests/api-coverage.py            # rewrite research/16-ktlint-api-coverage.{md,tsv}
    py -3 tools/ktlint-tests/api-coverage.py --check    # print unmapped symbols and TODOs, write nothing

Census (third_party/ktlint/ktlint-ruleset-standard/src/main): every imported core-API / compiler / IntelliJ
symbol, plus the ASTNode/PSI members in MEMBERS (called without an import). Each symbol is looked up in
api-map.tsv (upstream, rust, probe, note); status is `done` when the probe regex (default: a definition of
the Rust name's last segment) matches the Rust sources, else `TODO`. Use counts are regex hits: approximate.
"""
import collections
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]


def third_party():
    """third_party/ is gitignored: in a git worktree, fall back to the main checkout's."""
    local = ROOT / "third_party"
    if local.is_dir():
        return local
    common = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--git-common-dir"], capture_output=True, text=True)
    return (ROOT / common.stdout.strip()).resolve().parent / "third_party"


UPSTREAM = third_party() / "ktlint/ktlint-ruleset-standard/src/main/kotlin"
RUST = [ROOT / "crates" / c / "src" for c in ("ktrs-ast", "ktrs-lint", "ktrs-psi", "ktrs-parser", "ktrs-syntax")]
MAP = pathlib.Path(__file__).with_name("api-map.tsv")
OUT_MD = ROOT / "research/16-ktlint-api-coverage.md"
OUT_TSV = ROOT / "research/16-ktlint-api-coverage.tsv"

CORE = "io.github.ktlint.core.rule.engine.core."
SKIP_IMPORTS = ("io.github.ktlint.core.ruleset.standard", "kotlin.", "java.", "io.github.oshai")
# ASTNode / PsiElement / Kt* members the rules call without importing them.
MEMBERS = """elementType text textLength startOffset textContains textMatches firstChildNode lastChildNode
treeParent treeNext treePrev findChildByType getChildren addChild addChildren removeRange replaceChild clone
rawInsertBeforeMe rawInsertAfterMe rawRemove psi node delete firstChild containingFile virtualFile
importPath importedName aliasName pathStr qualifiedName isAllUnder hasAlias selectorExpression isElse
leftParenthesis hasDeclaredReturnType typeReference bodyExpression renderName createASTNodeFromText
unquoteIdentifier virtualFilePath""".split()


def category(symbol):
    if symbol.startswith(CORE):
        return "core"
    if symbol.startswith("org.jetbrains.kotlin.com.intellij"):
        return "intellij"
    if symbol.startswith("org.jetbrains.kotlin"):
        return "compiler"
    if symbol.startswith("."):
        return "member"
    return "other"


def census():
    uses = collections.Counter()
    files = collections.defaultdict(set)
    element_types = set()
    for f in sorted(UPSTREAM.rglob("*.kt")):
        text = f.read_text(encoding="utf-8")
        body = "\n".join(line for line in text.splitlines() if not line.startswith("import "))
        for m in re.finditer(r"^import ([\w.]+)", text, re.M):
            symbol = m.group(1)
            if symbol.startswith(SKIP_IMPORTS):
                continue
            if ".ElementType." in symbol:
                element_types.add(symbol.rsplit(".", 1)[-1])
                symbol = CORE + "api.ElementType.*"
            short = symbol.rsplit(".", 1)[-1]
            hits = len(re.findall(r"\b" + re.escape(short) + r"\b", body)) if not symbol.endswith("*") else 1
            uses[symbol] += max(hits, 1)
            files[symbol].add(f.stem)
        for member in MEMBERS:
            hits = len(re.findall(r"(?:\?\.|\.|::)" + member + r"\b", body))
            if hits:
                uses["." + member] += hits
                files["." + member].add(f.stem)
    return uses, files, element_types


def missing_element_types(names):
    """`ElementType.X` names that are neither a `SyntaxKind` variant nor an `element_type.rs` const."""
    kinds = (ROOT / "crates/ktrs-syntax/src/generated/kinds.rs").read_text(encoding="utf-8")
    aliases = (ROOT / "crates/ktrs-lint/src/element_type.rs").read_text(encoding="utf-8")
    known = set(re.findall(r"^\s+([A-Z][A-Z0-9_]*),", kinds, re.M)) | set(re.findall(r"pub const (\w+):", aliases))
    return sorted(names - known)


def load_map():
    mapping = {}
    for line in MAP.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        cols = (line.split("\t") + ["", "", ""])[:4]
        mapping[cols[0]] = cols[1:]
    return mapping


def rust_sources():
    return "\n".join(p.read_text(encoding="utf-8") for d in RUST for p in d.rglob("*.rs"))


def default_probe(rust):
    name = re.split(r"::|\.", rust.split("(")[0].split(" ")[0])[-1]
    n = re.escape(name)
    # an item definition, or a PSI class declared in the psi_classes! macro (`KtFoo(ast, n) =>`)
    return r"(?m)\b(?:fn|const|struct|enum|trait|type|mod)\s+" + n + r"\b|^\s*" + n + r"\(ast, n\) =>"


def short_name(symbol):
    s = symbol[len(CORE):] if symbol.startswith(CORE) else symbol
    for prefix in ("org.jetbrains.kotlin.com.intellij.", "org.jetbrains.kotlin."):
        if s.startswith(prefix):
            s = s[len(prefix):]
    return s.replace("api.", "", 1) if s.startswith("api.") else s


def main():
    uses, files, element_types = census()
    mapping = load_map()
    sources = rust_sources()
    missing = missing_element_types(element_types)
    rows = []
    for symbol in sorted(uses, key=lambda s: (category(s), -uses[s], s)):
        rust, probe, note = mapping.get(short_name(symbol), ["", "", "unmapped"])
        if rust in ("", "TODO"):
            status = "TODO"
        elif rust.startswith("n/a"):
            status = "n/a"
        else:
            status = "done" if re.search(probe or default_probe(rust), sources) else "TODO"
        if symbol.endswith("ElementType.*") and missing:
            status, note = "TODO", "missing: " + " ".join(missing)
        rows.append((category(symbol), short_name(symbol), uses[symbol], len(files[symbol]), rust or "-", status, note))
    counted = [r for r in rows if r[5] != "n/a"]
    done = [r for r in counted if r[5] == "done"]
    weighted = sum(r[2] for r in done) / max(1, sum(r[2] for r in counted))
    summary = f"{len(done)}/{len(counted)} symbols done ({100 * len(done) / max(1, len(counted)):.0f}%), {100 * weighted:.0f}% of use sites"
    tree = [r for r in counted if not r[6].startswith("engine")]
    summary += f"; AST/PSI/extension API (not engine/editorconfig): {sum(r[5] == 'done' for r in tree)}/{len(tree)}"
    if "--check" in sys.argv:
        print(summary)
        for r in rows:
            if r[5] == "TODO":
                print("TODO", r[1], r[4], r[6], sep="\t")
        return
    OUT_TSV.write_text(
        "category\tupstream\tuses\tfiles\trust\tstatus\tnote\n" + "".join("\t".join(map(str, r)) + "\n" for r in rows),
        encoding="utf-8",
    )
    OUT_MD.write_text(render_md(rows, summary), encoding="utf-8")
    print(summary)


def render_md(rows, summary):
    by_cat = collections.defaultdict(list)
    for r in rows:
        by_cat[r[0]].append(r)
    titles = {"core": "ktlint core API", "intellij": "IntelliJ platform", "compiler": "Kotlin compiler PSI / lexer",
              "member": "ASTNode / PSI members (no import)", "other": "Other libraries"}
    out = [
        "# 16 — ktlint API coverage: what the standard rules call, and its Rust name",
        "",
        "Generated by `py -3 tools/ktlint-tests/api-coverage.py` from ktlint 2.0.0-ALPHA-4's",
        "`ktlint-ruleset-standard` and `tools/ktlint-tests/api-map.tsv`; do not edit by hand. Full table with",
        "use counts and notes: `research/16-ktlint-api-coverage.tsv`. `uses` = regex hits (approximate).",
        "Receivers are the `Ast`: `node.nextLeaf` -> `ast.next_leaf(node)`; conventions in `crates/ktrs-ast/src/lib.rs`,",
        "`crates/ktrs-ast/src/psi/mod.rs` and `crates/ktrs-lint/src/lib.rs`.",
        "",
        f"**{summary}.**",
        "",
    ]
    for cat in ("core", "intellij", "compiler", "member", "other"):
        if cat not in by_cat:
            continue
        out += [f"## {titles[cat]}", "", "| upstream | Rust | | upstream | Rust | |", "|---|---|---|---|---|---|"]
        cells = [f"`{r[1]}` | {rust_cell(r[4])} | {mark(r[5])}" for r in by_cat[cat]]
        for i in range(0, len(cells), 2):
            pair = cells[i:i + 2] + ([" | | "] if i + 1 >= len(cells) else [])
            out.append("| " + " | ".join(pair) + " |")
        out.append("")
    out.append("Legend: ✓ done, ✗ TODO, – not ported by design (see the TSV note).")
    return "\n".join(out) + "\n"


def rust_cell(rust):
    return "-" if rust in ("", "-", "TODO") else f"`{rust}`"


def mark(status):
    return {"done": "✓", "TODO": "✗", "n/a": "–"}[status]


if __name__ == "__main__":
    main()
