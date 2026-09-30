//! The editing half of ktlint-rule-engine-core `ASTNodeExtension.kt` (`upsertWhitespaceBeforeMe`,
//! `replaceTextWith`, `upsertWhitespaceAfterMe`, `replaceWith`, `remove`), in file order; the queries are in
//! `ast_node_extension/`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::WHITE_SPACE;

use crate::ast_node_extension::AstNodeExtension;

pub trait AstNodeEdit {
    fn upsert_whitespace_before_me(&mut self, n: NodeId, text: &str);
    fn replace_text_with(&mut self, n: NodeId, text: &str);
    fn upsert_whitespace_after_me(&mut self, n: NodeId, text: &str);
    fn replace_with(&mut self, n: NodeId, node: NodeId);
    fn remove(&mut self, n: NodeId);
}

impl AstNodeEdit for Ast {
    /// Rewrites an adjacent whitespace, else inserts one; never as a composite's first child (climbs).
    fn upsert_whitespace_before_me(&mut self, n: NodeId, text: &str) {
        if self.is_leaf(n) {
            if self.is_white_space(n) {
                return self.replace_text_with(n, text);
            }
            let previous = self.prev_sibling(n).or_else(|| self.prev_leaf(n));
            if let Some(previous) = previous.filter(|&p| self.is_white_space(p)) {
                self.replace_text_with(previous, text);
            } else if self.parent(n).and_then(|p| self.first_child_node(p)) == Some(n) {
                if let Some(parent) = self.parent(n) {
                    self.upsert_whitespace_before_me(parent, text);
                }
            } else {
                assert!(self.is_leaf_element(n), "ClassCastException: (psi as LeafElement) on an empty composite");
                let psi_white_space = self.new_leaf(WHITE_SPACE, text);
                self.raw_insert_before_me(n, psi_white_space);
            }
        } else {
            match self.prev_sibling(n) {
                None => {
                    if let Some(parent) = self.parent(n) {
                        self.upsert_whitespace_before_me(parent, text);
                    }
                }
                Some(prev_sibling) if self.is_leaf_element(prev_sibling) => self.upsert_whitespace_after_me(prev_sibling, text),
                Some(_) => {
                    let psi_white_space = self.new_leaf(WHITE_SPACE, text);
                    if let Some(parent) = self.parent(n) {
                        self.add_child(parent, psi_white_space, Some(n));
                    }
                }
            }
        }
    }

    /// `rawReplaceWithText` unless equal: the leaf is replaced by a new node.
    fn replace_text_with(&mut self, n: NodeId, text: &str) {
        assert!(self.is_leaf_element(n), "IllegalArgumentException: replaceTextWith on a composite");
        if self.leaf_text(n) != text {
            self.raw_replace_with_text(n, text);
        }
    }

    fn upsert_whitespace_after_me(&mut self, n: NodeId, text: &str) {
        if self.is_leaf(n) {
            if self.is_white_space(n) {
                return self.replace_text_with(n, text);
            }
            let next = self.next_sibling(n).or_else(|| self.next_leaf(n));
            if let Some(next) = next.filter(|&x| self.is_white_space(x)) {
                self.replace_text_with(next, text);
            } else if self.parent(n).and_then(|p| self.last_child_node(p)) == Some(n) {
                if let Some(parent) = self.parent(n) {
                    self.upsert_whitespace_after_me(parent, text);
                }
            } else {
                assert!(self.is_leaf_element(n), "ClassCastException: (psi as LeafElement) on an empty composite");
                let psi_white_space = self.new_leaf(WHITE_SPACE, text);
                self.raw_insert_after_me(n, psi_white_space);
            }
        } else {
            match self.next_sibling(n) {
                None => {
                    if let Some(parent) = self.parent(n) {
                        self.upsert_whitespace_after_me(parent, text);
                    }
                }
                Some(next_sibling) if self.is_leaf_element(next_sibling) => self.upsert_whitespace_before_me(next_sibling, text),
                Some(next_sibling) => {
                    let psi_white_space = self.new_leaf(WHITE_SPACE, text);
                    if let Some(parent) = self.parent(n) {
                        self.add_child(parent, psi_white_space, Some(next_sibling));
                    }
                }
            }
        }
    }

    /// `parent.addChild(node, this)`, then `remove()`.
    fn replace_with(&mut self, n: NodeId, node: NodeId) {
        if let Some(parent) = self.parent(n) {
            self.add_child(parent, node, Some(n));
        }
        self.remove(n);
    }

    /// `parent.removeChild(this)`: the node moves into a dummy holder; adjacent whitespace is not merged.
    fn remove(&mut self, n: NodeId) {
        if let Some(parent) = self.parent(n) {
            self.remove_child(parent, n);
        }
    }
}
