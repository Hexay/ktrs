# 15 — ktlint spike: mutable arena + 3 rules (Rust half of research/11 "Prototype first") (2026-09-30)

Target ktlint **2.0.0-ALPHA-4** (`third_party/ktlint`). IntelliJ semantics from the platform classes in
`tools/psi-dump/lib/kotlin-compiler-embeddable-2.4.20.jar`, decompiled with IntelliJ's fernflower.

## Verdict

| Criterion (research/11) | Result |
|---|---|
| (a) dumps + text byte-equal to the JVM oracle where the rules fire | **Go**: 113 files fire on either side (see below). All 272 violation rows are identical, including the stale-table line:col of pass 2. So are all 111 formatted texts, the pass flags and both parse failures. The mutated dumps are byte-identical in 111 of 113 files; the other 2 differ only in untouched KDoc, from the 2.4.10-vs-2.4.20 KDoc lexer. |
| (b) seeding ≤ 25% of parse | **Go**: 11.3% (corpus, 6123 files, 30.8 MB, 10.5 M nodes: parse 0.975 s, `Ast::from_parse` 0.110 s; best of 3, thread cycles). |
| (c) no allocation per navigation step | **Go**: 0 allocations for 11 navigation calls on each of 70,403 nodes; the 3-rule traversal allocates 10 times in total (children stack growth). `crates/ktrs-lint/tests/alloc.rs`. |

Upstream unit tests of the three rules: 60 of 63 run, all pass (3 are `@Disabled` upstream). Formatting checks of
the 4 `MultiLineIfElseRuleTest` cases that add `IndentationRule` are skipped (violations still checked).

**Overall: go for option (a)** (mutable arena, IntelliJ primitives ported 1:1, no reparse).

## Oracle diff (a), details

- **Oracle run:** tools/ktlint-oracle `KtlintProbe` (ktlint 2.0.0-ALPHA-4 fat jar) on corpus/ (6123 files, the
  revisions in corpus/REVISIONS). Flags: `--rules standard:no-semi,standard:comma-spacing,standard:multiline-if-else
  --dumps --no-lint`.
  - Run on the testbox (`~/work/ktlint-probe/out/three-for-rust.tgz`) and once locally; the results are identical.
- **Rust run:** `cargo ktlint-probe corpus target/ktlint-probe/three <same flags>`.
- **Compare:** `cargo ktlint-probe compare target/ktlint-oracle/three-for-rust target/ktlint-probe/three corpus`
  reports 211 files with a row on either side:
  - 111 identical in every artifact.
  - 98 differ **only** in `internal:ktlint-suppression` rows, all `manual`. That internal rule always runs and is
    not ported (finding 6). These are `@Suppress("ktlint:…")` in ktlint's own sources, reported as "unknown or not
    loaded" because only 3 rules are loaded. None of the 98 has a row from the three rules on either side.
  - 2 differ in `mut.p1` only, in KDoc the rules never touch: `AbstractCoroutine.kt` and `Semaphore.kt`.
    Around `` `true` ``, the jar's 2.4.10 lexer yields `KDOC_TEXT('`true`') KDOC_CODE_SPAN_TEXT(' if …')`, where
    2.4.20 (our pin) yields `KDOC_TEXT('`') KDOC_CODE_SPAN_TEXT('true') …`. This is a parser-pin difference,
    not an engine one (finding 3).
- **Oracle smoke set** (testbox `~/work/ktlint-probe/smoke`): identical in every artifact. It includes research/11
  §3's comma grafted into `REFERENCE_EXPRESSION`, a `;` removal and two block wraps.

## What was built

- `crates/ktrs-ast`: arena of `{kind, flags, parent/first/last/prev/next, text start, len, offset-in-parent
  cache}`. `NodeId`s are never reused. Seeded from `ktrs_syntax::Tree` in one preorder pass. IntelliJ member names:
  - `TreeElement`: `clone`, `start_offset`, `raw_insert_before_me`, `raw_insert_after_me`, `raw_remove`,
    `raw_replace_with_list`, `raw_remove_up_to`.
  - `CompositeElement`: `find_child_by_type`, `get_children`, `add_child`, `remove_child`, `remove_range`,
    `replace_child`, `add_children`, `raw_add_children`.
  - `LeafElement`: `raw_replace_with_text`. `TreeUtil`: `next_leaf`, `prev_leaf`, `find_last_leaf`,
    `get_file_element`. psiUtil: `leaves`.
  - `psi_to_string` (seeded dump == `psi_dump` on all parser fixtures, `tests/seed.rs`) and
    `create_ast_node_from_text` (parse as script, graft).
  - `tests/primitives.rs` has one test per primitive; each test cites the Java method it checks.
