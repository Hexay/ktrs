# kt-syntax

A parser for Kotlin, in Rust, with no JVM. It is a port of the Kotlin compiler's own parser, and the tree
it builds is the compiler's PSI tree: same node kinds, same nesting, same error elements, checked against
the compiler's parser test fixtures and thousands of real files.

- **Lossless.** Every character is in the tree, white space and comments included, so tokens spell the
  source and edits can be made by range.
- **Error tolerant.** Broken code still gives a full tree, with the compiler's error messages.
- **KDoc is parsed** into sections, tags and links.
- **Fast and thread friendly.** A file is five flat arrays; a node is a `Copy` handle of two words;
  `SourceFile` is `Send + Sync`.

This is an independent project. It is not affiliated with or endorsed by the Kotlin Foundation or
JetBrains. Kotlin® is a trademark of the Kotlin Foundation.

## Parse and navigate

```rust
use kt_syntax::{SyntaxKind, parse};

let file = parse("fun main() {\n    println(\"hi\")\n}\n").unwrap();
assert!(!file.has_errors());

let function = file.root().find_all(&[SyntaxKind::FUN]).next().unwrap();
let name = function.child(SyntaxKind::IDENTIFIER).unwrap();
assert_eq!(name.text(), "main");
assert_eq!(name.range(), 4..8);

// Offsets are UTF-8 bytes; lines and columns are zero-based.
let at = file.line_col(name.range().start).unwrap();
assert_eq!((at.line, at.col), (0, 4));

// Up, down and sideways.
assert_eq!(name.parent(), Some(function));
assert_eq!(name.prev_sibling().unwrap().kind(), SyntaxKind::WHITE_SPACE);
assert!(name.ancestors().any(|node| node.kind() == SyntaxKind::KT_FILE));
let call = file.token_at(17).unwrap(); // the token under a cursor
assert_eq!((call.kind(), call.text()), (SyntaxKind::IDENTIFIER, "println"));
```

A [`Node`] is a node or a token, as in the compiler's PSI; `is_token()` tells them apart. `children()`
yields both, `child_nodes()` only composite nodes, `tokens()` only tokens. `descendants()` and
`find_all()` walk a subtree in source order, `preorder()` reports enter and leave and can skip subtrees.

## Kinds

Kinds have the compiler's names: `KtNodeTypes.FUN` is `SyntaxKind::FUN`, `KtTokens.FUN_KEYWORD` is
`SyntaxKind::FUN_KEYWORD`. If you know a node from IntelliJ's PSI viewer, detekt or ktlint, it has the
same name here. `SourceFile::dump()` prints a tree in the compiler's test format, which is the quickest
way to learn the shape of a construct:

```rust
let file = kt_syntax::parse("val x = 1").unwrap();
assert!(file.dump().contains("PROPERTY\n    PsiElement(val)('val')"));
```

## Syntax errors

```rust
let file = kt_syntax::parse("fun broken( {}\n").unwrap();
assert!(file.has_errors());
for error in file.errors() {
    assert!(error.node().is_error());
    println!("{error}"); // line:column: the compiler's message
}
// The tree still covers the whole text.
let spelled: String = file.root().tokens().map(|token| token.text()).collect();
assert_eq!(spelled, file.text());
```

## Comments and KDoc

Comments are tokens in the tree; a KDoc comment is a node with its sections and tags. Kotlin attaches the
comments before a declaration to that declaration.

```rust
use kt_syntax::{SyntaxKind, parse};

let file = parse("/**\n * Adds.\n * @return the sum\n */\nfun add() = 1\n\n// plain\nfun sub() = 2\n").unwrap();
let undocumented: Vec<&str> = file
    .root()
    .find_all(&[SyntaxKind::FUN])
    .filter(|function| function.doc_comment().is_none())
    .filter_map(|function| function.child(SyntaxKind::IDENTIFIER))
    .map(|name| name.text())
    .collect();
assert_eq!(undocumented, ["sub"]);

let kdoc = file.root().find_all(&[SyntaxKind::FUN]).next().unwrap().doc_comment().unwrap();
let tag = kdoc.find_all(&[SyntaxKind::KDOC_TAG]).next().unwrap();
assert!(tag.text().starts_with("@return"));
assert_eq!(file.root().comments().count(), 2);
```

## Edit

Edits are text edits. Collect them from nodes or ranges, apply them all at once and get a new string;
overlapping edits are rejected. Parse the result again to continue.

```rust
use kt_syntax::{SyntaxKind, parse};

let file = parse("val a = foo()\nval b = foo(1)\n").unwrap();
let edits = file
    .root()
    .find_all(&[SyntaxKind::CALL_EXPRESSION])
    .filter_map(|call| call.first_child())
    .filter(|callee| callee.text() == "foo")
    .map(|callee| callee.replace("bar"));
assert_eq!(file.apply_edits(edits).unwrap(), "val a = bar()\nval b = bar(1)\n");
```

## Input, offsets and limits

- `parse` drops a leading byte order mark and turns `\r\n` into `\n`, like the compiler's tooling. All
  offsets refer to `SourceFile::text()`, the normalized text. `had_bom()` and `had_crlf()` say what was
  there, for tools that write files back.
- Offsets and ranges are UTF-8 bytes. `line_col_utf16` and `offset_utf16` convert for editors.
- `parse` returns `Err` in three cases, none of them a syntax error: brackets nested deeper than
  `MAX_NESTING_DEPTH` (1000); a text of 4 GiB or more; and malformed input for which Kotlin's parser
  itself leaves tokens out of the tree (IntelliJ logs "Tokens [..] were not inserted into the tree"), since
  such a tree would not spell its text.
- The parser is recursive. `parse` runs it with a stack budget sized to the input, so deep nesting gives
  an error or a tree, not a stack overflow, whatever thread it is called from.
- Like the compiler's parser, it takes exponential time on some pathological nestings of unclosed
  parentheses. Don't parse untrusted input without a time limit.

## Stability

- This crate follows semver. While it is 0.x, a minor release may break the API.
- The `ktrs-*` crates it is built on are internals of [ktrs](https://github.com/Hexay/ktrs) and may change
  in any release. Depend on `kt-syntax`.
- `KOTLIN_VERSION` is the compiler version the parser is a port of. The tree for a given text is what that
  compiler builds, and it can change when the version does.
- Kinds are identified by name. `SyntaxKind` is opaque, so a newer Kotlin can add kinds in a minor
  release: match with a wildcard arm and store `name()`s, not numbers.

[`Node`]: https://docs.rs/kt-syntax/latest/kt_syntax/struct.Node.html
