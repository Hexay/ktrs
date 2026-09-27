# Replay reparse: which parser decisions see whitespace (2026-09-27)

Idea: after a whitespace-only (and comment-text) edit, re-lex, then run only `balance_white_spaces` +
`bind` (`builder/tree.rs`) on the old parse's production. Exact iff every recursive-descent decision
that observed trivia/offsets/raw indices answers the same on the new lexing. Audit of
`crates/ktrs_parser/src/parsing/**` + the builder code it calls, at commit 8482a62.

Notation: *k* = ordinal of a non-trivia token; *gap(k)* = the trivia run (WHITE_SPACE + comments)
right before token *k*. Classes: **(a)** newline in gap, **(b)** gap empty (adjacency) / raw kind
next to a token, **(c)** text of a non-trivia token, **(d)** offsets/raw indices used otherwise,
**(e)** other.

## 1. Call sites

All raw queries below run with `current_lexeme` at a non-trivia token (a preceding `at`/`tt`/`mark`/
`eof` skipped trivia), so "raw ±1" means "the raw token adjacent to token *k*".

### (a) newline_before_current_token — 15 sites + 1 `eol` caller

Predicate everywhere: `newlines_enabled.top && (eof || gap(k) has a WHITE_SPACE token containing '\n')`
(comments in the gap are skipped, their text is not scanned; `semantic.rs:76`).

| Site | Upstream fn | Use |
|---|---|---|
| expressions/calls.rs:86 | parseCallSuffix | `(` after type args on the same line |
| expressions/calls.rs:107 | parseSelectorCallExpression | call suffix only on same line |
| expressions/control.rs:173 | parseLabelReferenceWithNoWhitespace | `return@l` label on same line |
| expressions/operations.rs:148 | parseDoubleColonSuffix | `foo::bar(` reserved-syntax error |
| expressions/primary.rs:228 | interruptedWithNewLine | binary-op loop stops at newline |
| expressions/statements.rs:42 | parseStatements | "Unexpected tokens (use ';' ...)" |
| expressions/statements.rs:84 | parseBlockLevelExpression | `@Ann\nexpr` vs annotated expr (rollback) |
| expressions/atomic.rs:77 | parseAtomicExpression | rollbackIfDefinitelyNotExpression |
| abstract_kotlin_parsing.rs:73 | errorWithRecovery | EOL_OR_SEMICOLON in recovery set |
| abstract_kotlin_parsing.rs:138 | tokenMatches (`_at`/`at`) | `at(EOL_OR_SEMICOLON)` — hottest |
| abstract_kotlin_parsing.rs:182 | tokenMatchesSet (`_atSet`/`atSet`) | sets containing EOL_OR_SEMICOLON |
| abstract_kotlin_parsing.rs:258 | eol | `newline || eof` |
| declarations/preamble.rs:65 | parsePackageName | package name on one line |
| declarations/preamble.rs:200 | closeImportWithErrorIfNewline | import on one line |
| declarations/properties.rs:134 | parseProperty | `;` then newline ends accessors |
| declarations/properties.rs:72 (via eol) | parseProperty | isNameOnTheNextLine |

Also reached indirectly by `skip_until`/`error_until` (stop at EOL_OR_SEMICOLON).

### (b) adjacency / raw neighbour kind — 7 explicit + complex-token joining

