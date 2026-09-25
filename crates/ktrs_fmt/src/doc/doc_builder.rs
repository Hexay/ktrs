//! Port of `DocBuilder.java`.
//!
//! Java keeps `appendLevel` as a reference that can outlive the level's `close()` (docs added
//! after a close, before the next break, still land in the closed level). Levels therefore live in
//! an arena here and are assembled into the `Doc` tree by `build`.

use super::doc::{Doc, DocKind};
use super::indent::Indent;
use super::level::Level;
use super::op::Op;

enum Item {
    Doc(Doc),
    Level(usize),
}

struct LevelBuf {
    plus_indent: Indent,
    items: Vec<Item>,
}

pub struct DocBuilder {
    levels: Vec<LevelBuf>,
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
            levels: vec![LevelBuf {
                plus_indent: Indent::ZERO,
                items: Vec::new(),
            }],
            stack: vec![Self::BASE],
            append_level: Self::BASE,
        }
    }

    pub fn with_ops(mut self, ops: Vec<Op>) -> DocBuilder {
        for op in ops {
            op.add(&mut self); // These operations call the operations below to build the doc.
        }
        self
    }

    pub(crate) fn open(&mut self, plus_indent: Indent) {
        self.levels.push(LevelBuf {
            plus_indent,
            items: Vec::new(),
        });
        self.stack.push(self.levels.len() - 1);
    }

    pub(crate) fn close(&mut self) {
        let top = self.stack.pop().expect("close without open");
        let parent = *self.stack.last().expect("close of the base level");
        self.levels[parent].items.push(Item::Level(top));
    }

    pub(crate) fn add(&mut self, doc: Doc) {
        self.levels[self.append_level].items.push(Item::Doc(doc));
    }

    pub(crate) fn break_doc(&mut self, break_doc: Doc) {
        self.append_level = *self.stack.last().unwrap();
        self.levels[self.append_level]
            .items
            .push(Item::Doc(break_doc));
    }

    pub fn build(self) -> Doc {
        let mut levels: Vec<Option<LevelBuf>> = self.levels.into_iter().map(Some).collect();
        Self::assemble(&mut levels, Self::BASE)
    }

    fn assemble(levels: &mut [Option<LevelBuf>], i: usize) -> Doc {
        let buf = levels[i].take().expect("level attached twice");
        let mut level = Level::make(buf.plus_indent);
        for item in buf.items {
            match item {
                Item::Doc(doc) => level.add(doc),
                Item::Level(child) => level.add(Self::assemble(levels, child)),
            }
        }
        Doc::new(DocKind::Level(level))
    }
}
