# Parser profile: where the remaining time goes (2026-09-26, commit 5ae7b34)

Measured on the Linux testbox (12 cores, shared, load ~7), full corpus (6123 files, 30.8 MB),
`perf record -g` with frame pointers, plus temporary work counters (reverted; not in tree).
Parser alone: 24.6 MB/s single-thread; lexer alone 274 MB/s.

## Verdict

No algorithmic hot spot is left. Every work counter is linear in input size with a small
constant; the time is spread over tree construction, token queries and marker bookkeeping.
The remaining big lever is structural (the rowan tree), not a missing cache or a quadratic loop.

## Work counters (per file lexeme, whitespace included)

| Counter | Ratio | Reading |
|---|---|---|
| `advance_lexer` | 1.43x | Parser logic + rollback re-advances |
| lexemes rewound by `rollback_to` | 0.04x | Speculative parsing is cheap here |
| markers created | 1.99x | |
| lexemes copied into chameleon builders (BLOCK 0.46 + LAMBDA 0.48) | 0.94x | Copy, never re-lexed |
| `advance_balanced_block` tokens | 0.66x | One scan per lazy nesting level |
| `newline_before_current_token` iterations | 0.28x | |
| `tt()` / `at()` / `at_set()` per parsed lexeme | 3.3 / 2.3 / 0.3 | Soft-keyword text compares 0.12 |
| `index_of` average scan length | 6.2 entries | No quadratic `precede`/`drop` |

Lambdas are *not* parsed twice: with `is_lazy`, `parse_function_literal_2` skips the body with
`advance_balanced_block` and only the chameleon reparse parses it.

## Self time (parser bench, includes its lex-only pass and tree drop)

| Area | Share | Notes |
|---|---|---|
| Tree construction | ~26% | `create_leaf` 11% (≈40% of it the token interner's hash), `build_tree_into` 10% (whitespace balancing + bind), `finish_node` 3%, malloc |
| Tree drop (rowan `drop_slow`) | 6% | Recursive, atomic refcounts |
| Token queries | ~16% | `at` 7.4, `impl_get_token_type` 5.9, layers 2.2 |
| Marker bookkeeping | ~9% | `mark` ~5 (inlined), `drop_marker` 3.3, `process_done` 2.2, `rollback_to` 1.6 |
| Lexer | ~12% | Includes the bench's separate lex-only pass |
| Grammar functions | rest | Flat; no single function above 1.2% self |

## Candidates, measured

| Idea | Potential | Status |
|---|---|---|
| Chameleon (BLOCK/LAMBDA) green-subtree cache keyed by (kind, text) across one `format` call's parses | 22% of chameleon lexemes reusable | Done (`ChameleonCache`, `parse_file_cached`): format 3.47 -> 3.81 MB/s (+9.5%), 7.85 -> 7.11 parse-equivalents per format |
| Same cache within one file (duplicate lambdas/blocks) | 2.5% of chameleon lexemes | Not worth it alone |
| Binary-search `Production::index_of` (upstream is linear-then-binary) | avg scan already 6.2 | Not worth it |
| Interner: index fixed-text tokens (keywords, punctuation) by kind instead of hashing | ~2-3% of parse | Micro-opt |
| Own flat tree instead of rowan (arena, no per-node alloc, no atomic refcounts, no cursor allocs in visitors) | tree build + drop ~30% of parse; rowan cursor/alloc also ~10% of format time | Big refactor of ktrs_syntax/psi/fmt; the only large lever left |