| Site | Upstream fn | Predicate |
|---|---|---|
| expressions/calls.rs:186 | isAtLabelDefinitionOrMissingIdentifier | IDENTIFIER immediately followed by `@` (gap(k+1) empty ∧ kind(k+1)=AT) |
| expressions/control.rs:174 | parseLabelReferenceWithNoWhitespace | gap(k) before `@` non-empty → extra error item only |
| expressions/primary.rs:48 | parseLabelReference | `@` not immediately followed by IDENTIFIER → "Label must be named" |
| declarations/types.rs:176 | parseNullableTypeSuffix | QUEST immediately followed by COLON (`?:` joined regardless of state) |
| declarations/annotations.rs:56 | parseAnnotationOrList | raw token after `@`: IDENTIFIER/target/LBRACKET only if adjacent, else trivia → error path |
| declarations/annotations.rs:221 | parseAnnotation | gap(k) before `(` non-empty ∧ mode.withSignificantWhitespaceBeforeArguments |
| declarations/annotations.rs:240 | isNextRawTokenCommentOrWhitespace | inside `debug_assert!` only — no release effect |
| builder/semantic.rs:154,159 | getJoinedTokenType (from `getTokenType`, `advanceLexer`, `getTokenText`) | when join enabled: QUEST+DOT → SAFE_ACCESS, QUEST+COLON → ELVIS, EXCL+EXCL → EXCLEXCL iff gap empty. Lexer never emits `?.`/`?:`/`!!` (rules.rs:159,190), so **every** `tt()`/`at()`/`advance()` at a QUEST/EXCL observes adjacency; `advance` also emits a collapse marker |

### (c) text of a non-trivia token — 5 (+1 KDoc)

| Site | Upstream fn | Predicate |
|---|---|---|
| abstract_kotlin_parsing.rs:48 | expect | IDENTIFIER expected and token text == "`" |
| abstract_kotlin_parsing.rs:153 | at | IDENTIFIER text == soft keyword text → remap |
| abstract_kotlin_parsing.rs:200 | atSet | softKeyword-by-text ∈ set → remap |
| expressions/strings.rs:80 | parseStringTemplateElement | `$name` text is a hard keyword → remap + error |
| declarations/user_types.rs:84 | recoverOnParenthesizedWordForPlatformTypes | text == "Mutable"/"out" |
| kdoc/kdoc_parser.rs:38 | (KDocParser) tag name | KDoc chameleon only |

### (d) offsets / raw indices used for something else — 13

| Site | Upstream fn | Use | Replay-safe? |
|---|---|---|---|
| parsing/token_stream.rs:21 | matchTokenStreamPredicate | offset of token *k* → FirstBefore/LastBefore result → Truncated `eof_position` (functions.rs:133) | yes: only compared with other non-trivia starts (monotone in *k*) |
| builder/layers.rs:61 | Truncated.eof | `offset(k) >= eof_position` | yes (same) |
| builder/layers.rs:91 | Truncated.lookAhead | `rawTokenTypeStart` of a non-trivia target vs eof_position; raw lookahead skips trivia | yes |
| parsing/optional_marker.rs:16,26 | OptionalMarker.error | offset equality → "no empty errors" | yes: equality ⇔ same *k* (non-trivia starts strictly increase; zero-width DANGLING_NEWLINE still distinct) |
| expressions/strings.rs:98,107 | parseStringTemplateElement | progress check in `${` loop | yes (equality) |
| declarations/parameters.rs:109,130 | valueParameterLoop | progress check | yes (equality) |
| abstract_kotlin_parsing.rs:13 | getLastToken | char offset as loop bound for a trivia walk-back | yes: bound ≥ token count; result = previous non-trivia kind |
| abstract_kotlin_parsing.rs:283 | (test-only context string) | text around caret | n/a |
| builder/psi_builder.rs:151 | PsiBuilderImpl.error | drop 2nd error at *equal raw* lexeme | yes: pre/post-skip forms of one gap differ in raw index, but `duplicate_error_items` (tree.rs:96) dedupes after balancing, same output |
| builder/psi_builder.rs:196 | processDone | done = raw `current_lexeme` (pre- or post-skip) | yes, see §2 |

### (e) other — 4

| Site | Upstream fn | Predicate | Replay-safe? |
|---|---|---|---|
| abstract_kotlin_parsing.rs:14,17 | getLastToken (annotations.rs:229, properties.rs:147) | previous non-trivia kind | yes |
| declarations/mod.rs:134-137 | checkUnclosedBlockComment | last raw token is BLOCK/DOC comment whose text doesn't end with `*/` | recompute 1 bit (only an unclosed comment can fire; it always runs to EOF) |
| declarations/functions.rs:70,73 | parseFunction (2nd type-param list) | `count` = **raw** tokens (incl. trivia) spanned, then `advance` × count **non-trivia** tokens — upstream quirk, over-advances by the trivia count | **no**: record count or refuse replay (fires only on invalid code) |
| builder/semantic.rs:204 | lookAhead (join enabled) | joins the looked-ahead QUEST/EXCL using raw token **cur+2** regardless of `steps` | record: result differs when gap(k+1)/gap(k+2) emptiness changes; sites comparing with QUEST: operations.rs:164 (skipQuestionMarksBeforeDoubleColon); functions.rs:166 runs with join disabled |

