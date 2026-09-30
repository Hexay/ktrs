# 11 — ktlint rule engine: semantics, tree mutation, ktrs design (2026-09-30)

Source-reading only; nothing was executed except `javap` on the pinned compiler jar
(`tools/psi-dump/lib/kotlin-compiler-embeddable-2.4.20.jar`) to check IntelliJ internals. Builds on
`04-parity-scope.md` §2 (LOC, mutation-API counts) and `06-tree-library.md` (the flat `Tree`).

Paths: **M** = `corpus/ktlint` (master `3820242`, 2.0 line, `io/github/ktlint/core/...`). **V1** = the 1.8.0 tag
(scratch clone, `com/pinterest/ktlint/...`). `eng/` = `ktlint-rule-engine/.../rule/engine/`, `core/` =
`ktlint-rule-engine-core/.../core/api/`, `rules/` = `ktlint-ruleset-standard/.../standard/rules/`.

## Verdict

- **Recommended: option (a).** Build a mutable arena CST with IntelliJ `TreeElement` semantics. Seed it
  from the flat `Tree` in one O(n) copy, and **never reparse** between rules or passes, because ktlint never
  does. Snippet parsing (`createASTNodeFromText`) runs the ktrs parser, then grafts the subtree in.
- **Why reparsing can't give parity:**
  - Rules keep `ASTNode` references for the whole traversal. In 2.0 every rule is live at once, so no
    reparse point exists that keeps identity.
  - ktlint's edits produce trees a fresh parse never would (§3). Those trees persist through all three
    format passes.
- **Target 2.0 (master), not 1.8.0.** Master embeds Kotlin **2.4.20**, which is exactly our pin. 1.8.0
  embeds 2.2.21 (V1 `gradle/libs.versions.toml:6`), so we would need a second parser pin.
  - Rule order changes output: per-rule topological order in 1.8.0 vs per-node alphabetical in 2.0.
  - A port can match only one of them.
- **Key parity risk:** reproducing IntelliJ's *mutated* tree shape and node identity exactly, including:
  - leaves carrying composite types,
  - commas grafted into the wrong parent,
  - comment binding that goes stale after whitespace edits,
  - `rawReplaceWithText` swapping in a new node.

  Plus the quirk that line/col are computed from the **original** text's line table.

## 1. Engine semantics, 1.8.0 vs master

