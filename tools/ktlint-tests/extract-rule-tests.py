"""Vendors ktlint's rule unit tests (`<Rule>Test.kt`, KtLintAssertThat DSL) as data for ktrs_lint.

  py -3 tools/ktlint-tests/extract-rule-tests.py NoSemicolonsRuleTest SpacingAroundCommaRuleTest ...

Reads third_party/ktlint (tools/sync-ktlint.sh) and writes testdata/ktlint/<TestClass>.txt, one case per test:

  #### case <name>                 test name; flags follow as `#### flag <f>`
  #### code / #### formatted      raw-string bodies after trimIndent()
  #### violations                 `line:col<TAB>auto|manual<TAB>detail`, lint mode, rule under test only

Flags: `disabled` (@Disabled), `no-violations` (hasNoLintViolations), `additional-rules` (formatting depends on
another rule, so only violations are checked), `unsupported <why>` (skipped by the harness).
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TESTS = ROOT.parent.parent.parent / "third_party/ktlint" if (ROOT / ".git").is_file() else ROOT / "third_party/ktlint"
TEST_DIR = "ktlint-ruleset-standard/src/test/kotlin/io/github/ktlint/core/ruleset/standard/rules"

FUN = re.compile(r"^\s*fun `(.+)`\(\)", re.M)
RAW = r'"""(.*?)"""(\.trimIndent\(\))?'
VIOLATION = re.compile(r'(hasLintViolationWithoutAutoCorrect|hasLintViolation|LintViolation)\(\s*(\d+),\s*(\d+),\s*"((?:[^"\\]|\\.)*)"')


def trim_indent(text):
    lines = text.split("\n")
    indents = [len(l) - len(l.lstrip()) for l in lines if l.strip()]
    common = min(indents) if indents else 0
    out = [l[common:] for l in lines]
    if out and not out[0].strip():
        out = out[1:]
    if out and not out[-1].strip():
        out = out[:-1]
    return "\n".join(out)


def unescape(s):
    return re.sub(r"\\(.)", lambda m: {"n": "\n", "t": "\t"}.get(m.group(1), m.group(1)), s)


def raw_string(body, name):
    m = re.search(r"val " + name + r"\s*=\s*" + RAW, body, re.S)
    if not m:
        return None, None
    text = m.group(1)
    if not m.group(2):
        return None, "raw string without trimIndent"
    if re.search(r"\$[{A-Za-z_]", text.replace("${'$'}", "")):
        return None, "string template"
    return trim_indent(text.replace("${'$'}", "$")), None


def cases(source):
    starts = [m for m in FUN.finditer(source)]
    for i, m in enumerate(starts):
        body = source[m.end() : starts[i + 1].start() if i + 1 < len(starts) else len(source)]
        head = source[starts[i - 1].end() if i else 0 : m.start()]
        flags = []
        if "@Disabled" in head[head.rfind("}") + 1 :]:
            flags.append("disabled")
        code, why = raw_string(body, "code")
        formatted, why_formatted = raw_string(body, "formattedCode")
        why = why or why_formatted
        if code is None:
            why = why or "no `val code` raw string"
        if "addAdditionalRuleProvider" in body:
            flags.append("additional-rules")
        if "withEditorConfigOverride" in body or "asKotlinScript" in body:
            why = why or "editorconfig override / script"
        if "hasNoLintViolations()" in body:
            flags.append("no-violations")
        if why:
            flags.append("unsupported " + why)
        violations = [
            f"{v[1]}:{v[2]}\t{'manual' if v[0] == 'hasLintViolationWithoutAutoCorrect' else 'auto'}\t{unescape(v[3])}"
            for v in VIOLATION.findall(body)
        ]
        yield m.group(1), flags, code, formatted if "isFormattedAs(formattedCode)" in body else None, violations


def main(names):
    out_dir = ROOT / "testdata/ktlint"
    out_dir.mkdir(parents=True, exist_ok=True)
    for name in names:
        source = (TESTS / TEST_DIR / f"{name}.kt").read_text(encoding="utf-8")
        parts = []
        for case_name, flags, code, formatted, violations in cases(source):
            parts.append(f"#### case {case_name}\n")
            parts += [f"#### flag {f}\n" for f in flags]
            if code is not None:
                parts.append(f"#### code\n{code}\n")
            if formatted is not None:
                parts.append(f"#### formatted\n{formatted}\n")
            parts.append("#### violations\n" + "".join(v + "\n" for v in violations))
        (out_dir / f"{name}.txt").write_text("".join(parts), encoding="utf-8", newline="\n")
        print(f"{name}: {sum(1 for p in parts if p.startswith('#### case'))} cases")


if __name__ == "__main__":
    main(sys.argv[1:])
