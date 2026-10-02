//! Port of `core/util/ASTNodes.kt`.

use ktrs_ast::psi::PsiComment;
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::WHITE_SPACE;

pub fn last_child_leaf_or_self(ast: &Ast, node: NodeId) -> NodeId {
    let mut node = node;
    while let Some(last) = ast.last_child_node(node) {
        node = last;
    }
    node
}

pub fn first_child_leaf_or_self(ast: &Ast, node: NodeId) -> NodeId {
    let mut node = node;
    while let Some(first) = ast.first_child_node(node) {
        node = first;
    }
    node
}

/// `ASTNode.parent(p, strict)`.
pub fn parent(ast: &Ast, node: NodeId, p: impl Fn(&Ast, NodeId) -> bool, strict: bool) -> Option<NodeId> {
    let mut n = if strict { ast.tree_parent(node) } else { Some(node) };
    while let Some(current) = n {
        if p(ast, current) {
            return Some(current);
        }
        n = ast.tree_parent(current);
    }
    None
}

pub fn is_part_of_comment(ast: &Ast, node: NodeId) -> bool {
    parent(ast, node, PsiComment::is, false).is_some()
}

pub fn next_code_sibling(ast: &Ast, node: NodeId) -> Option<NodeId> {
    next_sibling(ast, node, |ast, it| ast.element_type(it) != WHITE_SPACE && !is_part_of_comment(ast, it))
}

pub fn next_sibling(ast: &Ast, node: NodeId, p: impl Fn(&Ast, NodeId) -> bool) -> Option<NodeId> {
    ast.siblings(node, true).find(|&it| p(ast, it))
}