- `crates/ktrs-lint`: ports of `Rule.kt`/`AutocorrectDecision.kt` (`RuleV2`), the 2.0 per-node traversal,
  `CodeFormatter` (format loop, error set, sort, BOM, EOL quirk), `PositionInTextLocator` (UTF-16,
  original-text table), the used subset of `ASTNodeExtension.kt` (queries + `ast_node_edit.rs`) and `IndentConfig`.
  Rules: `standard:no-semi`, `standard:comma-spacing`, `standard:multiline-if-else`.
- `examples/ktlint_probe` (`cargo ktlint-probe`) writes the oracle's layout (below). Its `compare` mode is the
  go/no-go diff.
- `tools/ktlint-tests/extract-rule-tests.py` vendors `<Rule>Test.kt` into `testdata/ktlint/`.

## Output format (shared with tools/ktlint-oracle/src/KtlintProbe.kt; that code is the spec)

Per out-dir:

- `fmt/<rel>`: formatted text, only when it differs from the input.
- `mut/<rel>.p<N>.txt`: `psiToString(file, true, false)` after pass N, only when the tree changed from the
  previous pass (`--dumps`). Line 1 carries each side's own file path, so `compare` skips it.
- `diff/<rel>.p<N>.diff`: mutated vs fresh parse, when they diverge.
- `passes.tsv`: `file pass changed diverged`. Pass N counts every traversal, including lint-after-format.
- `format.tsv`: `file pass line col rule auto|manual detail`, in format-callback (emit) order.
- `lint.tsv`: `file line col rule auto|manual detail`, lint mode (sorted, distinct).
- `failed.tsv`: `file parse|rule|crash message`.

The task brief's fallback format (`<file>.pass<N>.txt` / `.formatted.kt` / `.lint.txt`) was dropped because the
oracle already existed.

## What fires on the corpus (identical on both sides)

- 112 files with violations: multiline-if-else 261 rows, no-semi 6, comma-spacing 5. 270 rows in pass 1, 2 in pass 2.
- 111 files changed. 2 files had a second mutating pass (nested `if` in `else`).
- 2 parse failures: Exposed `sourceFiles/*.kt` templates.
- 3 files diverge from a fresh parse. These are real ktlint shapes, reproduced:
  - `ktfmt/.../ListFormatter.kt`: KDoc `(,)`. `KDOC_TEXT(',')` gets a following `PsiWhiteSpace`; a reparse makes one
    `KDOC_TEXT(', ')`.
  - Two ktor `.gradle.kts`: `else @Suppress(..) { … }` is wrapped in a `BLOCK`; a reparse reads a lambda.
- Time (local, 2 threads, no lint), summed over files, probe included:
  - Rust: 3.8 s of `format`, 9.9 s wall.
  - JVM: 53.0 s of `format`, 33.4 s wall.
  - One earlier Rust run took 87 s wall with the same format sum, under machine load. Compare runs by the sums,
    not by wall time.

## Findings that correct or sharpen research/11

1. **Removed nodes are not orphaned.**
   - `CompositeElement.removeChild`/`removeRange`/`addChild`(move)/`replaceChild` go through
     `removeChildrenInner` → `repairRemovedElement`. That parks the removed range in a fresh `DummyHolder`, a
     `FileElement` of type `DUMMY_HOLDER`.
   - So `treeParent != null` for them. The 2.0 bail-out (`parent == null && type != FILE`) fires only for
     `rawReplaceWithText` (`replaceTextWith`, `upsertWhitespace*`) and `raw*` removals.
   - A removed-but-snapshotted child is still visited. Its `startOffset` is relative to its holder.
   - research/11 §1/§3 "removed ones are still visited, detached" should read "…visited, inside a dummy holder".
