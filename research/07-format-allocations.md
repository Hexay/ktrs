# Format allocations (2026-09-26, with ChameleonCache)

dhat-rs heap profile of `ktrs_fmt::format` (META) over `corpus/okhttp` (617 files, 4.5 MB), testbox.
Whole corpus with a counting allocator: `parse_file` 0.93 allocations/token; `format` 15.3/token
(9.2 GB requested for 30.8 MB, mostly two up-front `Vec<Op>` reservations per file of `tokens * 4`
large `Op`s — untouched pages, cheap). Allocation + free ≈ 12-15% of format CPU time (perf).

## Allocation blocks by site (okhttp: 13.4M blocks)

| Share | Site | Fix |
|---|---|---|
| **~35%** | rowan cursor `NodeData` per handle: `first_child_or_token_by_kind` 10.6% (ktrs_psi `find_child_by_type` accessors), `next/prev_sibling_or_token` ~12%, visitor child iteration (`accept_children`: AddVisitor 6.1%, DropVisitor 2.6%), `first_child_or_token` 2.3%, `by_kind` 1.0% | Flat tree (research/06-tree-library.md) |
| ~10% | rowan green nodes (`TreeSink::finish_node` 8.7%, token interner misses 0.9%) | Flat tree |
| 13.4% | Tokens: `Rc<KotlinTok>` per tok 7.2%, `Rc<KotlinToken>` per token 4.5%, `toks_before/after` Vecs 1.7% | Arena + indices in `KotlinInput`; `Input`/`Token` traits hand out borrows (self-referential today, so index ranges) |
| 9.7% | Doc engine: `Box<Level>` in `DocBuilder::build` 5.0%, `Level::add` Vec growth 4.7% | Levels stay in the builder's arena (`DocKind::Level(index)`); size child Vecs from ops |
| ~8% | Strings: `str::to_owned` 4.1%, `push_str` growth 3.8% (incl. `element_text::text` for names 2.5%) | Borrowed `&str` for single-token text; reserve |
| 1.8% | `Tokenizer::split_whitespace_newlines` | |
| 1.3% | `FqName` + `String` box clones (import detector) | |

## Follow-up (after the flat-tree migration and commit 7cd5fdf)

Allocations on okhttp dropped from 13.4M to about 7M. Replacing the per-tok and per-token `Rc`s with
list+index handles (`TokRef`/`TokenRef` derefing to `dyn Tok`) removed another ~20% of the
allocations but did not change time (format/parse ratio 6.33 vs 6.32, three A/B rounds): mimalloc
makes small allocations cheap, and the handle's extra indirection ate the rest. Reverted. From here
allocation count is not the bottleneck; CPU work is.

## CPU work after the tree migration (commits 8d34a2a..b87d29a)

What paid, measured by the bench's format/parse ratio (stable under load), three A/B rounds each:

| Change | Ratio |
|---|---|
| Trailing-comma suggestions from a preorder scan (8d34a2a) | 6.31 -> 6.10 |
| Redundant-element removal from a preorder scan (f64def9) | 6.18 -> 6.09 |
| Tree building: skip the error-dedup pass when no error items (e606323) | parser-side, corpus-diff clean |
| JavaOutput whitespace-free fast path + lazy whitespace split (b87d29a) | 6.10 -> 5.95 |

What did not pay (reverted): list+index `Tok`/`Token` handles (see above); preloading the source
into `TreeBuilder` to skip per-token text copies (no change: it still copies the source once);
computing `Doc` widths eagerly instead of memoizing (slightly slower, ~1%: it computes widths the
lazy path never reads).

Result at b87d29a on the testbox: format 5.3 MB/s single-thread on a quiet machine (4.9 best-of-3
under load average ~3), parser 30.8 MB/s, 12 threads 19.8 MB/s wall for the 30.8 MB corpus.

## Verdict

Rowan accounts for ~45% of the formatter's allocations and ~30% of parse time; everything else
is a list of 1-2% items. The flat tree is the one change that moves the number; prototype it
first (research/06-tree-library.md, "Prototype first").
