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

pub struct DocBuilder {
    levels: Vec<Level>,
    /// Where each closed level goes: its parent and the index of its placeholder there.
    slots: Vec<Option<(usize, usize)>>,
    stack: Vec<usize>,
    append_level: usize,
}

impl Default for DocBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DocBuilder {
    const BASE: usize = 0;

    pub fn new() -> DocBuilder {
        DocBuilder {
            levels: vec![Level::make(Indent::ZERO)],
            slots: vec![None],
            stack: vec![Self::BASE],
            append_level: Self::BASE,
        }
    }

    pub fn with_ops(mut self, ops: Vec<Op>) -> DocBuilder {
        let opens = ops.iter().filter(|op| matches!(op, Op::Open(_))).count();
        self.levels.reserve(opens);
        self.slots.reserve(opens);
        for op in ops {
            op.add(&mut self); // These operations call the operations below to build the doc.
        }
        self
    }

    pub(crate) fn open(&mut self, plus_indent: Indent) {
        self.levels.push(Level::make(plus_indent));
        self.slots.push(None);
        self.stack.push(self.levels.len() - 1);
    }

    pub(crate) fn close(&mut self) {
        let top = self.stack.pop().expect("close without open");
        let parent = *self.stack.last().expect("close of the base level");
        self.slots[top] = Some((parent, self.levels[parent].docs.len()));
        self.levels[parent].add(Doc::new(DocKind::Space));
    }

    pub(crate) fn add(&mut self, doc: Doc) {
        self.levels[self.append_level].add(doc);
    }

    pub(crate) fn break_doc(&mut self, break_doc: Doc) {
        self.append_level = *self.stack.last().unwrap();
        self.levels[self.append_level].add(break_doc);
    }

    pub fn build(self) -> Doc {
        let mut levels: Vec<Option<Level>> = self.levels.into_iter().map(Some).collect();
        // A level's children were opened after it, so they have higher indices.
        for i in (Self::BASE + 1..levels.len()).rev() {
            let Some((parent, index)) = self.slots[i] else { continue };
            let level = levels[i].take().expect("level attached twice");
            levels[parent].as_mut().expect("parent attached before child").docs[index] =
                Doc::new(DocKind::Level(Box::new(level)));
        }
        Doc::new(DocKind::Level(Box::new(levels[Self::BASE].take().unwrap())))
    }
}
