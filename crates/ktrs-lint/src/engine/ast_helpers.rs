//! The `ASTNodeExtension.kt` helpers and PSI class checks the engine needs that `ast_node_extension.rs`
//! does not have yet. A PSI check on a node's type uses ktlint's `dummyPsiElement` view (composite
//! PSI class of the type); [`psi_kind`] is the real `node.psi` view (a leaf is never a `KtElement`).
// TODO: move to ast_node_extension (1B): recursive_children, find_child_by_type_recursively, replace_with,
// indent_without_newline_prefix, is_root, is_kt_annotated, create_psi_file_from_text.

use ktrs_ast::{Ast, NodeId, tree_util};
use ktrs_syntax::SyntaxKind::{self, *};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;

/// `recursiveChildren`: the descendants in preorder, without the node itself.
pub(crate) fn recursive_children(ast: &Ast, n: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    let mut stack: Vec<NodeId> = ast.children(n).collect();
    stack.reverse();
    while let Some(node) = stack.pop() {
        out.push(node);
        let first = stack.len();
        stack.extend(ast.children(node));
        stack[first..].reverse();
    }
    out
}

/// `findChildByTypeRecursively(elementType)`.
pub(crate) fn find_child_by_type_recursively(
    ast: &Ast,
    n: NodeId,
    element_type: SyntaxKind,
) -> Option<NodeId> {
    recursive_children(ast, n)
        .into_iter()
        .find(|&it| ast.element_type(it) == element_type)
}

/// `replaceWith(node)`: `parent?.addChild(node, this)`, then `this.remove()`.
pub(crate) fn replace_with(ast: &mut Ast, this: NodeId, node: NodeId) {
    if let Some(parent) = ast.parent(this) {
        ast.add_child(parent, node, Some(this));
    }
    ast.remove(this);
}

/// `indentWithoutNewlinePrefix`.
pub(crate) fn indent_without_newline_prefix(ast: &Ast, n: NodeId) -> String {
    let indent = ast.indent(n);
    indent.strip_prefix('\n').unwrap_or(&indent).to_owned()
}

/// `isRoot`.
pub(crate) fn is_root(ast: &Ast, n: NodeId) -> bool {
    ast.element_type(n) == FILE
}

/// `textRange`: `[startOffset, startOffset + textLength)`, UTF-8 offsets of the current tree.
pub(crate) fn text_range(ast: &Ast, n: NodeId) -> (usize, usize) {
    let start = ast.start_offset(n);
    (start, start + ast.text_length(n))
}

/// `isKtAnnotated`: the PSI class of the type implements `KtAnnotated`.
pub(crate) fn is_kt_annotated(kind: SyntaxKind) -> bool {
    ktrs_psi::is_modifier_list_owner(kind)
        || matches!(kind, ANNOTATED_EXPRESSION | FILE | TYPE_CONSTRAINT)
}

/// The type whose PSI class `node.psi` has; `None` for a leaf (`LeafPsiElement`, `PsiWhiteSpace`, ...).
pub(crate) fn psi_kind(ast: &Ast, n: NodeId) -> Option<SyntaxKind> {
    (!ast.is_leaf_element(n)).then(|| ast.element_type(n))
}

pub(crate) fn psi_is_kt_expression(ast: &Ast, n: NodeId) -> bool {
    psi_kind(ast, n).is_some_and(ktrs_psi::is_expression)
}

pub(crate) fn psi_is_kt_declaration(ast: &Ast, n: NodeId) -> bool {
    psi_kind(ast, n).is_some_and(ktrs_psi::is_declaration)
}

/// `psi.parent`; the file's parent (a directory) is not in the tree.
pub(crate) fn psi_parent(ast: &Ast, n: NodeId) -> Option<NodeId> {
    if is_root(ast, n) { None } else { ast.parent(n) }
}

/// `KtlintKotlinCompiler.createPsiFileFromText(fileName, text).firstChild?.node`, for a `.kt` file name.
/// Parsed as a script (the only snippet parse the arena has): the leading file annotations are the same.
pub(crate) fn create_psi_file_first_child_from_text(ast: &mut Ast, text: &str) -> Option<NodeId> {
    let node = ast.create_ast_node_from_text(text)?;
    let file = tree_util::get_file_element(ast, node)?;
    ast.first_child_node(file)
}