2. **`addChild` on a composite outside any file throws.** `ChangeUtil.prepareAndRunChangeAction` dereferences
   `getFileElement(changedElement)`. `KtBlockExpression(null)` works in MultiLineIfElse only because
   `replaceChild` attaches it first. Ported as a panic.
3. **ALPHA-4 ids and pins differ from master.**
   - The comma rule's id is `comma-spacing`, not `spacing-around-comma`.
   - Per `tools/sync-ktlint.sh`, the ALPHA-4 fat jar embeds Kotlin **2.4.10**, not 2.4.20. Trees can differ
     wherever the parser changed between the two, and the jar's platform classes should be re-checked against
     the 2.4.20 decompile used here.
4. **ktlint's own leaf walk is not TreeUtil's.**
   - ktlint's `nextLeaf`/`prevLeaf` can return an empty composite. The property forms skip only zero-length
     nodes; the `{ predicate }` forms skip nothing.
   - `TreeUtil.nextLeaf` and psiUtil `leaves`, which MultiLineIfElse uses, return only `LeafElement`s.
   - Both are ported. The TreeUtil ones are free functions (`ktrs_ast::tree_util`) so the two can't be confused.
5. **The line table maps the end of a file without a trailing newline to `1:1`.** `SegmentTree.indexOf` returns
   -1 there. The port keeps this.
6. **The internal `ktlint-suppression` rule always runs** (`InternalRuleProvidersFilter`), in its own first
   traversal, even with `--rules`. It is not ported, so files with `ktlint-disable` directives may differ.
   `compare` flags files mentioning `ktlint` as suspects.
7. **Cost.**
   - Offsets use IntelliJ's lazy offset-in-parent cache, with the valid-prefix invariant.
   - Lengths are kept eagerly (propagated in `set_tree_parent`), so `text_length` is O(1).
   - `utf16_offset` is O(1) on all-ASCII trees. Otherwise it walks the leaves, only on emit.
8. **Take IntelliJ/PSI semantics from the ALPHA-4 fat jar (Kotlin 2.4.10), not 2.4.20 sources** (testbox
   `~/work/ktlint-probe/lib`; `tools/ktlint-oracle/{Edit,Extension}Oracle.java` run on it). `KtElement.delete()` is
   `deleteSemicolon()` + `ASTDelegatePsiElement.delete` there (2.4.20 routes it to `KtPsiMutationService`), and
   `BLOCK`/`LAMBDA_EXPRESSION` are lazy types, so `isKtAnnotated` throws on them.
9. **`psi.delete()` rewrites whitespace.** `CodeEditUtil.needToForceReformat` is true unless the removed node starts
   its parent, so the neighbours of a deleted node are merged into, or replaced by, new whitespace leaves
   (`crates/ktrs-ast/tests/data/edit_cases.jvm.txt`). `KtModifierList` deletes itself once emptied.

## Missing (TODO pointers in code)

- `SuppressionLocator`, `KtlintSuppressionRule`, `EditorConfigLoader`/rule-execution properties (defaults of
  `ktlint_official` hard-wired), `stopTraversalOfAST`, and `KtLintRuleException` line/col wrapping (rule panics
  propagate; the probe reports `crash`).
- Done (1B): `CodeEditUtil.removeChild` path + `psi.delete()` (`ktrs_ast::code_edit_util`, `psi::delete`), `addLeaf`.
- Done (1B): all of `ASTNodeExtension.kt` (`ast_node_extension/`), `IndentConfig`, `ElementType`, `TokenSets`.
- Done (1B): typed PSI of research/11 §5 (`ktrs_ast::psi`). Coverage of the rules' API: research/16.
- `psi_to_string` picks the PSI class from the type. Only an explicit `LeafPsiElement(WHITE_SPACE, …)` would print
  differently, and no rule builds one.

## Reproduce

```sh
cargo test -p ktrs-ast -p ktrs-lint --release
cargo run -p ktrs-ast --release --example seed_bench -- corpus 3
cargo ktlint-probe corpus target/ktlint-probe/corpus --dumps --threads 4
tools/ktlint-tests/oracle-diff.sh target/ktlint-oracle/three-for-rust   # (a); unpack the testbox tarball there first
```
