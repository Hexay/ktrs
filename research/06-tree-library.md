# Syntax tree library: rowan 0.16 vs cstree vs own flat tree (2026-09-26, commit 5ae7b34)

Source-reading and crate-docs only; no builds run. Profile numbers are from `05-parser-profile.md`
and the `alloc_count` example (`crates/ktrs_fmt/examples/alloc_count.rs`).

## Verdict

- Recommendation: **build our own flat tree.** Do not migrate to cstree.
- cstree does fix the cursor allocations, which are about 25% of format allocations and about 10% of
  format time.
- It does not fix green build and drop, which is about 30% of parse time. Its green trees are rowan's
  design: one `ThinArc` per node and atomic refcounts.
- It would also make our token interner worse. Keys must never be evicted, and reading text needs a
  resolver.
- The flat tree fixes both costs. It also deletes the interner and the three "walk the green tree to
  dodge cursor allocs" workarounds.
- Blast radius is small either way: **14 files** touch tree types directly. Everything else goes
  through `PsiElement`.
- Validate first with a microbenchmark before committing (see "Prototype first").

## 1. rowan API surface

Wrapper layers localize everything:
- `ktrs_syntax` re-exports `GreenNode/TextRange/TextSize` and aliases `SyntaxNode/Token/Element`
  (`crates/ktrs_syntax/src/lib.rs:11,28-30`).
- `ktrs_psi::PsiElement(SyntaxElement)` (`crates/ktrs_psi/src/element.rs:13`) and `AstNode(PsiElement)`
  (`element.rs:197`). Every typed PSI class is a newtype over `PsiElement` (`crates/ktrs_psi/src/cast.rs:35-69`).
- The roughly 300 navigation calls in psi and fmt (`first_child` 30, `next_sibling` 38, `parent` 31,
  `prev_sibling` 32, `last_child` 26, `.kind()` 63, `.text()` 102) are `PsiElement` methods, not rowan.
  They do not change.

Direct rowan and tree-type call sites (every file that names a tree type):

| Crate | File:line | APIs | Sites |
|---|---|---|---|
| ktrs_syntax | `lib.rs:11-30,52,58` | `Language` impl, `rowan::SyntaxKind`, 3 aliases, `GreenNode`, `TextRange/Size`, `SyntaxNode::new_root` | 10 |
| | `dump.rs:3-45` | `children_with_tokens`, match `SyntaxElement::{Node,Token}`, `token.text()` | 5 |
| ktrs_parser | `builder/sink.rs:5,10,50,66,83` | `GreenToken`, `NodeOrToken` (into_node, match), `GreenNode::new(kind, drain)` | 6 |
| | `builder/interner.rs:10,19,32-36,42,55` | `GreenToken::new/kind/text/clone`, `rowan::SyntaxKind` | 7 |
| | `builder/chameleon_cache.rs:17,25-31` | `GreenNode` clone/`kind()` as cache value | 4 |
| | `kdoc/tests.rs:88-89`, `tests/chameleon_cache.rs:24` (tests) | `GreenNode::new`, `children().to_owned()`, `GreenNode ==` | 4 |
| ktrs_psi | `element.rs:30-162` | `as_node/as_token`, `kind`, `parent`, `first/last_child_or_token`, `next/prev_sibling_or_token`, `text_range`, `first_child`, `first_child_or_token_by_kind`, `children_with_tokens().by_kind` | 16 |
| | `element_text.rs:33-43` | `.green()`, `GreenNodeData::children`, `NodeOrToken`, `token.text()` | 6 |
| | `tree_util.rs:5,104-114` | `preorder_with_tokens`, `WalkEvent`, `descendants` | 5 |
| | `visitor/dispatch.rs:141-143` | `node.children()` | 2 |
| | `kt/file.rs:65,92-93` | `parse.syntax()`, `descendants`, `SyntaxNode ==` | 3 |
| ktrs_fmt | `format/input/tokenizer.rs:12-88` | `&SyntaxNode`, `.green()`, `GreenNodeData::{children,text_len}`, `NodeOrToken`, `text_range`, `parent` | 10 |
| | `format/parser.rs:15,26-28` | green walk `has_descendant_of_kind` | 3 |
| | `format/input/kotlin_input.rs:8,38,125`, `formatter.rs:73`, `redundant_element_manager.rs:76` | `&SyntaxNode` parameter, `.green()` | 5 |
| | 4 test files (`script.rs:129`, `tokenizer_test.rs:17`, `kotlin_input_test.rs:16`, `oracle_diff.rs:37`) | `parse.syntax()` | 4 |
| xtask | none | Uses `psi_dump` strings only | 0 |

