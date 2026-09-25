# 03 — Kotlin parser in Rust: feasibility

Research date 2026-09-25. Sources: JetBrains/kotlin @ `c92aefba` (2026-09-25), Kotlin/kotlin-spec (2026-09-02), tree-sitter-kotlin 1.1.0 (tree-sitter-grammars on PyPI), plus measurements taken on this machine.

## Recommendation

**Port JetBrains' PSI parser (`KotlinParsing.java` + `KotlinExpressionParsing.java` + `Kotlin.flex`, plus the `PsiBuilder` semantics they depend on) 1:1 into a hand-written Rust recursive-descent parser. Have it emit events into a rowan/cstree lossless green tree.** Verify it by comparing PSI dumps against kotlinc, first on the 736 JetBrains fixture pairs and then on large real-world corpora.

- Don't build on tree-sitter-kotlin. 3.2% of real files have parse errors (up to 15% in repos that use context parameters). Even the files that parse cleanly match PSI only about 61% of the time.
- Don't build on the ANTLR spec grammar. It is out of date: it has no context parameters, no `$$` multi-dollar strings and no `when` guards. Its tree shape is also unrelated to PSI.
- The parser is small. The core is **5,155 Java LOC**. A faithful port is **about 12–18k Rust LOC and 4–7 person-months** to reach parity-tested quality (see §3).
- **Both ktfmt and ktlint reject any file that contains a `PsiErrorElement`.** ktfmt does this in `Parser.kt:75`; ktlint throws `KtLintParseException`. So a byte-exact PSI shape is required for **valid** code, but for invalid code only the yes/no answer "is there an error?" must match. This removes most of the error-recovery long tail from the critical path.

---

## 1. Grammar hard parts

Line numbers refer to `compiler/psi/parser/src/org/jetbrains/kotlin/...` at the commit above.