Not whitespace-sensitive: `look_ahead` (skips trivia), ForByClause layer (counts newline-stack
calls), `is_empty` in processDone, `raw_advance_lexer` (unused by the parser).

## 2. Marker lexeme indices

| Source | Lexeme recorded | Form |
|---|---|---|
| `mark()` (psi_builder.rs:128) | skips trivia first, except the root marker (raw 0) | token *k* |
| `error()` item | raw `current_lexeme` | token *k*, or start of gap(k) if nothing skipped since `advance` |
| `process_done` / `collapse` / `error()` marker | raw `current_lexeme` | same two forms |
| `precede`, `done_before`, `done_before_with_error_item` | copies an existing start lexeme | token *k* |
| `rollback_to` | resets cursor to a `mark()` lexeme, `token_type_checked = true` | token *k* |
| complex-token collapse (semantic.rs:172) | `mark()` then 2 × advance (adjacent, no trivia) | *k* / gap-start |

Invariant: `current_lexeme` is only ever a non-trivia token, one past one (gap start), or
`lexeme_count`; nothing moves it into the middle of a trivia run (`raw_advance_lexer` is unused;
`token_type_checked` is only set after a skip or a rollback to a non-trivia lexeme). The two forms
of gap(k) are equivalent for `balance_white_spaces`: for any lexeme L in [gap start, gap end],
`ws_start = walk_back(max(L, last))` bounded by `prev` and `ws_end = shift_forward(L)` give the same
run, so the balanced index is independent of L. Encoding every lexeme as ordinal *k* ("gap before
token *k*"; EOF = n; root start = raw 0) is lossless for the tree phase. The newline stack
(`layers.rs`) never touches marker lexemes.

Must also be stored: final `lex_types` of non-trivia tokens (`remap_current_token` — soft keywords,
IDENTIFIER, keyword-in-template — mutates kinds, survives rollback, and becomes leaf kinds), error
messages, binders, `collapsed` flags. `balance_white_spaces` mutates lexemes in place: snapshot
the production (in *k* form) before `build_tree_into`.

## 3. Lazy chameleons

`reparse_lazy` (kotlin_parser.rs:70) runs during `bind` for collapsed BLOCK / LAMBDA_EXPRESSION
leaves (fresh `PsiBuilder` over the leaf's unremapped outer lexemes, fresh `Parser`:
newlines/join stacks at defaults, no layers, `is_lazy = true`) and for DOC_COMMENT /
KDOC_MARKDOWN_LINK (KDoc lexer + parser on the text). A nested parse sees only its own text:
`raw_lookup(-i)` stops at the leaf start, EOF is the leaf end. Its range comes from
`advance_balanced_block` (brace counting on non-trivia kinds), so in *k* terms it is invariant.
`ChameleonCache` (keyed by kind + exact text, error-free only) misses as soon as a body's
whitespace changes; the nested production is dropped after `run_into`.

Per chameleon a replay needs: kind; non-trivia range [k_start, k_end) in the parent; its own
production in *k* form (relative); its final kinds/remaps; its signature (§5); its child
chameleons recursively. KDoc: always reparse from text (whitespace-significant lexer, ktfmt
rewrites KDoc; cheap), or cache hit. Because chameleons are independent, one whose own non-trivia
token range is unchanged can be replayed even when the file-level token sequence changed (import
sorting, trailing commas).

## 4. Lexer

Non-trivia tokens depend on whitespace only through adjacency, all visible in the non-trivia
(kind, text) sequence:

| Case | Where | Effect |
|---|---|---|
| Shebang | kotlin/rules.rs:79 | `#!` only at offset 0 **and** only if a `\n` follows; else HASH, EXCL, ... (adding a final newline flips it) |
| `!in`/`!is`, `as?` | rules.rs:169, 202 | depend on the next char (identifier part / `?`) |
| Merges/splits | operators, numbers `1..2`, identifiers | removing/adding trivia between tokens |
| Comments | mod.rs:242 | nesting and `*/`; changed comment text can swallow or expose code |
| String states, `${}` brace count | mod.rs:162-239, rules.rs:45 | whitespace inside a string is REGULAR_STRING_PART (not trivia); `${ }` trivia is ordinary WHITE_SPACE |
| DANGLING_NEWLINE | mod.rs:164 | zero-width token before `\n` in an unterminated string; skipped in `insert_leaves` |

No state carries across a trivia token otherwise, so "same non-trivia (kind, text) sequence" ⇒ the
same lexer decisions. KDoc lexer (kdoc/*) is whitespace-significant but only runs in the KDoc
chameleon.

## 5. Verdict

Replay of one parse unit (file or chameleon) is exact iff all of:

1. **Token check**: new non-trivia (orig kind, text) sequence == old (covers (c), §4, shebang).
2. **Query log** recorded during the original parse, re-evaluated on the new lexing:
   - `NL(k)` for each evaluated `newline_before_current_token` (only when newlines are enabled and
     not at EOF);
   - `ADJ(k)` = gap(k) empty, for each `raw_lookup(±1)` site in (b), each join evaluation where
     kinds are QUEST→DOT/COLON or EXCL→EXCL, and the `lookAhead` cur+2 quirk (record the raw
     kind class at cur+2: non-trivia kind vs trivia);
   - the `checkUnclosedBlockComment` bit.
   Queries from rolled-back speculative parses count too. Simplest implementation: log at the
   primitives (`newline_before_current_token`, `PsiBuilder::raw_lookup`) as `(k, steps, normalized
   result)`; everything else is either (c) or invariant (d).
3. **Bail-out**: the `parseFunction` raw-count quirk (functions.rs:70) fired → full reparse.
4. Stored for rebuild: production in *k* form + final kinds, per chameleon, recursively.

Risky spots: the join (`?.`/`?:`/`!!` built from adjacent QUEST/EXCL — every `tt()` at `?`/`!`
observes a gap); `lookAhead`'s cur+2 join quirk; `parseAnnotation`'s whitespace-before-`(` rule
(`@Ann (x)` in type contexts); label `@` adjacency; the functions.rs raw-count quirk; the
pre-/post-skip lexeme forms (safe only because balancing normalizes them — keep the root marker at
raw 0); remaps must be replayed; keeping nested productions alive (today dropped, and absent on
cache hits). NL queries dominate volume (`at(EOL_OR_SEMICOLON)`, binary-op loop), but the
file-level parse skips bodies via `advance_balanced_block`, so per-unit logs stay small. A flipped
answer with an identical tree just costs a fallback reparse.

## 6. Outcome (2026-09-27): implemented, exact, no gain — reverted

Built as a second, trivia-insensitive index in `ChameleonCache` (records per BLOCK/LAMBDA parse,
replay on a text miss); the code is in `09-replay-reparse.patch` (apply on 8482a62 + later).

- Exact: `KTRS_VERIFY_REPLAY=1` (every replayed body re-parsed and compared) over fmt-diff in all
  three styles, corpus-diff 6123/6123, and a fixture test with five whitespace perturbations.
- Hit rate after pretty-print: 72% with raw lookups logged by exact result; 98.6% once the hot
  sites log the question they ask (`raw_is(1, AT)`, `raw_is_trivia(-1)`, the `?.`/`?:`/`!!` joins).
- Speed: format/parse ratio 6.00/6.19/6.36 vs 6.09/6.12/6.21 base (testbox, load ~11), plain
  `parse_file` +0.8%. Recording every body's parse (query log, ordinal snapshot, token copy) on the
  earlier passes costs about what replay saves: it skips only recursive descent, and balancing,
  binding and tree building (half of a parse) still run.
- Would need: recording that is nearly free (no token-text copy, no per-parse snapshot) or a
  replay that also skips tree building (splice the old subtree, re-derive only whitespace leaves).
