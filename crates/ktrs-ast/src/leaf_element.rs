//! Port of IntelliJ `LeafElement`'s edit.

use crate::arena::{Ast, NodeId};

impl Ast {
    /// `rawReplaceWithText(newText)`: swaps in a **new** leaf of the same type and returns it; `this`
    /// is left with no parent (the engine's "node was replaced" bail-out keys on that).
    pub fn raw_replace_with_text(&mut self, this: NodeId, new_text: &str) -> NodeId {
        assert!(self.is_leaf_element(this), "rawReplaceWithText on a composite");
        let new_leaf = self.new_leaf(self.element_type(this), new_text);
        self.raw_replace_with_list(this, Some(new_leaf));
        new_leaf
    }
}
