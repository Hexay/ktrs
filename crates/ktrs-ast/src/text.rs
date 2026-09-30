//! Text queries (`getText`, `textContains`, `textMatches`) and the UTF-16 conversion for emits.

use crate::arena::{Ast, NodeId};

/// Preorder over `root`'s subtree, `root` included.
pub(crate) struct Preorder<'a> {
    ast: &'a Ast,
    root: NodeId,
    next: Option<NodeId>,
}

impl Iterator for Preorder<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let cur = self.next?;
        self.next = self.ast.first_child_node(cur).or_else(|| {
            let mut c = cur;
            loop {
                if c == self.root {
                    return None;
                }
                if let Some(next) = self.ast.tree_next(c) {
                    return Some(next);
                }
                c = self.ast.tree_parent(c)?;
            }
        });
        Some(cur)
    }
}

impl Ast {
    pub(crate) fn preorder(&self, root: NodeId) -> Preorder<'_> {
        Preorder { ast: self, root, next: Some(root) }
    }

    /// The leaf texts of `n`'s subtree in order.
    pub fn text_chunks(&self, n: NodeId) -> impl Iterator<Item = &str> + '_ {
        self.preorder(n).filter(|&e| self.is_leaf_element(e)).map(|e| self.leaf_text(e))
    }

    /// `getText()`.
    pub fn text(&self, n: NodeId) -> String {
        let mut text = String::with_capacity(self.text_length(n));
        self.text_chunks(n).for_each(|chunk| text.push_str(chunk));
        text
    }

    /// `textContains(c)`.
    pub fn text_contains(&self, n: NodeId, c: char) -> bool {
        self.text_chunks(n).any(|chunk| chunk.contains(c))
    }

    /// `textMatches(seq)`.
    pub fn text_matches(&self, n: NodeId, seq: &str) -> bool {
        if self.text_length(n) != seq.len() {
            return false;
        }
        let mut rest = seq;
        self.text_chunks(n).all(|chunk| match rest.strip_prefix(chunk) {
            Some(r) => {
                rest = r;
                true
            }
            None => false,
        })
    }

    /// The UTF-16 offset (what the JVM's `startOffset` counts) of the UTF-8 `byte_offset` into the
    /// text of `n`'s tree.
    pub fn utf16_offset(&self, n: NodeId, byte_offset: usize) -> usize {
        if self.is_ascii() {
            return byte_offset;
        }
        let mut top = n;
        while let Some(p) = self.tree_parent(top) {
            top = p;
        }
        let (mut bytes, mut units) = (0, 0);
        for chunk in self.text_chunks(top) {
            if bytes + chunk.len() >= byte_offset {
                return units + chunk[..byte_offset - bytes].encode_utf16().count();
            }
            bytes += chunk.len();
            units += chunk.encode_utf16().count();
        }
        units
    }
}