| Area | Example | How the real parser handles it |
|---|---|---|
| Newline-sensitive statement end | `val x = a`<br>`- b` makes two statements, but `a`<br>`?: b` and `a`<br>`.b()` make one | `interruptedWithNewLine()` (`KotlinExpressionParsing.java:1881`): a newline ends a binary or postfix chain unless the next token is in `ALLOW_NEWLINE_OPERATIONS` = `. ?. |. : as as? ?: && \|\|` (line 101). `is` and `!is` are deliberately left out because of `when` conditions. A `newlinesEnabled` **stack** in `SemanticWhitespaceAwarePsiBuilderImpl` switches newline sensitivity off inside `()`, `[]` and type-argument lists. The ANTLR grammar instead writes `NL*` explicitly almost everywhere. |
| Generic call vs comparison | `f<T>(x)`, `a < b > (c)`, `foo<bar, baz>=x` | Speculative: `tryParseTypeArgumentList` (`KotlinParsing.java:2508`) parses `<...>` as types, then **rolls back** when the next token is in `TYPE_ARGUMENT_LIST_STOPPERS` (about 60 tokens, `KotlinExpressionParsing.java:60`). The source has a `// TODO GTEQ` for `foo<bar, baz>=x`. Your parser needs cheap mark/rollback. There is no `>>` token: the lexer only emits `GT`, so `List<List<T>>` needs no token splitting. |
| Composite tokens | `a?.b`, `a ?: b`, `x!!`, `a\|.b` (rich errors) | **The lexer never emits `?.` `?:` `!!` `\|.`.** The builder joins adjacent raw `QUEST`+`DOT`, `QUEST`+`COLON`, `EXCL`+`EXCL` and `OR`+`DOT` and collapses each pair into one leaf (`SemanticWhitespaceAwarePsiBuilderImpl.java:22,139-151`). Joining can be switched off with a stack. Getting this wrong breaks leaf-by-leaf parity. |
| String templates | `"a${ "b${c}" }d"`, `"""raw ${x}"""`, `$$"""{"$ref": $$id}"""` | The flex lexer keeps a **state stack** of (state, `lBraceCount`, `requiredInterpolationPrefix`) (`Kotlin.flex:19-60,137-216`). Multi-dollar interpolation (Kotlin 2.2+) emits `INTERPOLATION_PREFIX`. Inside a string, the number of `$` has to equal the prefix for it to count as a template. A raw string ending in `""""` pushes back three characters. A newline inside a normal string gives `DANGLING_NEWLINE`. |
| Nested comments | `/* /* */ still comment */` | `commentDepth` counter. `/**` becomes `DOC_COMMENT`, which is then lazily re-parsed by a separate KDoc lexer and parser (`KDoc.flex` 405 LOC, `KDocParser` + `KDocLinkParser` 190 LOC). |
| Soft/modifier keywords | `val open = 1`, `in1.skip()`, `infix fun get(...)`, `x get y`, `get("/") {}` inside a lambda, `where?.let {}`, `suspend { }`, `value class`, `expect`/`actual` | Only the ~30 hard keywords are lexed as keywords (`Kotlin.flex:285-312`). Everything else is `IDENTIFIER` and is classified by the parser using lookahead: modifier lists are parsed only when followed by a declaration start. Per the spec's list there are 47 soft keywords. |
| Annotations with use-site targets | `@field:JvmStatic`, `@get:[A B]`, `@file:Suppress(...)`, `@Synchronized set(v) {}` | Handled in the parser, which makes whitespace-sensitive `@` decisions. The ANTLR grammar has to split `@` into `AT_NO_WS`, `AT_PRE_WS` and `AT_POST_WS` lexer tokens to express this. |
| Labels | `return@f true`, `a@ 1`, `this@A`, `super<A>@a`, `f@{ }` | `isAtLabelDefinitionOrMissingIdentifier` checks for IDENTIFIER followed immediately by `@`. Fixture `Labels.kt` covers about 25 variants, including error forms such as `return@@`. |
| Lambda vs block, trailing lambdas | `foo {}`, `foo() {}`, `foo<T> {}`, `if (x) { }` vs `run { }`, `f()`<br>`{ }` (newline) | `parseCallWithClosure` (line 451) accepts an annotated or labeled lambda only when there is no newline before it. The `preferBlock` flag decides between a block and a lambda in statement position. |
| `when` | `when (val x = f()) { is A if x.ok -> ... ; in 1..<9, !in s -> ... }` | Subject variable, multi-conditions, and the new `WHEN_ENTRY_GUARD` (`if` guard, line 955). |
| Context parameters | `context(_: Scope)` / `context(ctx: A, b: B) fun f()`, function type `context(A) () -> Unit` | Present in the compiler (fixture folder `psi/contextParameters/`). Missing from the ANTLR grammar and broken in tree-sitter (see §2). |
| Operators | `!!`, `?:`, `::` (`String?::plus` requires skipping `?` before `::`, `skipQuestionMarksBeforeDoubleColon`), `..<` (`RANGE_UNTIL`), `1..2` (the lexer pushes back `..` after an int, `Kotlin.flex:278`), `!in`/`!is` vs `!inside` (`Kotlin.flex:316`) | Covered by the lexer rules and parser code cited here. |
| Shebang | `#!/usr/bin/env kotlin` | Only at offset 0. Otherwise it lexes as `HASH` (`Kotlin.flex:268`). It is bound to the file node by a dedicated binder. |
| Identifiers | Unicode `[:letter:]` (`Kotlin.flex:96`), backticks `` `fun name` `` (no newline or backtick inside), a lone backtick → `BAD_CHARACTER` + `UNMATCHED_BACKTICK` state | Needs Unicode letter classes matching JFlex's Java `Character.isLetter`. Use `unicode-ident`-style tables generated from the same Unicode version. |
| Whitespace/comment ownership | Does the KDoc or line comment before `fun` belong inside FUNCTION or outside it? | `KotlinWhitespaceAndCommentsBinders.kt` defines `PrecedingCommentsBinder`, `PrecedingDocCommentsBinder`, `TrailingCommentsBinder`, `BindAll` and `BindFirstShebangWithWhitespaceOnly`, applied at 17 call sites. **ktlint rules depend on this placement**, for example comment-spacing and kdoc rules. PsiBuilder's default is to leave edge trivia outside the node. |
| Scripts | `.kts` top-level statements | `parseStatements(isScriptTopLevel=true)`, with a separate declaration mode. |

**Useful property:** the parser is **not gated on language version**. `LanguageFeature` does not appear in `parsing/`. It parses the union of all syntax and leaves rejection to later compiler phases, so you only need to track one grammar per Kotlin release.

