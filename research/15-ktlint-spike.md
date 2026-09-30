# 15 — ktlint spike: mutable arena + 3 rules (Rust half of research/11 "Prototype first") (2026-09-30)

Target ktlint **2.0.0-ALPHA-4** (`third_party/ktlint`). IntelliJ semantics from the platform classes in
`tools/psi-dump/lib/kotlin-compiler-embeddable-2.4.20.jar`, decompiled with IntelliJ's fernflower.

## Verdict

| Criterion (research/11) | Result |
|---|---|
| (a) dumps + text byte-equal to the JVM oracle where the rules fire | **Pending**: the oracle's three-rule corpus run (tools/ktlint-oracle, testbox `~/work/ktlint-probe/out/three`) had not been produced when this was written. Runner ready: `tools/ktlint-tests/oracle-diff.sh`. |
| (b) seeding ≤ 25% of parse | **Go**: 11.3% (corpus, 6123 files, 30.8 MB, 10.5 M nodes: parse 0.975 s, `Ast::from_parse` 0.110 s; best of 3, thread cycles). |
| (c) no allocation per navigation step | **Go**: 0 allocations for 11 navigation calls on each of 70,403 nodes; the 3-rule traversal allocates 10 times in total (children stack growth). `crates/ktrs-lint/tests/alloc.rs`. |

Upstream unit tests of the three rules: 60 of 63 run, all pass (3 are `@Disabled` upstream). Formatting checks of
the 4 `MultiLineIfElseRuleTest` cases that add `IndentationRule` are skipped (violations still checked).
Go on (b) and (c); (a) decides whether option (a) is confirmed.

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

## Rust-side corpus run (6123 files, 4 threads, 57 s wall including lint)

- 112 files with violations: multiline-if-else 261 rows, no-semi 6, comma-spacing 5. 270 rows in pass 1, 2 in pass 2.
- 111 files changed. 2 files had a second mutating pass (nested `if` in `else`).
- 2 parse failures: Exposed `sourceFiles/*.kt` templates.
- 3 files diverge from a fresh parse:
  - `ktfmt/.../ListFormatter.kt`: KDoc `(,)`. `KDOC_TEXT(',')` gets a following `PsiWhiteSpace`; a reparse makes one
    `KDOC_TEXT(', ')`.
  - Two ktor `.gradle.kts`: `else @Suppress(..) { … }` is wrapped in a `BLOCK`; a reparse reads a lambda.

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

## Missing (TODO pointers in code)

- `SuppressionLocator`, `KtlintSuppressionRule`, `EditorConfigLoader`/rule-execution properties (defaults of
  `ktlint_official` hard-wired), `stopTraversalOfAST`, and `KtLintRuleException` line/col wrapping (rule panics
  propagate; the probe reports `crash`).
- `CodeEditUtil.removeChild`/`makePlaceHolderBetweenTokens` (for the one `psi.delete()` site); `addLeaf`.
- About half of the `ASTNodeExtension` functions; the rest of `IndentConfig`; the typed-PSI accessors of
  research/11 §5.
- `psi_to_string` picks the PSI class from the type. Only an explicit `LeafPsiElement(WHITE_SPACE, …)` would print
  differently, and no rule builds one.

## Reproduce

```sh
cargo test -p ktrs-ast -p ktrs-lint --release
cargo run -p ktrs-ast --release --example seed_bench -- corpus 3
cargo ktlint-probe corpus target/ktlint-probe/corpus --dumps --threads 4
tools/ktlint-tests/oracle-diff.sh [jvm-out]     # (a), once the oracle's three-rule run exists
```