| Aspect | 1.8.0 | master (2.0) |
|---|---|---|
| Rule API | `Rule`; `RuleAutocorrectApproveHandler` optional. The legacy `autoCorrect: Boolean` path is V1 `eng/internal/RuleExecutionContext.kt:128-162`. All standard rules implement the handler (V1 `StandardRule.kt:30`). | `RuleV2` only. `emit` returns `AutocorrectDecision`; rules gate edits with `ifAutocorrectAllowed` (`core/AutocorrectDecision.kt`). |
| Instantiation | New instances on **every pass** (`VisitorProvider.rules` maps `createNewRuleInstance`, V1/M `eng/internal/VisitorProvider.kt:40-49`), so rule state lives for one pass. | same |
| Enable filter | `RuleExecutionRuleFilter`: `ktlint_<set>_<rule>` beats `Experimental`/`OfficialCodeStyle` (needs `ktlint_code_style=ktlint_official`)/`OnlyWhenEnabledInEditorconfig`/rule-set default (`eng/internal/rulefilter/RuleExecutionRuleFilter.kt:62-122`). `ktlint-suppression` cannot be disabled (:70). | Identical logic. |
| Load filter | `RunAfterRuleFilter` drops a rule whose `RunAfterRule(.., ONLY_WHEN_RUN_AFTER_RULE_IS_LOADED_AND_ENABLED)` target is off. Example: `string-template-indent` needs `indent` (V1 `rules/StringTemplateIndentRule.kt:57`). | Removed: visitor modifiers are ignored (M `ktlint-com-pinterest-backward-compatibility/.../Rule.kt:206`). |
| Order | Sort key: suppression rule first → not `RunAsLateAsPossible` → standard set → id (V1 `eng/internal/RuleProviderSorter.kt:104-127`). Then a topological sort over `RunAfterRule` (:56-93). 16 standard rules declare modifiers (indent, max-line-length, trailing-comma-*, function/class-signature, …). | Standard set first, then alphabetical id (M `RuleProviderSorter.kt:54-63`). The suppression rule gets its own full traversal first (M `RuleExecutionContext.kt:45`). |
| Traversal | **Per rule**: `beforeFirstNode` → DFS(before, children, after) → `afterLastNode`, then the next rule on the same mutated tree (V1 `CodeFormatter.kt:106-110`, `RuleExecutionContext.kt:41-73,164-190`). | **Per node**: `beforeFirstNode` for all rules; then at each node `before` for every rule, then children, then `after` for every rule; then `afterLastNode` for all (M `RuleExecutionContext.kt:54-72,126-228`). |
| Children | `node.getChildren(null)` is a **snapshot** taken after `before` (V1:148). Children inserted later are not visited this pass; removed ones are still visited, detached. | Same snapshot, taken after *all* rules' `before` (M:175-184). **New:** a node with `parent == null` that isn't `FILE` (i.e. replaced) aborts that node: remaining rules, children and `after` (M:133-145, 186-198). |
| stopTraversal | Checked on node entry; `after` still runs for the stopping node and its ancestors (V1:96,147). | The rule list is re-filtered per child (M:180); same observable contract (`core/Rule.kt:145-163`). |
| Suppression check | Once per node, reused for before and after (V1:143). | Rechecked for before and after using `startOffset` at that moment (M:147,200). |
| Suppression source | `@Suppress`/`@SuppressWarnings` on the nearest `KtAnnotated` owner (`"ktlint"` = all; `"ktlint:<id>"`; IDE names like `PropertyName` map to rule ids) plus `ij_formatter_off/on` comment tags. An unclosed tag runs to the containing `}` or the next sibling (`eng/internal/SuppressionLocator.kt:59-221`). Hints are rebuilt when `rootNode.text.hashCode()` changes (:41-48). | Same. 1.8.0 also prefixes a bare `ktlint:foo` with `standard:`. |
| `ktlint-disable` | Not honored by the locator. `KtlintSuppressionRule` emits and autocorrects directives into `@Suppress` annotations it builds from parsed snippets (`eng/internal/KtlintSuppression.kt:322,353,374`). | same |
| Emit → error | `positionInTextLocator(offset)` gives `LintError(line,col,ruleId,detail,canBeAutoCorrected)`. The handler is always asked. Autocorrect = `ALLOW && canBeAutoCorrected`; a rule that got approval is assumed to have fixed it (`CodeFormatter.kt:128-153`). | same |
| Handlers | `None` (lint), `All` (deprecated `format(code, cb)`, V1 `KtLintRuleEngine.kt:113-116`), `LintErrorAutocorrectHandler(defaultAutocorrect, cb)`. | `None`, `LintErrorAutocorrectHandler(cb)` (M `AutocorrectHandler.kt`). |
| Format loop | See the next table. | same |
| lintAfterFormat | Only logs a warning; its errors are discarded. Stops at the first autocorrectable error (V1 `CodeFormatter.kt:114-130`). | Runs all rules (M:110-120). Still no output effect, except that it can throw. |
| Rule sets | 102 rule files | 107: adds blank-line-before-{file-annotation,imports,package}, call-expression-wrapping, kdoc-delimiter, lambda-return, no-blank-line-at-start-of-file, context-parameter-list-wrapping; drops condition-wrapping, discouraged-comment-location. |
| Embedded Kotlin | 2.2.21 | 2.4.20 (M `gradle/libs.versions.toml:10`). |
| Ext-fn names | `isWhiteSpace20`, `nextSibling20`, … (transitional names, same semantics) | `isWhiteSpace`, … |

**Format loop** (M/V1 `eng/internal/CodeFormatter.kt:46-97`, identical in both):

