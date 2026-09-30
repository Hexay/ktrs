//! Port of `DocBuilder.java`.
//!
//! Java keeps `appendLevel` as a reference that can outlive the level's `close()` (docs added
//! after a close, before the next break, still land in the closed level). Levels therefore live in
//! an arena here: `close` leaves a placeholder in the parent, and `build` moves each level into its
//! placeholder, children first.

use super::doc::{Doc, DocKind};
use super::indent::Indent;
use super::level::Level;
use super::op::Op;

pub struct DocBuilder<'a> {
    levels: Vec<Level<'a>>,
    /// Where each closed level goes: its parent and the index of its placeholder there.
    slots: Vec<Option<(usize, usize)>>,
    stack: Vec<usize>,
    append_level: usize,
    /// Docs each level will receive, by creation order (see `child_counts`).
    child_counts: Vec<u32>,
}

impl Default for DocBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DocBuilder<'a> {
    const BASE: usize = 0;

    pub fn new() -> DocBuilder<'a> {
        DocBuilder {
            levels: vec![Level::make(Indent::ZERO)],
            slots: vec![None],
            stack: vec![Self::BASE],
            append_level: Self::BASE,
            child_counts: Vec::new(),
        }
    }

    pub fn with_ops(mut self, ops: Vec<Op<'a>>) -> DocBuilder<'a> {
        self.child_counts = Self::child_counts(&ops);
        let opens = self.child_counts.len() - 1;
        self.levels.reserve(opens);
        self.slots.reserve(opens);
        self.levels[Self::BASE].docs.reserve_exact(self.child_counts[Self::BASE] as usize);
        for op in ops {
            op.add(&mut self); // These operations call the operations below to build the doc.
        }
        self
    }

    /// How many docs each level (by creation order) will receive: a dry run of `open`/`close`/
    /// `add`/`break_doc`, so each level's Vec is allocated once at its final size.
    fn child_counts(ops: &[Op<'_>]) -> Vec<u32> {
        let mut counts = vec![0u32];
        let mut stack = vec![Self::BASE];
        let mut append_level = Self::BASE;
        for op in ops {
            match op {
                Op::Open(_) => {
                    stack.push(counts.len());
                    counts.push(0);
                }
                Op::Close => {
                    stack.pop();
                    counts[*stack.last().expect("close of the base level")] += 1;
                }
                Op::Break(_) => {
                    append_level = *stack.last().unwrap();
                    counts[append_level] += 1;
                }
                Op::Token(_) | Op::Space | Op::Tok(_) => counts[append_level] += 1,
                Op::FenceComments => {}
            }
        }
        counts
    }

    pub(crate) fn open(&mut self, plus_indent: Indent) {
        let capacity = self.child_counts.get(self.levels.len()).copied().unwrap_or(0);
        self.levels.push(Level::with_capacity(plus_indent, capacity as usize));
        self.slots.push(None);
        self.stack.push(self.levels.len() - 1);
    }

    pub(crate) fn close(&mut self) {
        let top = self.stack.pop().expect("close without open");
        let parent = *self.stack.last().expect("close of the base level");
        self.slots[top] = Some((parent, self.levels[parent].docs.len()));
        self.levels[parent].add(Doc::new(DocKind::Space));
    }

    pub(crate) fn add(&mut self, doc: Doc<'a>) {
        self.levels[self.append_level].add(doc);
    }

    pub(crate) fn break_doc(&mut self, break_doc: Doc<'a>) {
        self.append_level = *self.stack.last().unwrap();
        self.levels[self.append_level].add(break_doc);
    }

    pub fn build(self) -> Doc<'a> {
        let mut levels: Vec<Option<Level<'a>>> = self.levels.into_iter().map(Some).collect();
        // A level's children were opened after it, so they have higher indices.
        for i in (Self::BASE + 1..levels.len()).rev() {
            let Some((parent, index)) = self.slots[i] else { continue };
            let level = levels[i].take().expect("level attached twice");
            levels[parent].as_mut().expect("parent attached before child").docs[index] =
                Doc::new(DocKind::Level(level));
        }
        Doc::new(DocKind::Level(levels[Self::BASE].take().unwrap()))
    }
}
