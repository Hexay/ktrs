//! Port of the IntelliJ `TreeUtil` statics ktlint reaches (free functions: `TreeUtil.nextLeaf(node)` ->
//! `tree_util::next_leaf(ast, node)`, so they can't be mistaken for ktlint's own `nextLeaf`), and of the
//! compiler's psiUtil `ASTNode.leaves`. These walks only ever return `LeafElement`s.

use crate::arena::{Ast, NodeId};

/// `TreeUtil.findLastLeaf(element)`.
pub fn find_last_leaf(ast: &Ast, element: NodeId) -> Option<NodeId> {
    if ast.is_leaf_element(element) {
        return Some(element);
    }
    let mut child = ast.last_child_node(element);
    while let Some(c) = child {
        if let Some(leaf) = find_last_leaf(ast, c) {
            return Some(leaf);
        }
        child = ast.tree_prev(c);
    }
    None
}

/// `TreeUtil.getFileElement(element)`: the enclosing file or dummy holder, `element` included.
pub fn get_file_element(ast: &Ast, element: NodeId) -> Option<NodeId> {
    let mut parent = Some(element);
    while let Some(p) = parent {
        if ast.is_file_element(p) {
            return Some(p);
        }
        parent = ast.tree_parent(p);
    }
    None
}

/// `TreeUtil.nextLeaf(node)`.
pub fn next_leaf(ast: &Ast, start: NodeId) -> Option<NodeId> {
    let mut element = Some(start);
    while let Some(e) = element {
        let mut next_tree = ast.tree_next(e);
        while let Some(t) = next_tree {
            if let Some(next) = find_first_leaf_or_type(ast, t) {
                return Some(next);
            }
            next_tree = ast.tree_next(t);
        }
        element = ast.tree_parent(e);
    }
    None
}

fn find_first_leaf_or_type(ast: &Ast, element: NodeId) -> Option<NodeId> {
    if ast.is_leaf_element(element) {
        return Some(element);
    }
    let mut child = ast.first_child_node(element);
    while let Some(c) = child {
        if let Some(leaf) = find_first_leaf_or_type(ast, c) {
            return Some(leaf);
        }
        child = ast.tree_next(c);
    }
    None
}

/// `TreeUtil.prevLeaf(node)`.
pub fn prev_leaf(ast: &Ast, start: NodeId) -> Option<NodeId> {
    let mut start = Some(start);
    while let Some(s) = start {
        let mut prev_tree = ast.tree_prev(s);
        while let Some(t) = prev_tree {
            if let Some(prev) = find_last_leaf(ast, t) {
                return Some(prev);
            }
            prev_tree = ast.tree_prev(t);
        }
        start = ast.tree_parent(s);
    }
    None
}

impl Ast {
    /// psiUtil `ASTNode.leaves(forward)`: the leaves after (or before) `node`, nearest first.
    pub fn leaves(&self, node: NodeId, forward: bool) -> Leaves<'_> {
        let next = if forward { next_leaf(self, node) } else { prev_leaf(self, node) };
        Leaves { ast: self, next, forward }
    }
}

pub struct Leaves<'a> {
    ast: &'a Ast,
    next: Option<NodeId>,
    forward: bool,
}

impl Iterator for Leaves<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let cur = self.next?;
        self.next = if self.forward { next_leaf(self.ast, cur) } else { prev_leaf(self.ast, cur) };
        Some(cur)
    }
}