| Step | Detail |
|---|---|
| Parse | Parse **once**: CRLF/CR→LF, strip the BOM, file name `File.kt`/`.kts` or the real path (`RuleExecutionContext.kt:235-258`). |
| Passes | `do { pass; stop if no error was autocorrected; stop if root text is unchanged; count++ } while (count < max)` (:57-79). `max` = `MAX_FORMAT_RUNS_PER_FILE = 3` (`api/KtLintRuleEngine.kt:171`). It is 1 with `rerunAfterAutocorrect=false` and 1 for `lint` (:61-71), where the first `break` fires. |
| Final lint | After 3 mutating passes, one extra lint pass that only logs (:80-89). |
| Output | Mutated: `rootNode.text` with `\n` → EOL (:99). Unmutated: the **original bytes** (:90-95). BOM re-prefixed (:34). |
| EOL | CRLF if `end_of_line=crlf`, or if `end_of_line≠lf` and the input contains `\r`. The helper named `doesNotContain` actually tests *contains* (:157-169). Port as-is. |
| Error order | The `(LintError, Boolean)` callback path sorts by (line, col) over a `Set` (`LintError` is `@Poko`, so duplicates collapse) (:30-32). Only `lint` uses it. `format` reports through the autocorrect callback, in emit order, with duplicates across passes. CLI: `ktlint-cli/.../KtlintCommandLine.kt:480-507`. |
| **Line/col quirk** | The locator is built from the **original** normalized text (`RuleExecutionContext.kt:236`) and reused for every pass. Emit offsets are `startOffset` in the **mutated** tree. So any error emitted after an earlier edit (same pass or later passes) is mapped through the stale line table. Offsets are UTF-16. |
| Parse errors | The first `PsiErrorElement` (DFS over PSI children) throws `KtLintParseException(line, col of textOffset)` before any rule runs (:251-257). The CLI reports ruleId `""`, "Not a valid Kotlin file (…)" (`KtlintCommandLine.kt:670-679`). |
| Reparse | **Never** between rules or passes. Only CLI `--force-lint-after-format` re-lints the output text as a check (`KtlintCommandLine.kt:518`). |
| Rule crash | Wrapped as `KtLintRuleException`. Line/col come from the node in lint mode and are 0:0 in format mode (`RuleExecutionContext.kt:152-172`). |

## 2. How rules see and mutate the tree

**Reads are ASTNode, not PSI.** `ASTNode` is imported in 106 files. `.psi` has 125 hits, but almost all are
`...psi.impl/psiUtil/tree` package imports. There are **~15 typed-PSI call sites** (§5), no `KtPsiFactory`,
and no `PsiTreeUtil`. Navigation goes through ktlint's `core/ASTNodeExtension.kt` (714 LOC), the compiler's
`psiUtil` (`leaves`, `siblings`, `children`, `parents`) and IntelliJ `ASTNode` members.

Mutation primitives (master `rules/`: sites/files). The IntelliJ effects were checked with `javap`:

| Primitive | Sites | IntelliJ-level effect |
|---|---|---|
| `upsertWhitespaceBeforeMe` / `AfterMe` (`core/ASTNodeExtension.kt:317-415`) | 128/39, 75/30 | If the adjacent leaf is `WHITE_SPACE`, `replaceTextWith`. Never inserts as a composite's first/last child (climbs to the parent). Otherwise a new `PsiWhiteSpaceImpl`, via `rawInsertBefore/AfterMe` (leaf) or `parent.addChild` (composite). |
| `replaceTextWith` (:363-367) | 81/34 | No-op if the text is equal. Otherwise `LeafElement.rawReplaceWithText` → `ASTFactory.leaf(type, text)` creates a **new node** and `rawReplaceWithList` swaps it in. The old leaf is **detached** (parent = null) and any rule-held reference goes stale. This is why M:133-145 exists. |
| `remove()` (:632) | 106/40 | `parent.removeChild`. Adjacent whitespace is **not** merged. |
| `addChild(node, anchor)` | 57/21 | `CompositeElement.addChild`: `removeChildrenInner` first (so it **moves** an attached node), then `ChangeUtil.prepareAndRunChangeAction`. ktlint's `FormatPomModel` just runs it (`core/KtlintKotlinCompiler.kt:134-139`): no reformat, no merging. |
| `replaceChild` / `removeRange` / `addChildren` | 6 / 2 / 1 | Same change-action path; `addChildren` moves a sibling range. |
| `rawInsertBeforeMe` / `AfterMe`, `rawRemove` | 6, 3, 3 | Raw relinking, no events. |
| `clone()` | 2 | Deep copy, detached (`ModifierOrderRule.kt:64`, `SpacingAroundCommaRule.kt:51`). |
| `psi.delete()` | 1 | `ArgumentListWrappingRule.kt:187`. Goes through `deleteChildInternal` → **`CodeEditUtil.removeChild` → `makePlaceHolderBetweenTokens`**, the only path that can insert or merge whitespace. Port that subset. |
| `KtImportDirective.rawDelete()` | 1 | `NoUnusedImportsRule.kt:133-136` (`delete()` throws on 2.4: no `KtPsiMutationService`). |
| New leaves | 44 | `PsiWhiteSpaceImpl` 22, `LeafPsiElement(type, text)` 20 (any type, even composite ones, §3), `PsiCommentImpl` 2. |
| New composites | 3 | `KtBlockExpression(null)` filled by `addChild` (`IfElseBracingRule.kt:155`, `MultiLineIfElseRule.kt:151`, `MultilineLoopRule.kt:100`). |
| **Snippet parse** | 4 rules + engine | `createASTNodeFromText(text)` parses as `File.kts` and returns SCRIPT>BLOCK>(SCRIPT_INITIALIZER) (`core/KtlintKotlinCompiler.kt:49-56`). Users: `FunctionExpressionBodyRule.kt:164` (`: Unit` TYPE_REFERENCE), `StringTemplateRule.kt:128` (SHORT_STRING_TEMPLATE_ENTRY), `WhenEntryBracing.kt:161` (a whole WHEN_ENTRY rebuilt from text), `KdocDelimiterRule.kt:431` (KDOC_END, 2.0 only). Engine: `KtlintSuppression.kt:322,353,374` (annotations). 1.8.0 has the first three. |

