//! Port of `Output.java` (and its nested `BreakTag`).

use std::cell::Cell;
use std::rc::Rc;

use super::blank_line_wanted::BlankLineWanted;
use super::comments_helper::CommentsHelper;
use super::input::Token;
use super::range::Range;

/// Remembers whether a tagged `Break` was taken. Clones share state (Java reference semantics).
#[derive(Clone, Debug, Default)]
pub struct BreakTag {
    taken: Rc<Cell<Option<bool>>>,
}

impl BreakTag {
    pub fn new() -> BreakTag {
        BreakTag::default()
    }

    pub fn record_broken(&self, broken: bool) {
        self.taken.set(Some(broken));
    }

    pub fn was_break_taken(&self) -> bool {
        self.taken.get().unwrap_or(false)
    }
}

pub trait Output {
    fn indent(&mut self, indent: i32);

    fn append(&mut self, text: &str, range: Range);

    fn blank_line(&mut self, k: i32, wanted: BlankLineWanted);

    fn mark_for_partial_format(&mut self, start: &dyn Token, end: &dyn Token);

    fn get_comments_helper(&self) -> &dyn CommentsHelper;
}
