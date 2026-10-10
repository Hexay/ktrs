# 36 — The parser as a library: stable Rust facade, Python and Node bindings

2026-10-09, at ed15609 (v0.5.1). Source reading and web research only; nothing was built or measured (the Windows
box has under 1 GB free). **F** = fact (read in the repo, or checked against a registry/doc today), **I** = inference
or estimate. Registry checks are direct API calls: `crates.io/api/v1/crates/<n>`, `registry.npmjs.org/<n>`,
`pypi.org/pypi/<n>/json`.

## TL;DR

- Today's crates are published but are implementation crates: the porting scaffolding (`PsiBuilder`, markers, token
  stream patterns) is public, the handle type is `Rc`-based (`!Send`), and nothing a third party needs first exists
  (descendants, ancestors, line/column, error-to-node, edits, names of declarations, CRLF handling, examples).
- Proposal: one new facade crate, **`kt-syntax`** (proposed as `kotlin-syntax`; renamed after the trademark check,
  §9, and built, §10), over the existing
  crates, which get documented as unstable internals (the ruff `0.0.x` / `ra_ap_*` model, without renaming them).
- The flat tree is the asset for bindings. Python: PyO3 handles = `Arc<file>` + `u32`, one abi3 wheel per platform.
  JavaScript: export the five arrays once and navigate in JS with no boundary crossings, so **Wasm first** and napi
  only if a benchmark asks for it.
- Kinds are versioned by name, never by number. A Kotlin pin bump that only adds kinds is a minor release;
  `SyntaxKind` becomes `#[non_exhaustive]`.