**Do later rules see the mutated tree? Yes, always.**
- There is a single `rootNode` per format call, and every pass reuses it (`CodeFormatter.kt:51,99`).
- 1.8.0: rule k+1 traverses the tree that rules 1..k left behind (V1 `CodeFormatter.kt:106-110`).
- 2.0: at node N, rule j's `before` sees rules 1..j-1's edits at N and every edit made at earlier nodes.
  N's children are the snapshot taken after all `before` hooks ran (M `RuleExecutionContext.kt:132-184`).

## 3. Mutate-then-continue vs reparse: concrete divergences

All derived from the code; none of these was executed (see "Prototype first").

| Case | Mutated tree (ktlint) | Fresh parse of the same text | Why it matters |
|---|---|---|---|
| `SpacingAroundCommaRule.kt:47-53`: `foo(a // c\n, b)` | The comma clone is added at the end of `prevCodeLeaf.parent`, i.e. **inside REFERENCE_EXPRESSION `a`** (`nextSibling` of IDENTIFIER is null) | COMMA is a child of VALUE_ARGUMENT_LIST | Comma/argument rules (trailing-comma-on-call-site, argument-list-wrapping) look for COMMA among the list's children, in this pass and passes 2-3. |
| `WrappingRule.kt:544`: newline before closing `"""` | A **leaf** typed LITERAL_STRING_TEMPLATE_ENTRY with text `\n` | LITERAL_STRING_TEMPLATE_ENTRY > REGULAR_STRING_PART (`\n` is its own token, `Kotlin.flex:145`) | `IndentationRule.kt:1510` branches on `firstChildNode == null`, so rules already depend on this shape. |
| `IndentationRule.kt:1511`, `StringTemplateIndentRule.kt:301,334,355` | REGULAR_STRING_PART inserted as a **sibling** of entries, directly under STRING_TEMPLATE | Always wrapped in an entry | Entry-walking code sees different children. |
| Blank line removed or added before a commented declaration (no-consecutive-blank-lines, blank-line-before-declaration, spacing-between-declarations-with-comments) | The comment stays where it was | `PrecedingCommentsBinder` binds comments into the declaration unless a blank line intervenes (`ktrs_parser/src/builder/binders.rs:68-95`) | `firstChildNode is comment` vs `prevSibling is comment` decides several rules' output. |
| Newline inserted before a same-line trailing comment (`TrailingCommentsBinder`) | The comment stays inside the previous declaration | It rebinds to the next declaration or goes outside | Same as above. |
| `remove()` of a node between two whitespaces | Two adjacent WHITE_SPACE leaves | One merged WHITE_SPACE | `prevLeaf.isWhiteSpaceWithNewline` sees only half. `upsert*` rewrites one leaf. |
| `remove()` of a composite's first child | Whitespace can become a composite's first child | The parser never puts it there (edge binders) | `upsert*` climbing logic takes a different branch. |
| `replaceTextWith` | A new node; the old one is detached but **still in the child snapshot** | n/a | 1.8.0 visits the detached leaf (startOffset 0 → `1:1` errors); 2.0 aborts that node. Identity must be modeled. |
| Rule state across nodes (NoUnusedImports `imports` map, IndentationRule context stack, ImportOrdering) | References stay valid | All identities invalid | A reparse is impossible mid-traversal in 2.0. |

