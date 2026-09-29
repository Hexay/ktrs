//! [`Tree`]: a parsed file as flat preorder arrays. An element (node or token) is its preorder
//! index; navigation is index arithmetic, and token text is a slice of the source. Replaces rowan's
//! green/red trees, which allocate per node when built and per step when walked (see
//! research/06-tree-library.md for the measurements).

mod builder;

pub use builder::TreeBuilder;

use crate::{SyntaxKind, TextRange, TextSize};

/// Preorder index of an element in its [`Tree`].
pub type ElementId = u32;

const NONE: u32 = u32::MAX;
/// Tokens and childless nodes both span one preorder slot; the kind's top bit tells them apart.
const TOKEN_BIT: u16 = 0x8000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    text: Box<str>,
    kinds: Vec<u16>,
    starts: Vec<u32>,
    /// Preorder index one past the element's subtree.
    ends: Vec<u32>,
    parents: Vec<u32>,
    prev_sibs: Vec<u32>,
}

impl Tree {
    pub const ROOT: ElementId = 0;

    /// The whole source text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Number of elements (nodes and tokens).
    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    pub fn kind(&self, e: ElementId) -> SyntaxKind {
        SyntaxKind::from_raw(self.kinds[e as usize] & !TOKEN_BIT)
    }

    pub fn is_token(&self, e: ElementId) -> bool {
        self.kinds[e as usize] & TOKEN_BIT != 0
    }

    /// One past the last element of `e`'s subtree, in preorder.
    pub fn subtree_end(&self, e: ElementId) -> ElementId {
        self.ends[e as usize]
    }

    pub fn parent(&self, e: ElementId) -> Option<ElementId> {
        some(self.parents[e as usize])
    }

    pub fn first_child(&self, e: ElementId) -> Option<ElementId> {
        (self.ends[e as usize] > e + 1).then_some(e + 1)
    }

    pub fn last_child(&self, e: ElementId) -> Option<ElementId> {
        let mut last = self.first_child(e)?;
        // The subtree's last element is a descendant of the last child; climb to it.
        let mut cur = self.ends[e as usize] - 1;
        while let Some(p) = self.parent(cur) {
            if p == e {
                last = cur;
                break;
            }
            cur = p;
        }
        Some(last)
    }

    pub fn next_sibling(&self, e: ElementId) -> Option<ElementId> {
        let parent = self.parent(e)?;
        let next = self.ends[e as usize];
        (next < self.ends[parent as usize]).then_some(next)
    }

    pub fn prev_sibling(&self, e: ElementId) -> Option<ElementId> {
        some(self.prev_sibs[e as usize])
    }

    pub fn children(&self, e: ElementId) -> Children<'_> {
        Children { tree: self, next: self.first_child(e) }
    }

    pub fn text_range(&self, e: ElementId) -> TextRange {
        let start = self.starts[e as usize];
        let end = self.starts.get(self.ends[e as usize] as usize).copied().unwrap_or(self.text.len() as u32);
        TextRange::new(TextSize::from(start), TextSize::from(end))
    }

    pub fn text_of(&self, e: ElementId) -> &str {
        &self.text[self.text_range(e)]
    }

    /// Whether any element strictly inside `e`'s subtree has `kind`.
    pub fn has_descendant_of_kind(&self, e: ElementId, kind: SyntaxKind) -> bool {
        let raw = kind as u16;
        self.kinds[e as usize + 1..self.ends[e as usize] as usize].iter().any(|&k| k & !TOKEN_BIT == raw)
    }

    /// The elements of `e`'s subtree (`e` included), in preorder, that are nodes of one of `nodes` or
    /// tokens of one of `tokens`: a scan of the raw kind words, cheaper than a `kind`/`is_token` walk.
    pub fn find_kinds<const N: usize, const M: usize>(
        &self,
        e: ElementId,
        nodes: [SyntaxKind; N],
        tokens: [SyntaxKind; M],
    ) -> impl Iterator<Item = ElementId> + '_ {
        let (nodes, tokens) = (nodes.map(|k| k as u16), tokens.map(|k| k as u16 | TOKEN_BIT));
        let start = e as usize;
        self.kinds[start..self.ends[start] as usize]
            .iter()
            .enumerate()
            .filter(move |(_, k)| nodes.contains(k) || tokens.contains(k))
            .map(move |(i, _)| (start + i) as ElementId)
    }
}

pub struct Children<'t> {
    tree: &'t Tree,
    next: Option<ElementId>,
}

impl Iterator for Children<'_> {
    type Item = ElementId;

    fn next(&mut self) -> Option<ElementId> {
        let cur = self.next?;
        self.next = self.tree.next_sibling(cur);
        Some(cur)
    }
}

fn some(index: u32) -> Option<ElementId> {
    (index != NONE).then_some(index)
}

#[cfg(test)]
mod tests;
