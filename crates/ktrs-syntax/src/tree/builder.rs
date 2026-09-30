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
    /// The smallest capacity of the five element arrays (which always have equal lengths).
    room: usize,
}

impl TreeBuilder {
    pub fn new() -> TreeBuilder {
        TreeBuilder::default()
    }

    /// Room for `elements` elements over `text_len` bytes of text, so the arrays never regrow.
    pub fn with_capacity(elements: usize, text_len: usize) -> TreeBuilder {
        let mut builder = TreeBuilder { text: String::with_capacity(text_len), ..TreeBuilder::default() };
        builder.reserve(elements);
        builder
    }

    fn reserve(&mut self, additional: usize) {
        self.kinds.reserve(additional);
        self.starts.reserve(additional);
        self.ends.reserve(additional);
        self.parents.reserve(additional);
        self.prev_sibs.reserve(additional);
        self.update_room();
    }

    fn update_room(&mut self) {
        self.room = [
            self.kinds.capacity(),
            self.starts.capacity(),
            self.ends.capacity(),
            self.parents.capacity(),
            self.prev_sibs.capacity(),
        ]
        .into_iter()
        .min()
        .unwrap_or(0);
    }

    #[inline]
    pub fn start_node(&mut self, kind: SyntaxKind) {
        let e = self.push(kind as u16, NONE);
        self.open.push((e, NONE));
    }

    #[inline]
    pub fn token(&mut self, kind: SyntaxKind, text: &str) {
        let e = self.kinds.len() as u32;
        self.push(kind as u16 | TOKEN_BIT, e + 1);
        self.text.push_str(text);
    }

    #[inline]
    pub fn finish_node(&mut self) {
        let (e, _) = self.open.pop().expect("finish_node without start_node");
        self.ends[e as usize] = self.kinds.len() as u32;
    }

    /// Elements pushed so far; the id the next element gets.
    pub fn len(&self) -> ElementId {
        self.kinds.len() as ElementId
    }

    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    /// A standalone copy of the finished subtree rooted at `root` (a node pushed at the current
    /// nesting level, with nothing pushed after its end).
    pub fn extract(&self, root: ElementId) -> Tree {
        let r = root as usize;
        assert_eq!(self.ends[r], self.len(), "extract of an open or non-final subtree");
        let base = self.starts[r];
        // The root's own parent and previous sibling lie before it: it becomes a standalone root.
        let rebase = |ids: &[u32]| {
            std::iter::once(NONE).chain(ids[1..].iter().map(|&x| if x == NONE { NONE } else { x - root })).collect()
        };
        let parents: Vec<u32> = rebase(&self.parents[r..]);
        let prev_sibs: Vec<u32> = rebase(&self.prev_sibs[r..]);
        Tree {
            text: self.text[base as usize..].into(),
            kinds: self.kinds[r..].to_vec(),
            starts: self.starts[r..].iter().map(|s| s - base).collect(),
            ends: self.ends[r..].iter().map(|e| e - root).collect(),
            parents,
            prev_sibs,
        }
    }

    /// Appends all of `tree` as the next child of the open node: [`Self::push_subtree`] of its
    /// root, as block copies.
    pub fn push_tree(&mut self, tree: &Tree) {
        let root = self.len();
        let text_base = self.text.len() as u32;
        let (parent, prev) = match self.open.last_mut() {
            Some((parent, last_child)) => (*parent, std::mem::replace(last_child, root)),
            None => (NONE, NONE),
        };
        let shift = |x: u32| if x == NONE { NONE } else { x + root };
        self.kinds.extend_from_slice(&tree.kinds);
        self.starts.extend(tree.starts.iter().map(|s| s + text_base));
        self.ends.extend(tree.ends.iter().map(|&e| e + root));
        self.parents.push(parent);
        self.parents.extend(tree.parents[1..].iter().map(|&p| shift(p)));
        self.prev_sibs.push(prev);
        self.prev_sibs.extend(tree.prev_sibs[1..].iter().map(|&p| shift(p)));
        self.text.push_str(&tree.text);
        self.update_room();
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

    #[inline]
    fn push(&mut self, raw_kind: u16, end: u32) -> ElementId {
        let len = self.kinds.len();
        if len >= self.room {
            self.reserve(len.max(64));
        }
        let e = len as u32;
        let (parent, prev) = match self.open.last_mut() {
            Some((parent, last_child)) => (*parent, std::mem::replace(last_child, e)),
            None => (NONE, NONE),
        };
        let start = self.text.len() as u32;
        // SAFETY: the five arrays all have length `len` (every mutation appends to each of them)
        // and capacity of at least `room > len`.
        unsafe {
            push_unchecked(&mut self.kinds, raw_kind);
            push_unchecked(&mut self.starts, start);
            push_unchecked(&mut self.ends, end);
            push_unchecked(&mut self.parents, parent);
            push_unchecked(&mut self.prev_sibs, prev);
        }
        e
    }
}

/// # Safety
/// `v.len() < v.capacity()`.
#[inline(always)]
unsafe fn push_unchecked<T>(v: &mut Vec<T>, x: T) {
    debug_assert!(v.len() < v.capacity());
    let len = v.len();
    // SAFETY: the slot at `len` is allocated (caller's contract) and uninitialized.
    unsafe {
        v.as_mut_ptr().add(len).write(x);
        v.set_len(len + 1);
    }
}