## 4. Design options

| | (a) Mutable arena CST | (b) Text edits + reparse | (c) Flat base + mutable overlay, reparse on structure |
|---|---|---|---|
| Parity | Exact by construction: each IntelliJ primitive ported 1:1 over linked nodes, identity kept, no reparse (same as ktlint) | **Fails**: §3 shapes and identity; 2.0 has no safe reparse point | Fails wherever it reparses (snippet rules, block building); the overlay must still model detach/identity |
| 1:1 port rule | Rules port line-by-line against an `AstNode` with IntelliJ member names | Every rule's autocorrect body must be rewritten as text edits: a rewrite, not a port | Two code paths per primitive |
| Perf | Seed copy ≈ flat build cost (`06` flat build 0.059 s/corpus ≪ parse). Lint mode and clean files never mutate. `startOffset` needs an offset cache (IntelliJ caches offset-in-parent) | Reparse ≈ 1 parse per edit batch; cheap per file but O(edits × parse) worst case | Every navigation step checks the overlay; the complexity isn't bought back |
| Effort | Arena + primitives ≈ 1.5k LOC; `ASTNodeExtension` + `IndentConfig` ≈ 1k; engine ≈ 1.5k | Low engine effort, huge rule effort | Highest |

**Stale-offset gotcha, applies to (a):** every `emit` offset must convert to UTF-16 in the *mutated* text, then
go through the *original* line table. Keep a per-leaf UTF-16 length, or convert on emit (emits are rare).

## Recommendation

1. **Target master (2.0 line), behind a pinned tag** once 2.0.0 is final. The shared Kotlin 2.4.20 pin
   decides it: 1.8.0 needs a 2.2.21 parser and the `RunAfterRule` sorter and filter.
   - Record 1.8.0 deltas as a table in this doc, not as code.
2. **New crate `ktrs_ktlint_ast`** (or a `ktrs_syntax::mut_tree` module): an arena of
   `{kind, parent, first/last child, prev/next, leaf text span into an append-only buffer, cached len}`.
   - `NodeId` never gets reused, so detached nodes keep working.
   - Build it from `Tree` in one preorder pass.
   - Port each IntelliJ primitive in §2 as one fn with the Java name: `add_child`, `remove_child`,
     `replace_child`, `remove_range`, `add_children`, `raw_insert_before_me`/`after_me`, `raw_remove`,
     `raw_replace_with_text` (allocates a new id), `clone`, plus `CodeEditUtil.makePlaceHolderBetweenTokens`
     for the single `psi.delete()` site.
   - Its `psi_dump` must print the IntelliJ format so mutated trees can be diffed against the JVM.
3. **Port ktlint's `ASTNodeExtension.kt` / `IndentConfig.kt` 1:1** over that arena. Also port the ~12 PSI
   accessors in §5 over the same arena.
   - Don't make `ktrs_psi` generic: it would cost ktfmt speed for about 15 call sites.
4. **Snippet parse** = `ktrs_parser::parse_file(text, Script)`, then graft the requested subtree into the arena.
   This mirrors `KtlintKotlinCompiler.createASTNodeFromText`.
5. **Engine** = 1:1 port of master `CodeFormatter` and `RuleExecutionContext`, including:
   - the detached-node bail-out,
   - the children snapshot,
   - the original-text line table,
   - the `doesNotContain` EOL quirk,
   - unmutated → original bytes.

   Also `SuppressionLocator`: rebuild hints only when the root text changes. A mutation counter plus a text
   compare is equivalent to the hash check.

## 5. PSI/AST accessors the standard rules need (master, rough frequency: occurrences/files)

