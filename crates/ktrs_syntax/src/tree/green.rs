//! Bridge from rowan while consumers migrate: replays a green subtree into a [`TreeBuilder`].
//! TODO: delete with the `rowan` dependency (research/06-tree-library.md, step 4).

use rowan::{GreenNodeData, NodeOrToken};

use super::TreeBuilder;
use crate::SyntaxKind;

impl TreeBuilder {
    pub fn push_green(&mut self, node: &GreenNodeData) {
        self.start_node(SyntaxKind::from_raw(node.kind().0));
        for child in node.children() {
            match child {
                NodeOrToken::Node(n) => self.push_green(n),
                NodeOrToken::Token(t) => self.token(SyntaxKind::from_raw(t.kind().0), t.text()),
            }
        }
        self.finish_node();
    }
}