**ANTLR spec grammar vs the real parser** (`KotlinParser.g4` 928 LOC, `KotlinLexer.g4` 529, `UnicodeClasses.g4` 1,648):
- It is missing context parameters, `$$` interpolation and `when` guards. A grep finds no `context`, `guard` or interpolation-prefix rules. The spec says it tracks the latest stable compiler "excluding experimental features", so it lags by design.
- The tree shape is unrelated to PSI: it uses rules like `genericCallLikeComparison` and `NL*` noise, and has no `DOT_QUALIFIED_EXPRESSION` or `CALL_EXPRESSION`-style nodes.
- It moves whitespace sensitivity into the lexer (`AT_PRE_WS`, `QUEST_WS`, `EXCL_WS`), whereas the compiler handles it in the parser.
- It is known to accept or reject a slightly different language, because the compiler defers some errors past parsing (kotlin-spec issue #103 and the grammar README).

**Verdict:** the grammar is a readable reference, but it is useless as a parity target.

## 2. tree-sitter-kotlin: measured

Setup: `py -3 -m pip install tree-sitter tree-sitter-kotlin` (py-tree-sitter 0.26.0, tree-sitter-kotlin 1.1.0 from tree-sitter-grammars). I parsed every `.kt`/`.kts` file in shallow clones taken on 2026-09-25 and counted a file as failing if its tree contains any ERROR or MISSING node. Scripts are `ts_measure.py` and `ts_innermost.py` in the session scratchpad.

| Repo | Files | With ERROR/MISSING |
|---|---:|---:|
| square/okhttp | 617 | 7 (1.1%) |
| Kotlin/kotlinx.coroutines | 1,082 | 19 (1.8%) |
| android/nowinandroid | 312 | 0 (0%) |
| pinterest/ktlint | 382 | 4 (1.0%) |
| facebook/ktfmt | 105 | **16 (15.2%)** |
| JetBrains/Exposed | 914 | 62 (6.8%) |
| ktorio/ktor | 2,495 | 82 (3.3%) |
| **Total** | **5,907** | **190 (3.2%)** |

Separately, on the kotlin repo's own `compiler/psi` tree, **39%** of files had errors. That set includes deliberately broken recovery fixtures, so it is not a fair real-world number.

Root causes, from the smallest ERROR/MISSING node in each of the 187 failing files:
- **Context parameters** `context(x: T)`: about 45 files. All 15 ktfmt failures, plus the ktor compiler-plugin and auth modules.
- **Multi-dollar strings** `$$"..."` / `$$"""..."""`: about 15.
- **Soft keywords used as identifiers**: about 45. Examples: `where?.let {}`; `} get Tasks.id` (infix `get`); `get("/") {}` at the start of a statement; `in1.close()`; `val open: Boolean` followed by `withLock {`; `suspend { }` lambdas; `@Synchronized set(value)`; `actual constructor`/`internal constructor` on the next line.
- **`when` guards** `is X if cond ->`: about 8.
- **Raw strings ending with extra quotes** `"""..."""""`: about 5.
- `String?::plus`: 2.
- Many files where the whole file collapsed into a single ERROR, so no finer location was available.

Upstream issue trackers:
- fwcd/tree-sitter-kotlin: 5 open issues, including #287 (`abort()` crashes the host process on about 512 unclosed string starts) and #274 (comment swallowed by the import list).
- tree-sitter-grammars fork: 9 open issues, all bugs. Among them: keyword-prefixed identifiers such as `in1` collapse to ERROR; several members on one line; `"$name"` vs `"${name}"` giving different trees; wrong precedence for `!A.b`; multiple `catch` blocks; `final`/`internal`/`suspend` treated as hard keywords.

Structural accuracy, which matters more than whether a file parses at all: the fwcd README reports a cross-validation against JetBrains PSI fixtures, **"74/121 (61.2%) structural match among clean parses"**. So even when tree-sitter reports no error, about 40% of fixture trees have the wrong shape.

Throughput: 0.8–1.7 MB/s through Python on a loaded machine. That number is not representative.

**Verdict: the claim "the tree-sitter grammar isn't accurate enough" is confirmed.** About 3% of real files fail outright, current language features are unsupported, the tree shape doesn't match PSI, and the node model is foreign to it (no `PsiWhiteSpace` leaves, different node kinds). It is fine for highlighting. It cannot be the foundation for byte-exact ktfmt or ktlint parity.

## 3. Rust architecture and effort

### Source size (measured, `wc -l`)

| File | LOC |
|---|---:|
| `parsing/KotlinParsing.java` | 2,842 |
| `parsing/KotlinExpressionParsing.java` | 1,884 |
| `parsing/AbstractKotlinParsing.java` | 429 |
| `SemanticWhitespaceAwarePsiBuilder*` (4 files) + `TokenStreamPattern`/`FirstBefore`/`LastBefore`… | about 560 |
| `parsing/KotlinWhitespaceAndCommentsBinders.kt` | about 150 |
| `lexer/Kotlin.flex` (JFlex spec; generated `_JetLexer.java` is 1,655) | 376 |
| `kdoc/lexer/KDoc.flex` + `KDocParser.java` + `KDocLinkParser.kt` | 405 + 92 + 98 |
| **Hand-written total** | **about 6.8k** (the full `parser/src` including generated lexers is 9,810) |

Token and node type tables live in `psi-api`: `KtTokens.java`, `KtNodeTypes.java`, `KDocTokens.java`, `KDocElementTypes.java`.

Hidden dependency: IntelliJ's `PsiBuilderImpl`, which lives in intellij-community rather than the Kotlin repo. You must reproduce these semantics: `mark`/`done`/`drop`/`rollbackTo`/`precede`/`doneBefore`/`collapse`/`error`/`errorBefore`, edge whitespace binders, `remapCurrentToken`, and lazy-parseable `BLOCK`/`LAMBDA_EXPRESSION`/KDoc (which only changes timing, not the final tree).

### Options

| Option | Fit |
|---|---|
| **A. 1:1 port of KotlinParsing onto an event-based builder + rowan/cstree (recommended)** | The rust-analyzer design, where events are `Start{kind, forward_parent}`, `Token`, `Finish` and `Error`, maps almost exactly onto PsiBuilder markers. `precede()` becomes `forward_parent`. `rollbackTo` becomes truncating the event vector and resetting the token index. Edge binders run during tree construction. rowan keeps trivia as child tokens, which is **the same model as PSI's `PsiWhiteSpace` leaves**, so a PSI dump can be printed directly from the tree. Consider `cstree` (a rowan fork with interning and `Send`/`Sync` trees) for parallel linting. |
| B. Biome's `biome_rowan` | Trivia is attached to tokens as leading or trailing, and nodes have fixed slots generated from a grammar. Both **mismatch PSI**: comment placement is a node-level decision in PSI, and Kotlin PSI nodes have variable shape. You would be fighting it. |
| C. Ruff-style hand-written parser to a typed AST | Fast (Ruff's hand-written parser beat its old LALRPOP one by 2.2–2.4×), but it produces an AST, not a lossless CST. ktlint rules walk `ASTNode`s including whitespace leaves, so you need a CST anyway. You could add a typed AST view on top of option A later. |
| D. Hand-written from the spec or ANTLR grammar | Similar LOC to option A, but every divergence from PSI becomes an open-ended investigation. |
| E. Parser generator (LALRPOP, pest, tree-sitter) | Rejected. Kotlin needs rollback lookahead, newline-mode stacks and composite-token joining. Ruff dropped LALRPOP after fighting it. |
| `ungrammar` | Useful only to generate typed AST accessors (`KtNamedFunction.name()` and so on) over the rowan tree. It does not generate parsers. Optional. |

Porting 1:1 is the most accurate route for a structural reason: ktfmt (`KotlinInputAstVisitor`, a PSI visitor) and ktlint (rules on `ASTNode`) consume exactly this tree. Parity bugs in the formatter or linter are otherwise very hard to tell apart from parser bugs. A port makes "same tree as kotlinc" a mechanically checkable invariant.

**License:** Apache-2.0. A port is a derivative work: keep the NOTICE and attribution headers. The result is compatible with an MIT/Apache-2.0 dual license.

### Effort estimate (one senior Rust dev familiar with rowan)

| Component | Rust LOC | Time |
|---|---:|---|
| Lexer: Kotlin + KDoc, state stack, Unicode tables, JFlex longest-match semantics | 1.5–2k | 3 wk |
| PsiBuilder equivalent: markers, rollback, precede, binders, newline and join stacks, token remap | 1–1.5k | 3 wk |
| Parser port (KotlinParsing + Expression + Abstract) | 5.5–7k | 8–10 wk |
| Kinds tables and PSI-dump printer in `DebugUtil.psiToString` format | 0.8k | 1 wk |
| Oracle harness: JVM dumper + diff runner + fixture importer | 0.5k Rust + 0.3k Kotlin | 1–2 wk |
| Parity burn-down on fixtures and corpora | — | 4–6 wk |
| Typed AST layer for lint rules (optional, can be generated) | 3–5k | 3–4 wk |
| **Total** | **about 12–18k** | **about 4–7 person-months** |

**Ongoing cost:** track the parser changes in each Kotlin release, aligned with the compiler version that ktlint/ktfmt pin (currently 2.4.20 and 2.4.10). Since this is a port, a release upgrade means diffing `compiler/psi/parser` between tags. That diff is usually a small number of hunks.

## 4. Testing oracle

- The fixtures have **moved**. `compiler/testData/psi` no longer exists; they now live in **`compiler/psi/psi-impl/testData/`**, driven by `compiler/psi/psi-impl/tests-gen/.../PsiParsingTestGenerated.java` (797 test methods).
  - `psi/`: **736 `.kt`/`.kts` files, each with a `.txt` PSI dump** (plus `.stubs.txt` and decompiled variants, 2,376 `.txt` files in total), in 61 directories including `contextParameters/`, `stringTemplates/`, `recovery/`, `kdoc/`, `script/`, `richErrors/` and `newLabels/`. About 13.6k lines of Kotlin input. **243 of the dumps contain `PsiErrorElement`**, so they cover error recovery.
  - `lexer/`: 20 pairs. `blockCodeFragment/`: 10 pairs. `expressionCodeFragment/`: 2 pairs.
- The dump format is lossless and easy to reproduce. Example:
  ```
  FUNCTION
    PsiElement(fun)('fun')
    PsiWhiteSpace(' ')
    PsiElement(IDENTIFIER)('foo')
  ```
  Whitespace and comments appear as explicit leaves, so the dump also verifies trivia ownership.
- **Scaling beyond the fixtures:** write a roughly 100-line Kotlin/JVM dumper on `kotlin-compiler-embeddable` at the same version ktlint pins. It builds a `KtFile` via `KtPsiFactory` and prints `DebugUtil.psiToString(file, false, true)`. Run it over large corpora such as ktor, Exposed, kotlinx.*, the Kotlin repo's own `compiler/testData/**` (tens of thousands of `.kt` files) and Android OSS code, then diff against the Rust dump. Cache the JVM dumps keyed by file hash.
- **Cheap invariants on every run:**
  1. Losslessness: the concatenated leaf text equals the input.
  2. "Has an error" matches kotlinc for every file.
  3. No panics, via cargo-fuzz.
- **Differential fuzzing:** apply token-level mutations (delete, duplicate or swap tokens; insert newlines) to corpus files and compare dumps. This exercises recovery and newline sensitivity.

## 5. Expected throughput

| Parser | Output | Measured | MB/s |
|---|---|---|---:|
| oxc (JS/TS) | arena AST, not lossless | typescript.js 8.21 MB in 26.3 ms; cal.com.tsx 1.06 MB in 3.4 ms (M3 Max) | **about 310** |
| swc | AST | 84.1 ms / 13.4 ms | about 98 / 79 |
| **Biome** | **lossless CST (rowan-derived)** | 130.1 ms / 16.7 ms | **about 63** |
| Ruff (hand-written RD) | AST | 2.2–2.4× faster than LALRPOP; full lint of CPython in 94 ms | n/a |
| tree-sitter-kotlin (via Python, loaded machine) | CST | 30.4 MB in 36.7 s | about 1 (not representative) |

MB/s figures are my calculations from the published benchmark times and the input sizes in the oxc benchmark repo.

**Expectation for a rowan-style Kotlin port: about 50–100 MB/s single-threaded,** in Biome's range. PsiBuilder-style rollback on `<` is rare and cheap. At 60 MB/s a 1M-LOC Kotlin codebase (about 35 MB) parses in about 0.6 s on one core and well under 0.1 s on a parallel file walk. Parsing will not be the bottleneck; formatting and rule passes will be.

## Sources

- Kotlin spec grammar: https://kotlinlang.org/spec/syntax-and-grammar.html ; https://github.com/Kotlin/kotlin-spec/tree/release/grammar ; issue #103
- JetBrains/kotlin `compiler/psi/parser` and `compiler/psi/psi-impl/testData` @ c92aefba
- https://github.com/fwcd/tree-sitter-kotlin (issues, README cross-validation) ; https://github.com/tree-sitter-grammars/tree-sitter-kotlin/issues ; https://github.com/dependencyskills/dependencyskills/issues/12
- https://astral.sh/blog/ruff-v0.4.0
- https://github.com/oxc-project/bench-javascript-parser-written-in-rust
- ktfmt `core/src/main/kotlin/org/jetbrains/ktfmt/format/Parser.kt:75`; ktlint `KtLintParseException.kt`, `gradle/libs.versions.toml`
