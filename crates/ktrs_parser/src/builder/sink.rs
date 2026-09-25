//! [`TreeSink`]: the green-tree builder `tree.rs` binds into, shared by a file and all of its
//! chameleons so each lazy node is built in place instead of being copied out of a sub-tree.

use ktrs_syntax::{GreenNode, Parse, SyntaxKind};
use rowan::{GreenToken, NodeOrToken};

use super::interner::{Interner, raw};

type GreenElement = NodeOrToken<GreenNode, GreenToken>;

pub struct TreeSink {
    children: Vec<GreenElement>,
    parents: Vec<(SyntaxKind, usize)>,
    /// `None` only once handed back in `drop`.
    interner: Option<Interner>,
    pub(super) errors: Vec<String>,
}

impl Default for TreeSink {
    fn default() -> TreeSink {
        TreeSink::new()
    }
}

impl TreeSink {
    pub fn new() -> TreeSink {
        TreeSink { children: Vec::new(), parents: Vec::new(), interner: Some(Interner::take()), errors: Vec::new() }
    }

    pub fn finish(mut self) -> Parse {
        assert!(self.parents.is_empty() && self.children.len() == 1, "unbalanced tree sink");
        let green = self.children.pop().and_then(NodeOrToken::into_node).expect("root is a node");
        Parse { green, error_messages: std::mem::take(&mut self.errors) }
    }

    pub(super) fn token(&mut self, kind: SyntaxKind, text: &str) {
        let token = self.interner.as_mut().expect("live sink").token(raw(kind), text);
        self.children.push(token.into());
    }

    pub(super) fn start_node(&mut self, kind: SyntaxKind) {
        self.parents.push((kind, self.children.len()));
    }

    pub(super) fn finish_node(&mut self) {
        let (kind, first_child) = self.parents.pop().expect("finish_node without start_node");
        let node = GreenNode::new(raw(kind), self.children.drain(first_child..));
        self.children.push(node.into());
    }
}

impl Drop for TreeSink {
    fn drop(&mut self) {
        if let Some(interner) = self.interner.take() {
            interner.give_back();
        }
    }
}
