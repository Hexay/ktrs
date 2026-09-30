//! A mutable Kotlin AST with the semantics of IntelliJ's `TreeElement`/`CompositeElement`/`LeafElement`
//! (the platform classes inside kotlin-compiler-embeddable 2.4.20), for porting ktlint rules 1:1.
//! Design and the divergences it has to reproduce: research/11-ktlint-engine.md §2-§4; which upstream
//! symbol maps to which Rust name: research/16-ktlint-api-coverage.md.
//!
//! # Model
//! - [`Ast`] is an arena; a node is a [`NodeId`] that is never reused, so a rule can keep one across
//!   edits exactly like an `ASTNode` reference, including after the node is detached.
//! - [`Ast::from_parse`] seeds it from a `ktrs_syntax::Tree` in one preorder pass. The root gets the
//!   file element type `FILE` (`KtFileElementType`, what ktlint's `ElementType.FILE` is), not the
//!   parser's `KT_FILE`.
//! - Removed nodes land in a fresh `DUMMY_HOLDER` (IntelliJ's `repairRemovedElement`), so their
//!   `tree_parent` is not null; only `raw_*` removals, replaced leaves and deleted leaves end up with no parent.
//!
//! # Porting conventions (Java/Kotlin -> Rust)
//! - `node.foo()` / `node.foo` on an `ASTNode` -> `ast.foo(node)`: `getTreeParent()` -> `tree_parent`,
//!   `getFirstChildNode()` -> `first_child_node`, `getStartOffset()` -> `start_offset`, and so on.
//!   Nullable results are `Option<NodeId>`. Methods keep the order of their Java file.
//! - Overloads with a trailing nullable parameter take an `Option`: `addChild(child)` ->
//!   `add_child(parent, child, None)`. `getChildren(null)` -> `get_children`, `getChildren(set)` ->
//!   `get_children_filtered`.
//! - Java statics are free functions in a module named after the class: `TreeUtil.nextLeaf(n)` ->
//!   `tree_util::next_leaf(ast, n)`, `CodeEditUtil.removeChild(p, c)` -> `code_edit_util::remove_child`.
//!   psiUtil `ASTNode` extensions (`children`, `parents`, `siblings`, `leaves`) are `Ast` methods.
//! - `node is LeafElement` -> [`Ast::is_leaf_element`]; `node is CompositeElement` -> its negation;
//!   `node is FileASTNode` -> [`Ast::is_file_element`].
//! - Offsets and lengths are UTF-8 bytes; [`Ast::utf16_offset`] converts for ktlint's line table, and
//!   [`Ast::text_length_utf16`] is the JVM's `textLength`/`String.length` where a count is compared or shown.
//! - `new LeafPsiElement(type, text)`, `new PsiWhiteSpaceImpl(text)`, `ASTFactory.leaf` ->
//!   [`Ast::new_leaf`]; `new KtBlockExpression(null)` -> [`Ast::new_composite`]. The PSI class of a
//!   leaf follows from its type, which is all `DebugUtil.psiToString` looks at for these callers.
//! - `node.psi` and the `Kt*` classes: [`psi`] (its docs have the conventions).

mod arena;
pub mod code_edit_util;
mod composite_element;
mod dump;
mod leaf_element;
pub mod psi;
mod psi_util;
mod snippet;
mod text;
mod tree_element;
pub mod tree_util;

pub use arena::{Ast, NodeId};
pub use text::Preorder;
pub use tree_util::Leaves;