- Edits are text edits (ast-grep's shape) in v1. `ktrs-ast` stays internal: its API is IntelliJ's, panics included.
- Side finding: **`@ktrs/cli` is not on npm** (404, and `scope:ktrs` search returns 0), so the npm job has never
  published. Whether the `@ktrs` scope is registered is unverified.
- About 4 to 5 weeks of work in six phases; eight owner decisions in §8.
- Phase 0 (§9): the name `kotlin-syntax` conflicts with the Kotlin brand guidelines, `Arc<Tree>` costs 4.5% on
  format (not adopted), and deep nesting overflows the stack.

## 1. Audit of the public Rust API

### What is exposed (F)

| Crate | Public surface | Notes |
|---|---|---|
| `ktrs-syntax` | `SyntaxKind` (293 variants, `#[repr(u16)]`, generated from `kinds.tsv`; `debug_name`, `is_token`, `is_trivia`, `keyword_text`, `hard_keyword`, `soft_keyword`, `from_raw`), `Tree`, `ElementId = u32`, `TreeBuilder`, `tree::{KindScan, Children}`, `Parse { tree: Rc<Tree>, error_messages, missed_tokens }`, `MissedTokens`, `psi_dump`, `TextRange`/`TextSize` (re-export of `text-size`), `caught_panic` | `Tree` navigation is complete for one step: `parent`, `first_child`, `last_child`, `next_sibling`, `prev_sibling`, `children`, `text_range`, `text_of`, `subtree_end`, `has_descendant_of_kind`, `find_kinds`. `Tree` is `Send + Sync`, `Clone`, `PartialEq`. |
| `ktrs-lexer` | `Token { kind, len }`, `tokenize`, `tokens_of`, `tokenize_kdoc` | Small and clean. |
| `ktrs-parser` | `parse_file`, `parse_file_cached`, `FileKind`, `ChameleonCache`, five fragment parsers, and the modules `builder`, `parsing`, `kt_tokens`, `token_set` | `parse_file` requires LF-normalized text and does not say what happens otherwise. |
| `ktrs-psi` | `PsiElement` (`Rc<Tree>` + id), `AstNode`, `PsiType`, 149 typed views, `KtVisitorVoid` + `kt_tree_visitor_void`, `FqName`, `ImportPath`, token sets, `psi_class_name`, `classes` | Scope is "everything ktfmt v0.64 calls" (`src/lib.rs`). Verified against the JVM. |
| `ktrs-ast` | `Ast` arena, `NodeId`, `tree_util`, `code_edit_util`, `psi` (typed views over the arena, `kt_psi_factory`) | IntelliJ `TreeElement` semantics for ktlint ports. |

### Accidentally public (F)

- `ktrs_parser::builder`: `PsiBuilder`, `Marker`, `MarkerHost`, `Layer`, `EdgeBinder`, `TreeSink`, `LazyLeaf`,
  `LazyReparse`, `SemanticWhitespaceAwarePsiBuilder`. `ktrs_parser::parsing`: `Parser`, `Consumer`, `OptionalMarker`,
  the token stream patterns. None of this is usable from outside (`Parser`'s fields are `pub(crate)`), but all of it
  is semver surface on crates.io.
- `ktrs_syntax::caught_panic` is CLI panic-hook plumbing living in the syntax crate.
  `MissedTokens::{log, assertion_error}` reproduce JVM stderr text.
- `TreeBuilder::{extract, push_tree, push_subtree}` exist for the chameleon cache. `Parse`'s fields are public, so
  its layout is frozen.
- `SyntaxKind::from_raw` panics on an out-of-range value, and the raw values are positions in `kinds.tsv`.
- `ktrs_psi`: `classes` (a `pub mod` of membership tables), `get_stub_or_psi_*`, `AstNode`,
  `try_for_each_text_chunk`, and `PsiType::cast_unchecked`, which is safe to call and wraps anything.
  `upcast` panics by design (it is Java's cast).
- `TokenSet` is defined in `ktrs-parser`, but `ktrs-psi` callers need it for `find_child_by_type_set`.

### Missing for third-party use

| Need | State (F) |
|---|---|
| Descendants / preorder | Not on `Tree`. `find_kinds` covers a bounded set of kinds; `ktrs-psi` has `collect_descendants_of_type` (allocates a `Vec`). No ancestors iterator, no leaf iterator on `Tree` (`next_leaf` is on `PsiElement`). |
| Element at offset, covering element | None. |
| Line/column | Only private copies: `ktrs-lsp/src/text.rs` (`LineIndex`, UTF-16 columns) and inside `ktrs-fmt`. |
| Offsets | UTF-8 bytes everywhere, documented. No UTF-16 or code-point conversion outside `ktrs-ast` and `ktrs-lsp`. |
| Error nodes | `ERROR_ELEMENT` nodes are in the tree; messages are a parallel `Vec<String>` in preorder. `PsiErrorElement::error_description` counts error elements from 0 on every call, so listing all errors is quadratic. |
| Typed accessors | ktfmt's needs only. No `name()` on declarations (only `name_identifier`, plus `KtImportAlias::name`), no `doc_comment()`, no `is_data`-style modifier helpers. `ktrs-ast::psi` has `name`, `is_public` and more, but over the arena and at ktlint's Kotlin versions (2.2.21 / 2.4.10). So there are two typed layers with different coverage and different handle types. |
| Comments / KDoc | In the tree: trivia tokens, and `DOC_COMMENT` as a node with `KDOC_SECTION`/`KDOC_TAG`/`KDOC_NAME` children. `KDoc*` views exist, with accessors on `KDocName` only. Nothing attaches a comment to its declaration. |
| Visitors | `KtVisitorVoid` mirrors upstream's super-chain. Good for ports, heavy for a script. No plain enter/leave walk. |
| Mutation | `ktrs-ast` only. Its API is `ast.foo(node)`, it mimics IntelliJ's divergences on purpose, and its panics carry Java exception names. No text-edit type anywhere outside `ktrs-lsp`. |
| Threads | `Parse` and `PsiElement` hold `Rc<Tree>`, so neither is `Send`. Research/06 chose `Rc` for the format hot path. |
| Input handling | No CRLF or BOM handling at the parser entry; callers do it. Offsets are `u32`. |
| Robustness | Parse time is exponential in some nestings, as upstream (research/24 finding 1; fuzzing skips `(` deeper than 10). Behaviour on very deep nesting (stack) is unverified. |
| Docs and examples | Crate-level docs are porting notes for maintainers. No per-crate README, no `readme`/`keywords`/`categories` in the sub-crate manifests, no examples except `bench.rs`, `seed_bench.rs` and the `psi_accessors` oracle mirror. How the crates render on docs.rs is unverified. |

### Semver (F, then I)

- F: every crate shares the workspace version, and each release bumps all of them. The crates.io pages carry no
  stability statement.
- F: `SyntaxKind` is generated from the compiler pin (`v2.4.20`), is not `#[non_exhaustive]`, and its discriminants
  follow file order. `kinds.tsv` has not changed since the first commit, so there is no history of how a bump behaves.
- I: a pin bump can add kinds (breaks exhaustive matches), renumber them (breaks anything that stored a `u16`),
  remove or rename them, and change the tree for the same source. Only the first two can be designed away.
- I: `ktrs-ast::psi` already follows two other Kotlin versions (the ktlint jars'), so "the Kotlin version of the API"
  is not one number across the workspace. The facade should expose only the 2.4.20 tree.

## 2. Proposed stable facade

### One crate or documented internals

A new facade crate, and the existing crates declared unstable. Reasons:

- The existing public surface cannot be made stable without hiding modules that `ktrs-fmt` and `ktrs-lint` import
  across crate boundaries.
- Users want one dependency and one docs.rs page. ruff (`0.0.x`, "no stability guarantees") and rust-analyzer
  (`ra_ap_*`) show what publishing internals without a facade gives: usable, and nobody can rely on it.
- The facade is thin: `Tree` already does the work.

### Name (checked 2026-10-09)

| Name | crates.io | npm | PyPI |
|---|---|---|---|
| `kotlin-syntax` | free | free | free |
| `kotlin-parser` | taken (vyfor, 0.0.2, 2024-12, dormant) | free | free |
| `kotlin-psi`, `kotlin-ast` | free | free | free |
| `ktrs-kotlin`, `kt-syntax` | free | not checked | not checked |
| `ktrs` | ours | taken (unrelated, 2015) | free |
| `ktrs-parser`, `ktrs-syntax` | ours (internals) | `@ktrs/parser`, `@ktrs/syntax` 404 | free |

- Recommended at the time: `kotlin-syntax` everywhere. **Decided after §9: `kt-syntax`** on crates.io, PyPI
  (`import kt_syntax`) and npm, described as "a parser for Kotlin". It says what it is, and it is findable by someone who has never heard of ktrs.
- Reusing `ktrs-parser` would mean breaking it to hide `builder`/`parsing`, and it cannot re-export `ktrs-psi`
  (which depends on it).
- The Kotlin Foundation's guidelines do not allow it as worded: see "Phase 0 results". `tree-sitter-kotlin` and
  `kotlin-parser` exist as precedent, not as permission; the README should say "unofficial" either way.
- I: register `ktrs` on PyPI now. `pip install ktrs` should one day mean the CLI (as with ruff), not the parser.

### Minimal API (sketch)

Borrowed `Copy` handles in Rust: no refcount, `Send + Sync`, and the lifetime is one parameter on one type.
Bindings wrap `Arc<SourceFile>` + id. No third-party types in signatures (`Range<usize>`, not `text-size`).

```rust
pub const KOTLIN_VERSION: &str = "2.4.20";

pub fn parse(text: &str) -> SourceFile;                 // .kt
pub fn parse_script(text: &str) -> SourceFile;          // .kts
// Strips a BOM and normalizes CRLF; every offset refers to `SourceFile::text()`.

pub struct SourceFile { /* Tree, errors indexed by element, OnceLock<LineIndex> */ }   // Send + Sync
impl SourceFile {
    pub fn text(&self) -> &str;
    pub fn root(&self) -> Node<'_>;
    pub fn errors(&self) -> impl Iterator<Item = SyntaxError<'_>>;   // node + message, linear
    pub fn has_errors(&self) -> bool;
    pub fn token_at(&self, offset: usize) -> Option<Node<'_>>;
    pub fn covering(&self, range: Range<usize>) -> Node<'_>;
    pub fn line_col(&self, offset: usize) -> LineCol;                // 0-based line, UTF-8 column
    pub fn line_col_utf16(&self, offset: usize) -> LineCol;
    pub fn offset(&self, pos: LineCol) -> Option<usize>;
    pub fn dump(&self) -> String;                                    // DebugUtil.psiToString format
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node<'a> { /* &'a SourceFile, u32 */ }
impl<'a> Node<'a> {
    pub fn kind(self) -> SyntaxKind;
    pub fn is_token(self) -> bool;       pub fn is_trivia(self) -> bool;    pub fn is_error(self) -> bool;
    pub fn text(self) -> &'a str;        pub fn range(self) -> Range<usize>;
    pub fn parent(self) -> Option<Node<'a>>;
    pub fn first_child(self) -> Option<Node<'a>>;    pub fn last_child(self) -> Option<Node<'a>>;
    pub fn next_sibling(self) -> Option<Node<'a>>;   pub fn prev_sibling(self) -> Option<Node<'a>>;
    pub fn children(self) -> Children<'a>;           // nodes and tokens, trivia included
    pub fn child_nodes(self) -> impl Iterator<Item = Node<'a>>;      // composites only
    pub fn ancestors(self) -> Ancestors<'a>;
    pub fn descendants(self) -> Descendants<'a>;     // preorder = a range of ids
    pub fn tokens(self) -> impl Iterator<Item = Node<'a>>;
    pub fn find_all(self, kinds: &[SyntaxKind]) -> impl Iterator<Item = Node<'a>>;   // scan of the kind words
    pub fn child(self, kind: SyntaxKind) -> Option<Node<'a>>;
    pub fn next_token(self) -> Option<Node<'a>>;     pub fn prev_token(self) -> Option<Node<'a>>;
    pub fn error_message(self) -> Option<&'a str>;
    pub fn replace(self, text: impl Into<String>) -> TextEdit;       pub fn remove(self) -> TextEdit;
}

pub fn walk<'a>(root: Node<'a>, f: impl FnMut(WalkEvent<Node<'a>>) -> Walk);   // Enter/Leave; Walk::{Continue, SkipChildren, Stop}

pub struct TextEdit { pub range: Range<usize>, pub text: String }
pub fn apply_edits(text: &str, edits: Vec<TextEdit>) -> Result<String, OverlappingEdits>;

#[non_exhaustive] pub enum SyntaxKind { .. }
impl SyntaxKind { pub fn name(self) -> &'static str; pub fn from_name(s: &str) -> Option<SyntaxKind>; }
```

Typed layer, phase 2: `kt_syntax::psi`, with the compiler's class and accessor names (`KtNamedFunction`,
`value_parameters`), over `Node<'a>`. Two ways to get it:

1. Re-use `ktrs-psi`. Needs `PsiElement` to be constructible from the facade's file, so `Rc<Tree>` must become
   `Arc<Tree>` (or the facade stays `!Send`, which rules out releasing the GIL and sharing a tree across Python
   threads). Gate: the fmt bench's "format = N parses" line with `Arc`.
2. A generated layer from an accessor table (class, method, child kind, result type), which also generates the
   `.pyi` and `.d.ts`. Covers the simple "child by kind" accessors; the ones with logic stay hand-written.

Recommended: 1 if the `Arc` measurement costs under 1%, plus the table from 2 as the single list of what is exposed
in every language. Whatever is exposed must stay inside what `psi_accessors` verifies against the JVM, and
additions (`name()`, `doc_comment()`, modifier checks) need rows in that oracle first.

Not in v1: the mutable arena, fragment parsers, `ChameleonCache`, a query language, incremental reparse.

### Stability policy

- Semver-stable: everything in `kt-syntax`'s root and `psi` modules. The `ktrs-*` crates say "internal, any
  release may break" in a README and their crate docs.
- Version: the workspace version (release.yml rejects a tag that disagrees, and one number is what the release
  process can carry). While 0.x, a breaking facade change is a minor bump.
- Kinds: names are the compiler's field names and are the stable identity. Numbers are not exposed in the facade
  (`from_raw` stays internal) and bindings serialize names. `SyntaxKind` is `#[non_exhaustive]`.
- Kotlin pin bump: `KOTLIN_VERSION` changes, and the changelog lists added and removed kinds, generated by diffing
  `kinds.tsv`. Added kinds only: minor. A removed or renamed kind: the variant stays, deprecated and never
  produced, until the next breaking release.
- Tree shape for a given source is "what Kotlin `KOTLIN_VERSION` produces". It changing with the pin is documented
  behaviour, not a semver break; the fixture diff of the bump is linked from the changelog.
- MSRV: the workspace `rust-version` (1.90 today). Raised only in a minor release, never in a patch.
- CI: `cargo semver-checks` on the facade against the last release; doc tests for every example.

## 3. Python bindings

### PyO3 + maturin, abi3

- F: PyO3 0.29.3 and maturin 1.15.0 are current. `abi3-py39` gives one wheel per platform for every CPython from 3.9.
  tree-sitter-kotlin does exactly this (`cp39-abi3`, 300 to 350 KB per wheel). ast-grep-py and libcst ship
  per-version wheels instead (about 5 MB and 2 MB).
- F: free-threaded 3.13t/3.14t cannot load abi3 wheels. `abi3t` (PEP 803) starts at 3.15; PyO3 0.29 and
  maturin 1.14+ can build it. So: abi3 now, a `cp314t` wheel only if someone asks, abi3t when 3.15 ships.
- Alternatives: cffi/ctypes over a C ABI (no compiled Python glue, but every node access becomes a Python-level FFI
  call and lifetime handling moves into Python); a Wasm module run with wasmtime-py (one artifact, but a heavy
  runtime dependency and slower calls). Neither beats PyO3 here.

### Object model

- `Tree` (frozen pyclass): `Arc<SourceFile>`. `Node` (frozen pyclass): `Arc<SourceFile>` + `u32`. Creating a node is
  one small allocation and an atomic increment; navigation is the index arithmetic of `tree/mod.rs`.
- `==` and `hash` are (file pointer, id), as in `PsiElement`.
- `node.kind` returns a member of a `SyntaxKind` `StrEnum`-like class, taken from a table of 293 objects built at
  import, so it is a refcount increment. Comparing with a string works (`node.kind == "FUN"`).
- `node.text` slices the source on demand and makes a new `str`. Nothing is copied at parse time except the source.
- Parsing releases the GIL (`SourceFile` is `Send`), so a thread pool over a repository scales.
- Bulk paths keep the loop in Rust: `find_all` (the `KindScan`), `descendants()` as a Rust-side iterator, and
  `walk()` returning a cursor that moves in place (py-tree-sitter's `TreeCursor`).
- I: PyO3 call overhead is tens of ns (PyO3 issue 3787), so a full walk of a 100k-element file through per-node
  calls is in the 10 to 50 ms range. That is why the bulk paths exist.
- Panics are caught at the boundary and raised as `kt_syntax.InternalError`.

### API sketch

```python
import kt_syntax as ks

tree = ks.parse(source)                    # ks.parse(source, script=True); ks.parse_file(path)
tree.kotlin_version                        # "2.4.20"
root = tree.root                           # Node, kind KT_FILE
for fn in root.find_all("FUN"):            # or ks.SyntaxKind.FUN; several kinds allowed
    fn.name                                # typed accessor, None if not a named declaration
    fn.range                               # (start, end) in str indices: source[start:end] == fn.text
    fn.byte_range; fn.start_point          # UTF-8 bytes; (line, column)
    [p.name for p in fn.value_parameters]  # typed view chosen by kind: isinstance(fn, ks.psi.KtNamedFunction)
node.parent; node.children; node.child_nodes; node.next_sibling; node.prev_sibling
node.ancestors(); node.descendants(); node.tokens(); node.is_token; node.is_trivia; node.is_error
tree.errors                                # [SyntaxError(node, message)]
tree.token_at(offset); tree.dump()         # the compiler's psiToString text

edits = [call.replace("bar()") for call in root.find_all("CALL_EXPRESSION") if call.text == "foo()"]
new_source = tree.apply_edits(edits)       # raises on overlap; reparse to continue
```

- Offsets: `range` is in `str` indices so slicing works; `byte_range` is the tree's own. The conversion is a
  per-line prefix table built on first use and skipped for ASCII files. This is an owner decision (§8): tree-sitter
  uses bytes, ast-grep gives both.
- Typed classes and the `.pyi` are generated from the accessor table; `py.typed` is shipped.
- A `Visitor`/`Transformer` pair in pure Python on top of `walk()` and `apply_edits` gives libcst's shape without
  a tree-rewriting engine. Optional `ks.format(source, style=...)` (ktrs-fmt behind a cargo feature) makes
  "edit, then format exactly like ktfmt" one call; decide after v1.

### Packaging and CI

- Layout: `bindings/python/{pyproject.toml, Cargo.toml, src/, python/kt_syntax/, tests/}` as its own cargo
  workspace, excluded from the root one (`fuzz/` is the precedent). This keeps `cargo test --workspace` and the
  manual `cargo publish --workspace` unchanged, and the box never compiles PyO3.
- The root `pyproject.toml` is untouched: it is the `ktrs-launcher` package that pre-commit installs from the
  repository root, with setuptools. PyPI builds come only from `bindings/python`. Its version is
  `dynamic = ["version"]` read from its `Cargo.toml`, which the `versions` job checks like the others.
- Wheels (`PyO3/maturin-action`, no cibuildwheel needed): manylinux_2_28 x86_64 and aarch64, musllinux_1_2 x86_64
  and aarch64, macOS x86_64 and arm64, Windows x64 and arm64, plus an sdist. Eight wheels, the same runners as
  release.yml's `build` matrix. I: 0.5 to 1 MB each (parser and psi only).
- Publish: PyPI Trusted Publishing (`uv publish --trusted-publishing always` or `pypa/gh-action-pypi-publish`).
  There is no secret to test, so gate the step on a repository variable, e.g. `vars.PYPI_PUBLISH == 'true'`.
  `maturin upload` is deprecated since 1.12.

## 4. Node / JavaScript bindings

### napi-rs vs Wasm

| | napi-rs v3 native | Wasm (`wasm32-unknown-unknown`, plain exports as in `ktrs-wasm`) |
|---|---|---|
| Parse speed | Native | I: 1.3 to 2x slower; to be measured with the parser `bench` corpus |
| Install | Main package + one optional package per platform (oxc 19, ast-grep 9; 2 to 10 MB each, F) | One package, one `.wasm` (I: under 1 MB) |
| Browser, Deno, Bun, edge | Only through napi's `wasm32-wasip1-threads` fallback, which needs `SharedArrayBuffer` and COOP/COEP headers (F) | Everywhere |
| Release work | A build per platform (or reuse of the six runners) and 6+ extra npm packages | One build on Linux |
| Per-node cost | A class instance per node crossing the boundary | None with the design below |
| Failure mode | A panic becomes a JS exception | `panic = "abort"` traps; the glue must re-instantiate |

The flat tree decides this. After parsing, the tree is five arrays and a string:

- Rust exports `kinds: Uint16Array` and `starts`, `ends`, `parents`, `prevSibs: Uint32Array`, copied once out of
  Wasm memory (about 18 bytes per element), with `starts` converted to UTF-16 units so that
  `source.slice(start, end)` is the node text. ASCII files skip the conversion.
- Navigation is then about 150 lines of JS mirroring `tree/mod.rs`. A node is `{ tree, id }`. Walking a file makes
  no calls into Wasm at all. oxc's raw transfer and node-tree-sitter's marshalling exist to get near this; here it
  falls out of the layout.
- Typed accessors that need Rust logic call Wasm with integers (`accessor(treeHandle, nodeId) -> id`). That needs
  the tree kept alive on the Wasm side: `tree.free()` plus a `FinalizationRegistry`.
- With this model napi would only speed up `parse` itself. The JS navigation code would be the same.

Recommended: **Wasm only for v1**, with one gate: if Wasm parse throughput is under half of native on the bench
corpus and a user needs more, add napi platform packages behind the same JS API later. swc, biome and tree-sitter
all ship native and Wasm as separate packages, so a later addition does not disturb the first.

### API sketch

```ts
import { parse, SyntaxKind, KOTLIN_VERSION } from "@ktrs/syntax";      // Node, Bun, Deno: sync init at import
// browser: import init, { parse } from "@ktrs/syntax/web"; await init();

const tree = parse(source, { script: false });
for (const fn of tree.root.findAll("FUN")) {
  fn.text; fn.range;            // [start, end] in UTF-16 units; fn.byteRange; fn.startPosition {line, column}
  fn.name;                      // typed accessors, generated
}
node.parent; node.children; node.childNodes; node.nextSibling; node.prevSibling;
node.ancestors(); node.descendants(); node.tokens(); node.isToken; node.isTrivia; node.isError;
tree.errors; tree.tokenAt(offset); tree.dump(); tree.applyEdits([node.replace("x")]); tree.free();
```

```ts
// generated by xtask from kinds.tsv, next to the Rust enum
export type SyntaxKind = "KT_FILE" | "FUN" | "FUN_KEYWORD" | "CALL_EXPRESSION" /* ... 293 */;
export const SyntaxKind: { readonly [K in SyntaxKind]: K };
export interface Node<K extends SyntaxKind = SyntaxKind> { readonly kind: K; /* ... */ }
export interface KtNamedFunction extends Node<"FUN"> { readonly name: string | null; readonly valueParameters: KtParameter[]; }
export function is<K extends SyntaxKind>(node: Node, kind: K): node is NodeOf<K>;
```

- `kind` is the name string (stable across pin bumps), looked up from a 293-entry table by the raw `u16`. A string
  union narrows in `switch`; a numeric `const enum` (what napi-rs generates by default) would bake unstable numbers
  into users' builds.
- Ship ESM with a CJS entry, and `.d.ts` generated from the same tables as the `.pyi`.

### Naming and publish flow

- F: `@ktrs/cli` and every `@ktrs/*` name return 404 today, so the scope either does not exist or is empty. Check
  the org and `NPM_TOKEN` before anything else; this affects the existing npm job too.
- Name: `@ktrs/syntax` next to `@ktrs/cli` keeps one scope and one token. Unscoped `kt-syntax` (free) matches
  the crate and PyPI. Decided: publish `kt-syntax` unscoped; a future native add-on
  would be `@ktrs/syntax-<os>-<cpu>` optional dependencies, the same shape as `@ktrs/cli-*`.
- Source in `npm/syntax/` (JS glue, `package.json` at `0.0.0`, tests), crate in `bindings/wasm/` (own workspace,
  `--profile wasm` settings copied: `panic = "abort"`, `strip`). `ktrs-wasm` stays the playground's formatter module.
- `tools/release/package-npm.mjs` gains a second output: copy `npm/syntax`, add the built `.wasm` and generated
  `.d.ts`, stamp the tag's version. The existing publish loop (`npm view` skip, `--provenance`, `./` prefix,
  `NPM_TOKEN` gate) then covers it by adding the directory to the `for`. npm Trusted Publishing is an option now
  (needs npm 11.5.1+, Node 22.14+) and would remove the token.

## 5. Comparable projects

| Project | What users get (F) | Lesson |
|---|---|---|
| tree-sitter (`tree-sitter-kotlin` fwcd 0.3.8, 2024-08; tree-sitter-grammars 1.1.0, 2025-01, as crate `tree-sitter-kotlin-ng`, npm `@tree-sitter-grammars/tree-sitter-kotlin`, PyPI `tree-sitter-kotlin`) | `Node`: `type`, `children`, `named_children`, `child_by_field_name`, `start_byte`, `start_point`, `text`, `parent`, `next_sibling`, `is_error`, `is_missing`, `walk()` cursor, `Query` | This is the vocabulary people arrive with: kind as a string, bytes and points, named vs anonymous children, a cursor. Map `child_nodes` to "named children". Field names have no PSI equivalent; typed accessors fill that role. |
| ast-grep 0.50.0 (`@ast-grep/napi`, `ast-grep-py`) | `SgRoot(src, lang).root()`; `find`/`find_all` with rule objects; `kind()`, `text()`, `range()`; `parent`, `children`, `next`, `prev`, `ancestors`; `replace()` returns an `Edit`, `commit_edits()` returns a string | The edit model to copy: edits are values, applying them gives text. Kotlin there is a tree-sitter fork (`tree-sitter-kotlin-sg`), and in JS needs `@ast-grep/lang-kotlin`. |
| ruff (`ruff_python_parser`, `ruff_python_ast`) | On crates.io since 2026-06 at `0.0.x`, "no stability guarantees" | Publishing internals is cheap and honest if labelled. It is not a library offering. |
| biome (`biome_js_parser` 0.5.7 from 2024-03, stale; `biome_rowan` 0.7.0) | Crates out of date, open issue about broken builds from crates.io; JS API is a wrapper over three Wasm packages of 45 MB | Half-published crates rot. Either release them with everything else or do not publish. |
| oxc 0.153 (`oxc-parser`) | ESTree JSON by default; `experimentalRawTransfer` reads the arena from a buffer; 19 platform packages of about 2 MB and a wasm32-wasi fallback | Crossing the boundary once with a flat buffer is the state of the art. ktrs has that layout already. |
| rust-analyzer (`ra_ap_syntax` 0.0.357, weekly; rowan) | `SyntaxNode`, `SyntaxKind`, `AstNode` trait with `cast`/`can_cast`, `SyntaxNodePtr`, `SyntaxEditor` | `ktrs-psi`'s `PsiType` is the same pattern. `ra_ap_*` shows auto-published internals are used despite the churn. |
| LibCST 1.9.0 | `parse_module`, `CSTVisitor`/`CSTTransformer`, matchers, metadata providers (positions), codemod framework and CLI; Rust parser via PyO3, per-version wheels | What Python codemod authors expect in the end. Visitor and transformer can be pure Python over text edits; matchers and a codemod CLI are a later layer. |
| swc, tree-sitter | Native and Wasm as separate packages (`@swc/core` vs `@swc/wasm*`; `tree-sitter` vs `web-tree-sitter`) | Nobody gets one package to be both well. Start with the one that runs everywhere. |

Other Kotlin parsers outside the JVM are dormant: kopyt (PyPI, 2021), kotlin-parser-antlr (npm, 2021),
vyfor/kotlin-parser (crates.io, 2024-12).

### The honest differentiator against tree-sitter-kotlin

For ktrs:

- **The tree is the compiler's.** Same kinds, same nesting, same error elements as `DebugUtil.psiToString`,
  gated on the upstream fixtures and a 6123-file corpus. The node names are the ones detekt, ktlint and IntelliJ
  plugin authors already know, and a rule prototyped here ports to those tools by renaming.
- **Correct on real Kotlin.** tree-sitter-grammars' open issues are misparses of ordinary code (identifiers that
  start with a keyword, one-line members, multiple `catch`, modifiers as hard keywords). fwcd's own cross-validation
  against PSI reports 96 of 122 clean parses matching structurally, with 118 fixtures excluded. Neither states which
  Kotlin version it covers.
- **KDoc is parsed** into sections, tags and links. **Lossless** (tokens spell the input; fuzz-checked).
- **Typed accessors with the compiler's semantics**, checked against the JVM.
- **Speed:** 31 MB/s single-threaded on the testbox (research/06). No tree-sitter-kotlin number was measured, so
  no ratio should be claimed until one is.

For tree-sitter:

- Incremental reparse and a query language, which editors are built on.
- One API for every language, and the tools on top (ast-grep, difftastic, Neovim, Helix).
- Bounded time on any input. ktrs inherits the compiler's exponential cases.
- A tree that does not change when Kotlin releases.

So the pitch is "the Kotlin compiler's parser without a JVM, for tools that need to be right about Kotlin", not
"a faster tree-sitter". Editor highlighting stays tree-sitter's.

## 6. Release, CI and tests

### release.yml

- `versions`: also check `bindings/python/Cargo.toml` and `bindings/wasm/Cargo.toml` against the tag.
- New job `python` (needs `versions`; matrix as in §3): maturin-action builds, uploads wheels as artifacts, then one
  `publish` step with `id-token: write`, gated on `vars.PYPI_PUBLISH`. Not in the `release` job's `needs`, so a
  PyPI problem cannot block the GitHub release (the npm job already works this way).
- `npm` job: add a Rust install and the Wasm build before `package-npm.mjs`; the publish loop gets one more
  directory. Still gated on `NPM_TOKEN`.
- crates.io stays manual. `kt-syntax` lives in `crates/` and goes out with
  `cargo publish --workspace --exclude ktrs-wasm --exclude xtask`; the bindings are separate workspaces and are
  never in that command.
- ci.yml: one job each for `bindings/python` (maturin develop + pytest on Linux) and `npm/syntax` (Wasm build +
  `node --test`), so `cargo test --workspace` stays as it is. Dry-run the wheel matrix on `workflow_dispatch`
  before the first tag.

### Tests

- **Rust API:** unit tests for every navigation method against the `Tree` primitives, property checks on the
  fixtures (children's ranges tile the parent's, `descendants` equals the id range, `line_col` and `offset`
  round-trip, `apply_edits` with no edits is the identity), doc tests, `cargo semver-checks`.
- **Cross-language conformance, level 1:** every binding exposes `dump()`. For each fixture in
  `crates/ktrs-parser/tests/passing.txt`, the Python and JS dumps must equal `testdata/kotlin/psi/<name>.txt`. This
  proves the parser arrived intact in each package.
- **Level 2, navigation:** a Rust example writes one line per fixture: a hash over a walk done only with the public
  API (kind name, range, parent, sibling links, token text). The same walk in Python and in JS must give the same
  hashes, checked in under `testdata/` like the psi-accessor hashes. This is what catches a wrong line in the JS
  navigation and a wrong offset conversion; include the non-ASCII fixtures for that reason.
- **Level 3, typed accessors:** the generated accessor table drives a report in each language, in the format of the
  `psi_accessors` example, hashed and compared with the Rust side (which the JVM already checks).
- **Types:** `tsc --noEmit` over the examples, `mypy`/`stubtest` over the `.pyi`.
- **Robustness:** the fuzz `parser` target extended to the facade (no panic, edits round-trip); a test that a
  panic surfaces as an exception in Python and that the Wasm glue recovers after a trap.

## 7. Phased plan

| Phase | Content | Size |
|---|---|---|
| 0 | Owner decisions (§8). Measure `Rc` to `Arc` on the fmt bench (testbox). Check the `@ktrs` npm org and token. Register the names. | S, 1 day |
| 1 (done, §10) | `crates/kt-syntax`: `SourceFile`, `Node`, iterators, `LineIndex`, errors indexed by element, `TextEdit`/`apply_edits`, CRLF/BOM entry, `#[non_exhaustive]` kinds via xtask, README, five examples (metrics, find calls, lint script, codemod, KDoc extraction), semver-checks in CI. "Internal" notes on the `ktrs-*` crates. | M, 4 to 6 days |
| 2 | Typed layer: the accessor table and its codegen, `psi` module, additions beyond ktfmt's scope (`name`, `doc_comment`, modifiers) with JVM oracle rows first. | M to L, 5 to 8 days |
| 3 | Python: PyO3 crate, generated classes and `.pyi`, pure-Python visitor/transformer, pytest conformance, wheel workflow. | M, 4 to 6 days |
| 4 | JavaScript: Wasm crate, JS navigation and glue, generated `.d.ts`, conformance, `package-npm.mjs` extension. | M, 4 to 6 days |
| 5 | Release wiring, docs site section, a benchmark against tree-sitter-kotlin for the README. | S to M, 2 to 3 days |
| 6 (later) | napi native add-on if the gate says so; `format` in the bindings; pattern matching; exposing the mutable arena. | open |

Phases 3 and 4 are independent after 1. Phase 2 can trail them: the first Python and JS releases can be untyped.

## 8. Decisions for the owner

| # | Decision | Recommendation |
|---|---|---|
| 1 | Facade crate or stabilize the existing crates | New facade crate; mark `ktrs-*` as internal. |
| 2 | Name | `kotlin-syntax` on crates.io, PyPI and npm (all free today). Check the Kotlin trademark guidelines first; fall back to `ktrs-kotlin` / `@ktrs/syntax`. Register `ktrs` on PyPI for a future CLI package. Checked: the guidelines conflict with `kotlin-syntax` (§9). **Decided: `kt-syntax`** on all three registries. |
| 3 | Versioning | Share the workspace version; breaking facade changes are minor bumps while 0.x; kinds stable by name, `#[non_exhaustive]`, pin bumps that only add kinds are minor. |
| 4 | Typed layer | Reuse `ktrs-psi` through `Arc<Tree>` if the bench cost is under 1%, otherwise a generated layer. Either way one accessor table drives Rust, `.pyi` and `.d.ts`. Measured 4.5% (§9): generated layer. |
| 5 | Mutation | Text edits only in v1. Keep `ktrs-ast` internal. |
| 6 | Offsets in bindings | Host-native by default (Python `str` indices, JS UTF-16 units) so slicing works, with `byte_range` beside it. Rust stays UTF-8 bytes. |
| 7 | JavaScript route | Wasm only for v1, flat arrays read in JS; napi later only if Wasm parse is under half of native and someone needs it. |
| 8 | Python wheels and publishing | abi3-py39, eight wheels + sdist, own workspace under `bindings/python`, root `pyproject.toml` untouched, Trusted Publishing gated on a repository variable. Free-threaded wheels on request. |

Open items that are not decisions: whether the `@ktrs` npm scope exists and why `@ktrs/cli` is absent; Wasm parse
throughput and artifact sizes. (`MissedTokens` and deep nesting: resolved in §10.)

## 9. Phase 0 results (2026-10-09, at be448f2)

Measured on the testbox under `flock ~/bench.lock taskset -c 0-4,6-10`. Nothing of phase 1 is built: the name
check below blocks it.

### Name: `kotlin-syntax` is not allowed by the guidelines as worded (decision 2 reopened)

Source: <https://kotlinfoundation.org/guidelines/> ("Kotlin brand assets usage guidelines"), section
"I. Kotlin word trademark", fetched 2026-10-09. Quotes:

- "Where identifying that a product or service is built on the Kotlin programming language or runs the Kotlin
  programming language, use the product's own name followed by "in Kotlin®", "for Kotlin®", "compatible with
  Kotlin®", "running Kotlin®" etc. Do not incorporate Kotlin into the product name."
- "Whether you're referring to a product, company or service, you shouldn't incorporate Kotlin as your brand name,
  i.e. your company cannot be called "Kotlin Consulting" or "Kotlin IDE"."
- "The Trademark may never be used in a manner that would cause confusion as to JetBrains, Google, or the Kotlin
  Foundation's sponsorship, affiliation, or endorsement, including as part of a company name, product name, domain
  name, or business trading name."
- FAQ (<https://kotlinfoundation.org/faq/>): "Any use of the Trademark other than those described in the
  Guidelines must be approved in advance."

Reading (I): the page has no carve-out for package or repository identifiers. `kotlin-syntax` has no name of its
own and leads with the mark, the shape of the "Kotlin IDE" example; it also reads as an official artifact, which is
the confusion the third quote is about. Referential text is fine ("a syntax tree library for Kotlin®").

Options:

| Option | Fit with the guidelines |
|---|---|
| A name without the mark (`kt-syntax` is free on crates.io; `ktrs-tree`, `ktrs-api` unchecked), described as "for Kotlin®" | Complies. Less findable; keywords and the description carry "kotlin". |
| `ktrs-kotlin` (the doc's fallback) | Own name first, the `tree-sitter-kotlin` / `mockito-kotlin` shape. Still "incorporates" the mark, so tolerated practice rather than compliance. |
| Keep `kotlin-syntax` and ask the Foundation's Trademark Subcommittee first | Compliant only with a written yes. |

Whatever the name, the README carries "not affiliated with or endorsed by the Kotlin Foundation or JetBrains".
The same question applies to the PyPI and npm names in §3 and §4.

### `Rc<Tree>` to `Arc<Tree>`: about 4.5% on format, so not adopted (decision 4 resolved)

Change measured: `Parse::tree` and `PsiElement::tree` as `Arc<Tree>` (`ktrs-syntax/src/lib.rs`,
`ktrs-psi/src/element.rs`), nothing else. `cargo build --release -p ktrs-{parser,fmt} --example bench`, both
binaries run interleaved on the same pinned cores, corpus of 6123 files / 30.8 MB, best of 5 reps per file.

| Bench | `Rc` (base) | `Arc` | Change |
|---|---|---|---|
| fmt `bench corpus 5`, "format = N parses", 5 rounds | 5.84, 5.84, 5.83, 5.82, 5.81 | 6.11, 6.10, 6.08, 6.07, 6.10 | +4.5% |
| same runs, CPU-s for the corpus | 4.32 (all 5) | 4.52, 4.49, 4.49, 4.49, 4.49 | +3.9% |
| parser `bench corpus 5`, `parse_file` s, 3 rounds | 0.753, 0.753, 0.754 | 0.752, 0.754, 0.753 | none |

- A first run with 1 rep while the box was loaded gave 5.65/5.75/6.02 against 5.97/6.32/5.87: too noisy to resolve
  1%, hence best-of-5.
- The gate was under 1%, so `ktrs-psi` keeps `Rc` and the typed layer is the generated one (route 2 in §2).
- The facade does not need `Arc<Tree>`: `SourceFile` owns the `Tree` by value (`Rc::into_inner(parse.tree)`, the
  parser returns the only reference) and `Node<'a>` borrows it, so `SourceFile` is `Send + Sync` and bindings wrap
  `Arc<SourceFile>` + id as planned. Only `PsiElement` interop is lost.

### Very deep nesting overflows the stack

The parser is recursive descent and aborts the process ("thread 'main' has overflowed its stack"; not a panic, so
`catch_unwind` does not help). Parser `bench` on generated one-file inputs, release build:

| Shape | 8 MB stack (Linux main thread) |
|---|---|
| nested lambdas `{ run { run {` | fine at 8000, overflow at 10000 |
| `((((1))))`, `f(f(f(1)))`, `if (a) { if (a) {`, `List<List<`, `class A { class A {`, `else if` chain | fine at 10000, overflow at 100000 |
| `a.b().b()`, `1 + 1 + 1`, `---1` | no overflow at 100000 (25 s and 35 s for the first two: quadratic) |

Nested lambdas against the stack size (`ulimit -s`): 1 MB (Windows main thread) fine at 1000, overflow at 2000;
2 MB (Rust's default for spawned threads) fine at 2000, overflow at 3000. About 0.5 to 1 KB of stack per level.

Bindings need a guard before anything else: an abort kills the Python interpreter or traps the Wasm instance.
Built in phase 1 (§10).

## 10. Phase 1 as built (2026-10-10): `crates/kt-syntax`

Owner decision after §9: the crate is `kt-syntax`, described as "a parser for Kotlin"; PyPI and npm follow.

### What differs from the sketch in §2

- `parse` and `parse_script` return `Result<SourceFile, ParseError>`. Syntax errors never fail a parse; the three
  errors are `TooDeeplyNested`, `TooLarge` (4 GiB) and `TokensNotInserted`.
- `SyntaxKind` is an opaque `Copy` struct with one associated constant per kind (generated by `cargo xtask codegen`
  into `kt-syntax/src/generated/kinds.rs`), not a `#[non_exhaustive]` enum. Constants work in patterns, a `match`
  needs a wildcard, no number is exposed, and the internal enum is not in the public API. `name`/`from_name`/`all`.
- The line index is built with the tree, not on first use: a `OnceLock` in `SourceFile` made clippy flag
  `HashSet<Node>` (`mutable_key_type`).
- Position and range queries return `Option` instead of panicking. `covering` returns `Option<Node>`.
- Added: `had_bom`/`had_crlf` (a codemod can restore what the entry normalized), `Node::comments`, `doc_comment`,
  `leading_comments`, `insert_before`/`insert_after`, `preorder` with `skip_subtree` in place of a `walk` callback.
- No `workspace.dependencies` entry: nothing in the workspace depends on the crate and cargo warns about an unused
  one. It takes `version.workspace` and goes out with `cargo publish --workspace` like the others.

### Open points resolved

- **`MissedTokens`.** The tree of such a parse leaves tokens out, so its text is not the input and every offset
  after the gap is wrong for edits. The facade returns `ParseError::TokensNotInserted { tokens }` (Display is
  IntelliJ's message). The guarantee is therefore unconditional: for every `SourceFile`, the tokens in order spell
  `text()`, which is the input minus a BOM, with `\r\n` as `\n`. No fixture and no corpus file is refused.
- **Deep nesting.** `MAX_NESTING_DEPTH = 1000` open `(`, `[`, `{`, `${`. The count is exact, on the lexer's tokens,
  with a typed stack (a closer that does not match stays open, since recovery may skip it); files with at most 1000
  opening-bracket bytes skip the lexer pass. The parser then runs under `stacker::maybe_grow` with a budget of
  `max((depth + 32) * 8 KiB, len * 256 B)`, capped at 256 MiB (64 MiB on 32-bit); debug builds use 64 KiB and
  2 KiB. It runs on the caller's stack when that much is left and on a fresh one otherwise. The per-byte term is
  for recursion that opens no bracket.
- Tested on 256 KiB threads, debug and `--profile ci` (`tests/nesting.rs`): ten bracket shapes at depth 1000 give
  the same dump as the unguarded parser on a 512 MiB stack; depth 1001 and a million `(` give the error; eleven
  bracket-free chains (3000 `else if`, 3000 `List<`, 60000 prefix operators, ..) parse.
- **Not covered:** parse time. Parenthesized function types, `((Int) -> Unit) -> Unit`, nested 1000 deep did not
  finish in 40 minutes (the test shape was dropped). This is the parser's own behaviour (research/24 finding 1),
  and the depth limit does not bound it. The per-byte stack term is an estimate checked on the shapes above, not
  a proof for every grammar path. `stacker` on Wasm is unverified (phase 4).

### Cost of the entry point

`cargo run -p kt-syntax --release --example entry_cost corpus 5` on the testbox (6123 files, 30.8 MB, 0 refused):
`kt_syntax::parse` = 1.073, 1.075, 1.073 `parse_file`s (37.5 MB/s against 40.7). About 5 points are the guard's
lexer pass and error indexing, 2 the line index. Handing the guard's tokens to the parser would remove most of it.

### Verification (testbox, 2026-10-10)

- `cargo test -p kt-syntax` and `cargo test --profile ci -p kt-syntax`: 21 unit, 16 API, 2 fixture, 4 nesting,
  3 example tests and 9 doctests (README included) pass in both.
- `tests/fixtures.rs`: the facade's dump, written against its public navigation only, equals `psi_dump` on every
  fixture and the compiler's `.txt` for every entry of ktrs-parser's `passing.txt`; tree invariants hold on all.
- `cargo +stable clippy -p kt-syntax --all-targets` and `RUSTDOCFLAGS="-D warnings" cargo doc -p kt-syntax`: clean.
- `cargo publish --dry-run -p ktrs-syntax -p ktrs-lexer -p ktrs-parser -p kt-syntax`: packages and verifies.
- Examples over the corpus: `missing_kdoc` 20044 findings, `rename_call println printLine` 735 calls.
- Not run: Windows and macOS (CI will), `cargo semver-checks` (nothing released to compare with), the fuzz target.
