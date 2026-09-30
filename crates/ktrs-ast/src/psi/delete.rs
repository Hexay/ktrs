//! `PsiElement.delete()` as the ALPHA-4 jar (Kotlin 2.4.10) runs it; every path ends in
//! `CodeEditUtil.removeChild(parent, child)` (`CompositeElement.deleteChildInternal`).

use ktrs_syntax::SyntaxKind::*;

use super::classes::{KtElement, KtEnumEntry, PsiComment, PsiWhiteSpace};
use super::psi_parent;
use crate::arena::{Ast, NodeId};
use crate::code_edit_util;

/// `psi.delete()`:
/// - a leaf (`LeafPsiElement.delete`): `getTreeParent().deleteChildInternal(this)`, then `invalidate()`,
///   so the leaf ends up with no parent although its dummy holder still lists it;
/// - a Kt element (`KtElementImpl`/`KtElementImplStub.delete`): `deleteSemicolon()`, then
///   `ASTDelegatePsiElement.delete` ([`raw_delete`]);
/// - any other composite (KDoc, error element): [`raw_delete`].
pub fn delete(ast: &mut Ast, element: NodeId) {
    if ast.is_leaf_element(element) {
        let parent = ast.tree_parent(element).expect("AssertionError: LeafPsiElement.delete of a detached leaf");
        code_edit_util::remove_child(ast, parent, element);
        ast.invalidate(element);
        return;
    }
    if KtElement::is(ast, element) {
        delete_semicolon(ast, element);
    }
    raw_delete(ast, element);
}

/// `KtElement.rawDelete()` = `super.delete()` = `ASTDelegatePsiElement.deleteElementFromParent(this)`.
pub fn raw_delete(ast: &mut Ast, element: NodeId) {
    let parent = psi_parent(ast, element)
        .unwrap_or_else(|| panic!("UnsupportedOperationException: {:?} under null", ast.element_type(element)));
    delete_child_internal(ast, parent, element);
}

/// `deleteChildInternal(child)` of the parent's PSI: `CodeEditUtil.removeChild`, plus `KtModifierList`'s
/// override that deletes the list once it is empty. (A `PsiFile` parent's `deleteChildRange(e, e)` is the
/// same `removeChildren`.)
fn delete_child_internal(ast: &mut Ast, parent: NodeId, child: NodeId) {
    code_edit_util::remove_child(ast, parent, child);
    if ast.element_type(parent) == MODIFIER_LIST && !ast.is_leaf_element(parent) && ast.first_child_node(parent).is_none() {
        delete(ast, parent);
    }
}

/// ktElementUtils `KtElement.deleteSemicolon()`: also removes a `;` after the element (skipping whitespace
/// and comments), with the whitespace that follows it.
fn delete_semicolon(ast: &mut Ast, element: NodeId) {
    if KtEnumEntry::is(ast, element) {
        return;
    }
    let sibling = skip_siblings_forward(ast, element, |ast, e| PsiWhiteSpace::is(ast, e) || PsiComment::is(ast, e));
    let Some(sibling) = sibling.filter(|&s| ast.element_type(s) == SEMICOLON) else { return };
    let last_sibling_to_delete = skip_siblings_forward(ast, sibling, PsiWhiteSpace::is)
        .and_then(|s| ast.tree_prev(s))
        .unwrap_or(sibling);
    let parent = psi_parent(ast, element).expect("NullPointerException: deleteSemicolon without a parent");
    let first = ast.tree_next(element).expect("the semicolon follows the element");
    code_edit_util::remove_children(ast, parent, first, last_sibling_to_delete);
}

/// `PsiTreeUtil.skipSiblingsForward(element, classes)`: the first next sibling that is none of `classes`.
fn skip_siblings_forward(ast: &Ast, element: NodeId, is_skipped: impl Fn(&Ast, NodeId) -> bool) -> Option<NodeId> {
    ast.siblings(element, true).find(|&e| !is_skipped(ast, e))
}
