//! Typed PSI over the mutable AST: the `Kt*` classes and accessors ktlint's standard rules and engine reach
//! through `node.psi`, with the compiler's semantics (Kotlin 2.4 psi-api; the ALPHA-4 jar's 2.4.10 bytecode
//! where it differs). Membership tables and value types (`FqName`, `ImportPath`) come from `ktrs_psi`,
//! which has the same classes over the immutable tree.
//!
//! # Porting conventions
//! - A class is a `Copy` newtype over [`NodeId`]: `node.psi is KtFoo` -> `KtFoo::is(ast, node)`,
//!   `node.psi as? KtFoo` -> `KtFoo::cast(ast, node)`, `node.psi as KtFoo` -> `KtFoo::of(ast, node)`
//!   (panics `ClassCastException`), `psi.node` -> `.node()`.
//! - Accessors take the arena: `(node.psi as KtImportDirective).importPath` ->
//!   `KtImportDirective::of(ast, node).import_path(ast)`. Getters drop `get`, as in `ktrs_psi`.
//! - `psi.delete()` -> [`delete`]; `KtElement.rawDelete()` (ktlint master) -> [`raw_delete`].
//! - `new KtBlockExpression(null)` -> `ast.new_composite(BLOCK)`; `PsiWhiteSpaceImpl(t)` / `LeafPsiElement(k, t)` /
//!   `PsiCommentImpl(k, t)` -> `ast.new_leaf(k, t)`.

mod classes;
mod delete;
mod directives;
mod elements;

pub use classes::*;
pub use delete::{delete, raw_delete};
/// Value types, token tables and helpers shared with `ktrs_psi` (`KtTokenSets.DECLARATION_TYPES`,
/// `KtSingleValueToken.value`, `KtPsiUtil.unquoteIdentifier`, `AnnotationUseSiteTarget.renderName`).
pub use ktrs_psi::{
    AnnotationUseSiteTarget, DECLARATION_TYPES, FqName, ImportPath, KtSingleValueToken, render_name, single_value,
    unquote_identifier,
};

use ktrs_psi::classes::is_modifier_list_owner;
use ktrs_syntax::SyntaxKind::{self, *};

use crate::arena::{Ast, NodeId};

/// ktlint `ASTNode.isKtAnnotated`: `dummyPsiElement() is KtAnnotated`, decided by the element type alone.
pub fn is_kt_annotated(ast: &Ast, node: NodeId) -> bool {
    let kind = dummy_psi_element_type(ast, node);
    is_modifier_list_owner(kind) || matches!(kind, ANNOTATED_EXPRESSION | TYPE_CONSTRAINT | FILE)
}

/// The element type `dummyPsiElement()` builds its cached PSI from. Panics `NotImplementedError` for types
/// that are neither `KtToken`s nor `KtNodeType`/`KtStubElementType`/`KtFileElementType`, as upstream.
// Gotcha: in the jar's Kotlin 2.4.10, BLOCK and LAMBDA_EXPRESSION are lazy-parseable types, so they throw
// (checked on the JVM over corpus/ktor); 2.4.20 made them stub types.
fn dummy_psi_element_type(ast: &Ast, node: NodeId) -> SyntaxKind {
    let kind = ast.element_type(node);
    assert!(
        !matches!(
            kind,
            WHITE_SPACE
                | ERROR_ELEMENT
                | BAD_CHARACTER
                | DUMMY_HOLDER
                | BLOCK
                | LAMBDA_EXPRESSION
                | DOC_COMMENT
                | KDOC_SECTION
                | KDOC_TAG
                | KDOC_NAME
                | KDOC_MARKDOWN_LINK
        ),
        "NotImplementedError: Cannot create dummy psi for {kind:?}"
    );
    kind
}

/// `ASTNode.toString()` as the jar prints it (it shows in rule exception messages): `TreeElement`'s `Element(<type>)`
/// for composites, but the PSI's own `toString` where node and PSI are one object — the lazy-parseable `BLOCK`,
/// `LAMBDA_EXPRESSION` and `DOC_COMMENT` (their Kt classes print the bare type) and the leaves.
pub fn ast_node_to_string(ast: &Ast, node: NodeId) -> String {
    element_to_string(ast.element_type(node), ast.is_leaf_element(node))
}

/// [`ast_node_to_string`] from what it depends on, for callers that must render a node after losing the arena.
pub fn element_to_string(kind: SyntaxKind, is_leaf: bool) -> String {
    let name = kind.debug_name();
    match kind {
        WHITE_SPACE => "PsiWhiteSpace".to_owned(),
        EOL_COMMENT | BLOCK_COMMENT | SHEBANG_COMMENT => format!("PsiComment({name})"),
        BLOCK | LAMBDA_EXPRESSION | DOC_COMMENT => name.to_owned(),
        _ if is_leaf => format!("PsiElement({name})"),
        _ => format!("Element({name})"),
    }
}

/// `PsiElement.getParent()`: the tree parent (`SharedImplUtil.getParent`); `None` for the file root.
pub fn psi_parent(ast: &Ast, node: NodeId) -> Option<NodeId> {
    ast.tree_parent(node)
}

/// `element instanceof PsiFile`: the file root or a dummy holder.
pub fn is_psi_file(ast: &Ast, node: NodeId) -> bool {
    ast.is_file_element(node)
}