**Total: about 90 sites in 14 non-generated source files (plus 5 test files).**

Four places already dodge rowan's cursor for speed:
- `element_text.rs:1-2`
- `tokenizer.rs:7-8`
- `parser.rs:26`
- `element.rs:152-156` (`*_by_kind`)

Any replacement should make these plain walks again.

Current rowan facts (0.16.1, `Cargo.lock:123`):
- Every navigation step `Box`-allocates a `NodeData`, with no free list. A clone is a non-atomic rc
  increment. See [cursor.rs](https://docs.rs/rowan/0.16.1/src/rowan/cursor.rs.html), `NodeData::new`.
- `SyntaxNode` is `!Send` (it holds a `NonNull`).
- Green nodes are `Arc`s with atomic refcounts, and dropping them is recursive.
- rowan 0.17 (2026-08) removed the mutation engine but still allocates per cursor step.

## 2. cstree (0.14.0, 2026-04-22)

Crate facts:
- MSRV 1.85, MIT/Apache.
- Single maintainer, about one release per year.
- Docs: <https://docs.rs/cstree>. Source: <https://github.com/domenicquirl/cstree>.

| Question | Answer (source) |
|---|---|
| Red node allocation | Lazily, on first access to each child, each as its own `Box<NodeData>`. `NodeData::new` also allocates 2 Vecs sized to the green child count (`children`, `child_locks`), so there are about 3 mallocs per realized node. No arena. (`cstree/src/syntax/node.rs`, `NodeData`, `get_or_add_node`) |
| Traversal after first visit | No allocation. It is persistent until the tree drops, and navigation returns `&SyntaxNode` / `&ResolvedNode`. |
| Cost of each child access | A `parking_lot` RwLock read per child slot, plus a write lock on first creation. This is uncontended, but it is still an atomic RMW per step. |
| Clone | One atomic `fetch_add` on the per-tree refcount. |
| Memory | About 100-120 B `NodeData` + about 24 B per child slot, once per realized node (estimate from field layout). |
| Send/Sync | `SyntaxNode`/`ResolvedNode` are `unsafe impl Send + Sync`. One tree per thread works. |
| Green build | `GreenNodeBuilder` (`start_node/token/finish_node/checkpoint/start_node_at/revert_to`) with `NodeCache`, which dedups nodes with 3 or fewer children. **`GreenNode::new(RawSyntaxKind, children)` is public**, so our manual `TreeSink` shape and inserting cached subtrees still work. Green nodes are `ThinArc`s, atomic, one per node, the same cost model as rowan. |
| Green token construction | Through the builder or `NodeCache` (interns text). **Verify before any commitment** whether a public `GreenToken` constructor exists; the docs pages do not show one. Without one, `sink.rs`'s manual children Vec must go through `NodeCache`. |
| Reusing a subtree across trees | Possible via `GreenNode::new(.., [cached.into()])`. Keys only resolve if **both trees share one interner**. |
| Token text | Tokens store a `TokenKey`, not text. `resolve_text(&resolver)`, or `ResolvedNode` via `new_root_with_resolver`, which puts an `Arc<dyn Resolver>` at the root; `text()` then goes through a dyn call. `Syntax::static_text` covers fixed tokens. |
| Kind | `trait Syntax { from_raw/into_raw(RawSyntaxKind(u32)), static_text }`. Our `SyntaxKind` is `#[repr(u16)]` (`generated/kinds.rs:5`); a manual impl is fine. |
| TextRange | Same `text-size` 1.1 crate as rowan, so no churn. |
| Benchmarks | **None published against rowan.** 0.12 claims "10% faster building" than 0.11 only. rust-analyzer#17491 notes syntree builds about 2x faster than rowan; rowan's structure is optimized for editing, which we never do. |

Consequences for ktrs:
- **The interner.** Our interner is a fixed 1024-slot table that evicts on collision
  (`interner.rs:1-5,12`). Keys cannot be evicted. The interner would have to live as long as every
  tree and cache entry that references it: per `format` call, owned next to the `ChameleonCache`
  and `Arc`-shared into each root. It grows without bound for that call's lifetime and adds a
  hash-map insert per new token.
- **Text access.** Every text read (`element_text.rs`, `dump.rs`, `tokenizer.rs`) needs the resolver.
- **Parse cost.** The same green-node allocation, atomic refcounts and recursive drop remain. The
  26% build + 6% drop in `05-parser-profile.md:33-34` is unaddressed. First-visit red realization
  adds about 3 mallocs per visited node, versus rowan's 1 per visit.

## 3. cstree migration plan (if chosen anyway)

| Step | Files | Lines touched |
|---|---|---|
| 1. `Syntax` impl, aliases to `ResolvedNode`/`ResolvedToken`/`ResolvedElement`, re-export `cstree::text`, `Parse { green, resolver: Arc<TokenInterner>, .. }`, `syntax()` via `new_root_with_resolver` | `ktrs_syntax/src/lib.rs`, `dump.rs` | ~50 |
| 2. Sink: keep `GreenNode::new` for nodes; tokens via `NodeCache` over a shared interner; delete `interner.rs`; the interner moves into a per-format session with `ChameleonCache` | `builder/sink.rs`, `interner.rs` (-57), `chameleon_cache.rs`, `lib.rs` (`parse_file_cached`), `kdoc/tests.rs` | ~150 |
| 3. `PsiElement` holds an owned `ResolvedElement` (clone = atomic inc). Replace `*_by_kind` with `children_with_tokens().filter`. The green walk in `element_text.rs` takes the resolver. | `element.rs`, `element_text.rs`, `tree_util.rs`, `dispatch.rs`, `kt/file.rs` | ~80 |
| 4. Tokenizer and has-descendant green walks | `tokenizer.rs`, `parser.rs`, `kotlin_input.rs`, 2 call sites, 4 tests | ~50 |

- **Total: about 330 lines, 1-2 days.**
- Incremental path: none worth having. Rowan and cstree types cannot coexist behind `PsiElement`
  without a trait layer, so it is one atomic switch behind the existing gates.
- Risks:
  - The `GreenToken` constructor gap above.
  - Unbounded interner lifetime.
  - Per-step lock overhead eating the allocation savings.
  - Red memory about 2-3x green.
  - A small upstream (2 open issues, one maintainer).

## 4. Own flat tree

Design (one `Tree` per parse; the node handle is an index; `SyntaxKind` stays):

```
struct Tree { text: Box<str>, kind: Vec<SyntaxKind>, start: Vec<u32>, end_idx: Vec<u32> /* preorder subtree end */,
              parent: Vec<u32>, prev_sib: Vec<u32> }            // ~18 B/element, 5 Vecs, tokens included
first_child(i) = i+1 if end_idx[i] > i+1;  next_sibling(i) = end_idx[i] if < end_idx[parent[i]]
text_range(i) = start[i]..start[end_idx[i]] (sentinel);  text(i) = &tree.text[range]   // no alloc, no interner
```

- **Build.** `TreeSink` is already start/token/finish (`sink.rs:72-85`), so it maps to preorder
  pushes: a node pushes on `start_node` and patches `end_idx` on `finish_node`. Per token the cost
  is 5 Vec pushes: no malloc, no interner hash. `create_leaf` is 11% of parse, and about 40% of that
  is the interner hash (`05-parser-profile.md:33`).
- **Drop.** Freeing 5 Vecs replaces the recursive atomic `drop_slow` (6%).
- **ChameleonCache.** Store a position-independent slice `(kinds, lens, rel_end, rel_parent)`. A hit
  memcpy's it into the sink with rebased indices: O(subtree), but far cheaper than reparsing. The
  cache's exactness argument (`chameleon_cache.rs:5-7`) still holds, because the subtree is still a
  function of (kind, text).
- **Handle.** `PsiElement { tree: Rc<Tree>, idx: u32 }`:
  - Navigation is index arithmetic, and clone is a non-atomic increment.
  - Equality is (tree ptr, idx), which matches today's identity semantics (`element.rs:11`).
  - `Rc` keeps it `!Send` like rowan today. `Tree` itself is `Send`, so `Parse` stays `Send`.
  - Using `&'t Tree` would put a lifetime on every PSI type, so it is not worth the churn.
- **Wins beyond speed.**
  - `text()` can later return `&str` (102 `.text()` sites; keep `String` initially for zero churn).
  - `element_text.rs`, the tokenizer and `has_descendant_of_kind` become trivial loops or slices.
  - `has_descendant_of_kind` becomes `kind[i..end].contains`.

| Step | Files | Lines |
|---|---|---|
| 1. `ktrs_syntax/src/tree/{mod,builder,nav}.rs` (Tree, TreeBuilder, cursor-free iterators: children, preorder_with_tokens, descendants), `Parse { tree, .. }`, port `dump.rs` | ktrs_syntax | ~400 new, 70 changed |
| 2. `TreeSink` over `TreeBuilder`; delete `interner.rs`; cache stores slices; kdoc test builds via builder | ktrs_parser builder | ~150 |
| 3. `PsiElement(Rc<Tree>, u32)`; rewrite `element.rs` bodies (signatures unchanged), `element_text.rs` (slice), `tree_util.rs`, `dispatch.rs`, `kt/file.rs` | ktrs_psi | ~150 |
| 4. Tokenizer and has-descendant over indices; `KotlinInput::new(&str, &PsiElement or TreeRef)`; tests | ktrs_fmt | ~80 |

- **Total: about 850 lines, about 60% of them new, isolated code in `ktrs_syntax`. 3-5 days.**
- Risks:
  - **Parity:** `dump.rs` is the parser gate itself, so bugs surface immediately in
    `cargo test -p ktrs_parser`.
  - The own-code maintenance burden is about 400 lines.
  - Chameleon splice bugs: index rebasing. Guarded by `tests/chameleon_cache.rs`, which compares
    cached vs uncached trees and becomes a `Tree ==` check.
  - The corpus is 30.8 MB, so offsets fit in u32.
  - Loss of rowan mutation/`replace_with` APIs: unused today.

Incremental path, with gates green at every step:
1. **Tree beside green.** `TreeSink` builds both, `Parse` carries both, and a debug assert or test
   checks `Tree`-dump == green-dump across fixtures and corpus (`cargo test -p ktrs_parser`,
   `cargo corpus-diff`).
2. **Switch ktrs_fmt's green walks** (tokenizer, has-descendant) to `Tree`. Gates: `ktrs_fmt` golden
   and `cargo fmt-diff`.
3. **Switch `PsiElement`** to `(Rc<Tree>, u32)`. Gates: `cargo test -p ktrs_psi` (JVM accessor
   hashes) and golden/fmt-diff.
4. **Drop green from `Parse`, `interner.rs` and the rowan dependency**, then move `ChameleonCache` to
   slices. Gates: all, plus the `ktrs_fmt` bench "format = N parses" line.

## 5. Expected speedups

- **Parse.**
  - Tree build and drop are about 32% of parse (`05-parser-profile.md:33-34`). Flat pushes should
    keep perhaps a quarter of that: whitespace balancing/bind in `build_tree_into` stays.
  - That gives about 0.76x time, **about 1.3x parse** (24.6 MB/s to about 32 MB/s).
  - cstree gives **about 1.0x**: the same green model, plus interner-map hashing.
- **Format.**
  - Parse is about 3 of about 7.1 parse-equivalents per format (`05-parser-profile.md:44`), about
    40%. So the parse gain is worth about 10% of format.
  - Cursor allocations are about 10% of format time (`05-parser-profile.md:48`) and about 25% of
    format heap allocations. The flat tree removes all of them; cstree removes the repeats but pays
    about 3 mallocs + locks on first visit.
  - Flat total: **about 1.2-1.25x format** (3.81 MB/s to about 4.6-4.8), more once
    `text()`/`AstNode::text` return `&str`.
  - cstree: optimistically **1.05-1.08x**, possibly negative given 3 parses/format where many nodes
    are visited only a few times.

## Outcome (2026-09-26): migrated, rowan removed

Done in commits ead2b0d (tree beside green), e83dfd4 (PsiElement over the tree), and the rowan
removal after it. Measured on the testbox, two A/B rounds per step:

| | Before migration | After |
|---|---|---|
| Parser alone (parser `bench`) | 23.6 MB/s (with both trees built) | 31.2 MB/s |
| Format (fmt `bench`) | 3.81 MB/s | 4.49 MB/s (1.18x) |

The prototype below was deleted with rowan (`tree_bench` needed it); `git show 8a8fefc` has it.

## Prototype result (2026-09-26, commit a6d85d6): GO for the flat tree

`cargo run -p ktrs_parser --release --example tree_bench corpus 3 3` on the testbox: each file's parse is
replayed into rowan (built exactly like `TreeSink` + interner) and into a flat preorder tree.
Walks are visitor-style: child/sibling steps with cloned `Rc<Tree>` handles, token text reads, and one
child-by-kind lookup per node. The walk checksums match on all 6123 files.

| | build | 3 walks | drop |
|---|---|---|---|
| rowan | 0.224 s, 4.66M allocs | 0.668 s, 35.6M allocs | 0.088 s |
| flat | 0.059 s, 53K allocs | 0.311 s, 0 allocs | 0.001 s |

- **Build + drop is 5.2x faster** (the bar was 4x). Rowan's build + drop is 0.31 s of the parser's
  ~1.15 s on this corpus, so parse should gain about 1.3x, as estimated.
- **Walks are 2.2x faster, with zero allocations** (the bar was 2x).
- cstree was not benchmarked. Its green model is rowan's (one `ThinArc` per node, recursive atomic
  drop), so its build and drop cost matches rowan's by construction.

## Prototype plan (as run above, minus cstree)

1. In `ktrs_syntax/examples/tree_bench.rs` (or a scratch crate), take `Parse.green` from
   `parse_file` over `corpus/`.
2. Convert it to (a) a flat `Tree` and (b) a cstree green tree (`GreenNode::new` + `TokenInterner`).
3. Time these with the parser bench's cycle clock (`crates/ktrs_parser/examples/bench.rs`):
   - **Build:** flat pushes vs rowan `GreenNode::new` vs cstree, replaying the same start/token/finish
     event stream recorded once.
   - **Drop.**
   - **Full `preorder_with_tokens` walk**, run k = 1, 3, 6 times. That mimics visitor passes and
     measures cstree's first-visit vs revisit trade.
   - **Visitor-style walk:** `first_child`/`next_sibling` recursion that also clones handles, like
     `PsiElement` does.
4. Count allocations with the `alloc_count` global allocator.
5. **Go/no-go for the flat tree:** build + drop at 4x or more faster than rowan, and a walk with zero
   allocations at ≥2x faster. Also verify cstree's `GreenToken` constructor gap, which settles its
   viability either way.
