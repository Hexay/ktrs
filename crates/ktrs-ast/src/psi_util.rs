//! The compiler's psiUtil `ASTNode` extensions (psi-api/.../psiUtil/psiUtils.kt) other than `leaves`
//! (`tree_util.rs`).

use crate::arena::{Ast, NodeId};

impl Ast {
    /// psiUtil `ASTNode.children()`; ktlint's `children` property is the same walk.
    pub fn children(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(self.first_child_node(node), |&n| self.tree_next(n))
    }

    /// psiUtil `ASTNode.parents()`: `treeParent`, its parent, and so on (not `node` itself).
    pub fn parents(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(self.tree_parent(node), |&n| self.tree_parent(n))
    }

    /// psiUtil `ASTNode.siblings(forward)`: the following (or preceding) siblings, nearest first.
    pub fn siblings(&self, node: NodeId, forward: bool) -> impl Iterator<Item = NodeId> + '_ {
        let step = move |n: NodeId| if forward { self.tree_next(n) } else { self.tree_prev(n) };
        std::iter::successors(step(node), move |&n| step(n))
    }
}
