//! Port of `core/util/PsiElements.kt`.

use std::collections::VecDeque;
use std::ops::RangeInclusive;

use ktrs_ast::psi::{self, PsiClass};
use ktrs_ast::{Ast, NodeId};

/// `findAllChildrenByClass<T>(shouldVisitChildren)`: `element` and its descendants that are `T`, breadth first
/// over `PsiElement.getChildren()`; the children of a node are queued only when `should_visit_children` holds.
/// Collected eagerly; when the consumer edits the tree between items, step a [`ChildrenByClass`] instead.
pub fn find_all_children_by_class<T: PsiClass>(
    ast: &Ast,
    element: NodeId,
    should_visit_children: impl Fn(&Ast, NodeId) -> bool,
) -> Vec<T> {
    let mut cursor = ChildrenByClass::new(element);
    std::iter::from_fn(|| cursor.next(ast, &should_visit_children)).collect()
}

/// [`find_all_children_by_class`] with every subtree visited (`PsiElementAlwaysTruePredicate`).
pub fn find_all_children<T: PsiClass>(ast: &Ast, element: NodeId) -> Vec<T> {
    find_all_children_by_class(ast, element, |_, _| true)
}

/// The lazy `sequence {}` of `findAllChildrenByClass`: each [`ChildrenByClass::next`] reads the tree as it is then
/// (the children of a yielded element are listed when the walk resumes, after the consumer handled it).
pub struct ChildrenByClass {
    queue: VecDeque<NodeId>,
    yielded: Option<NodeId>,
}

impl ChildrenByClass {
    pub fn new(element: NodeId) -> ChildrenByClass {
        ChildrenByClass { queue: VecDeque::from([element]), yielded: None }
    }

    pub fn next<T: PsiClass>(&mut self, ast: &Ast, should_visit_children: impl Fn(&Ast, NodeId) -> bool) -> Option<T> {
        if let Some(previous) = self.yielded.take() {
            self.enqueue_children(ast, previous, &should_visit_children);
        }
        while let Some(current) = self.queue.pop_front() {
            if let Some(found) = T::cast(ast, current) {
                self.yielded = Some(current);
                return Some(found);
            }
            self.enqueue_children(ast, current, &should_visit_children);
        }
        None
    }

    fn enqueue_children(&mut self, ast: &Ast, current: NodeId, should_visit_children: &impl Fn(&Ast, NodeId) -> bool) {
        if should_visit_children(ast, current) {
            self.queue.extend(psi::children(ast, current));
        }
    }
}

pub fn find_direct_first_child_by_class<T: PsiClass>(ast: &Ast, element: NodeId) -> Option<T> {
    ast.children(element).find_map(|c| T::cast(ast, c))
}

pub fn find_direct_children_by_class<T: PsiClass>(ast: &Ast, element: NodeId) -> Vec<T> {
    ast.children(element).filter_map(|c| T::cast(ast, c)).collect()
}

/// `walkBackwards(stopAtParent)`: from `element` backwards through its siblings, then each ancestor and its
/// previous siblings, up to (excluding) `stop_at_parent`.
pub fn walk_backwards(ast: &Ast, element: NodeId, stop_at_parent: Option<NodeId>) -> Vec<NodeId> {
    let mut out = Vec::new();
    for ancestor in ast.parents_with_self(element) {
        for sibling in ast.siblings_with_itself(ancestor, false, true) {
            if Some(sibling) == stop_at_parent {
                return out;
            }
            out.push(sibling);
        }
    }
    out
}

/// `PsiNameIdentifierOwner.startOffsetFromName`.
pub fn start_offset_from_name(ast: &Ast, owner: NodeId) -> usize {
    psi::name_identifier(ast, owner).map_or_else(|| ast.start_offset(owner), |n| ast.start_offset(n))
}

/// `PsiElement.range`: `IntRange(startOffset, endOffset)`, end included.
pub fn range(ast: &Ast, element: NodeId) -> RangeInclusive<usize> {
    ast.start_offset(element)..=ast.end_offset(element)
}