| Layer | Needed (≈ count) | ktrs today |
|---|---|---|
| IntelliJ `ASTNode` | `elementType` 926/103, `startOffset` 296/103, `findChildByType` 269/49, `text` 233/70, `firstChildNode` 99/36, `textContains` 46, `textLength` 44, `lastChildNode` 19, `treeParent/Next/Prev` 19, `getChildren(TokenSet?)`, `clone`, `psi` | `ktrs_psi::AstNode` over the immutable tree only. **Rebuild over the arena.** |
| ktlint `ASTNodeExtension` | `isWhiteSpace*` 507/197/90, `prevLeaf` 355, `parent` 270, `nextLeaf` 222, `nextSibling` 173, `isPartOf` 163, `prevSibling` 123, `children` 125, `isPartOfComment` 91, `lastChildLeafOrSelf` 90, `prevCodeLeaf` 73, `nextCodeSibling` 67, `indent` 64, `prevCodeSibling` 54, `firstChildLeafOrSelf` 53, `isCode` 39, `nextCodeLeaf` 34, `indentWithoutNewlinePrefix` 31, `lineLength`/`leavesOnLine` 24/21, `*NewLineIn*Range` 20, `findParentByType` 20, `hasModifier` 20, `isDeclaration` 10, `column` 5 | None. Port all ~60 fns (714 LOC). |
| `IndentConfig` (`core/IndentConfig.kt`, 173 LOC) | 180 uses; `childIndentOf`/`parentIndentOf`/`siblingIndentOf` 76 | None |
| Compiler `psiUtil` on ASTNode | `leaves(forward)` 13 files, `siblings(forward, withItself)` 9, `children` 5, `parents` 3 | PsiElement `siblings`/`prev_leaf`/`next_leaf` exist; need ASTNode forms over the arena |
| Typed PSI | `KtImportDirective.importPath/.importedName/.pathStr/aliasName` (ImportOrdering, NoUnusedImports, NoWildcardImports, FunctionNaming); `KtPackageDirective.qualifiedName`; `KtSuperTypeList.entries` (Wrapping:294); `KtDotQualifiedExpression.selectorExpression`; `KtWhenEntry.isElse` + **`KtWhenExpression.leftParenthesis`** (TrailingCommaOnDeclarationSite:168-170); **`KtFunction.hasDeclaredReturnType`/`name`/`typeReference`** (FunctionNaming:85-99); `KtPsiUtil.unquoteIdentifier`; `KtFile.virtualFilePath` (host path); `isKtAnnotated`/`dummyPsiElement` = psi class per element type (`core/ASTNodeExtension.kt:654-686`); `KtTokenSets.DECLARATION_TYPES`; `KtSingleValueToken.value` | Most exist in `ktrs_psi` (`lib.rs` API list), but over the immutable tree. **Bold** = missing. `psi_class_name` covers the type → class mapping; the `KtAnnotated` interface table is new. |

## Prototype first

The smallest experiment that de-risks (a) has two halves. Run the JVM part on the testbox, in the background.

1. **JVM divergence census (≈150 LOC Kotlin, uses the real engine unchanged).**
   - Add a probe `RuleV2` with id `zzz:probe`. Non-standard rule sets sort last, so it runs after every
     standard rule at each node.
   - The probe grabs the FILE node in `beforeVisitChildNodes`. In `afterLastNode` (end of each pass) it writes
     `DebugUtil.psiToString(mutated root)` and `psiToString(parse(root.text))`.
   - Run ktlint master `format` over `corpus/`. Output: per file and pass, whether the dumps differ.
     Then run once per rule (`{rule, probe}`) on files where that rule emits, to attribute divergences.
   - This quantifies §3 and produces **mutated-tree oracle dumps** for step 2.
2. **Rust arena spike (≈600 LOC).**
   - Build the arena from `Tree`, the §2 primitives, and an IntelliJ-format dump.
   - Port the master traversal plus 3 rules that cover the hard primitives:
     - `no-semi` (remove),
     - `spacing-around-comma` (clone + graft into a foreign parent),
     - `multiline-if-else` (`KtBlockExpression` build + `replaceChild`).
   - **Go/no-go:**
     - The mutated dump and the output text are byte-equal to step 1's oracle on every corpus file where these
       rules fire.
     - Arena seeding costs ≤ 25% of parse time (the parser `bench` cycle clock).
     - A 3-rule traversal allocates nothing per navigation step.

   If the dumps match only after a reparse, or identity can't be kept cheaply, revisit (c) for lint-only mode.
