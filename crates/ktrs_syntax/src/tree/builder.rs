//! [`TreeBuilder`]: start/token/finish events in, a [`Tree`] out. Every element is a push onto
//! each array; no per-node allocation.

use super::{ElementId, NONE, TOKEN_BIT, Tree};
use crate::SyntaxKind;

#[derive(Default)]
pub struct TreeBuilder {
    text: String,
    kinds: Vec<u16>,
    starts: Vec<u32>,
    ends: Vec<u32>,
    parents: Vec<u32>,
    prev_sibs: Vec<u32>,
    /// Open nodes, innermost last, each with its most recent child so far.
    open: Vec<(ElementId, u32)>,
}

impl TreeBuilder {
    pub fn new() -> TreeBuilder {
        TreeBuilder::default()
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        let e = self.push(kind as u16, NONE);
        self.open.push((e, NONE));
    }

    pub fn token(&mut self, kind: SyntaxKind, text: &str) {
        let e = self.kinds.len() as u32;
        self.push(kind as u16 | TOKEN_BIT, e + 1);
        self.text.push_str(text);
    }

    pub fn finish_node(&mut self) {
        let (e, _) = self.open.pop().expect("finish_node without start_node");
        self.ends[e as usize] = self.kinds.len() as u32;
    }

    /// Appends a copy of `e`'s subtree from another tree.
    pub fn push_subtree(&mut self, tree: &Tree, e: ElementId) {
        if tree.is_token(e) {
            return self.token(tree.kind(e), tree.text_of(e));
        }
        self.start_node(tree.kind(e));
        for child in tree.children(e) {
            self.push_subtree(tree, child);
        }
        self.finish_node();
    }

    pub fn finish(self) -> Tree {
        assert!(self.open.is_empty(), "unbalanced tree builder");
        assert!(self.parents.iter().skip(1).all(|&p| p != NONE), "tree has more than one root");
        Tree {
            text: self.text.into_boxed_str(),
            kinds: self.kinds,
            starts: self.starts,
            ends: self.ends,
            parents: self.parents,
            prev_sibs: self.prev_sibs,
        }
    }

    fn push(&mut self, raw_kind: u16, end: u32) -> ElementId {
        let e = self.kinds.len() as u32;
        let (parent, prev) = match self.open.last_mut() {
            Some((parent, last_child)) => (*parent, std::mem::replace(last_child, e)),
            None => (NONE, NONE),
        };
        self.kinds.push(raw_kind);
        self.starts.push(self.text.len() as u32);
        self.ends.push(end);
        self.parents.push(parent);
        self.prev_sibs.push(prev);
        e
    }
}
