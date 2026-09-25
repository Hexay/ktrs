//! Port of `Doc.java`: the abstract `Doc`, `FillMode` and `State`. `Level` is in `level`, the
//! leaf docs (`Token`, `Space`, `Break`, `Tok`) in `doc_leaves`.
//!
//! Java's `Doc` subclasses become [`DocKind`] variants inside [`Doc`], which owns the memoized
//! width/flat/range suppliers.

use std::cell::OnceCell;

use super::comments_helper::CommentsHelper;
use super::doc_leaves::{DocBreak, DocTok, DocToken};
use super::level::Level;
use super::output::Output;
use super::range::{EMPTY_RANGE, Range};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillMode {
    /// If a `Level` will not fit on one line, all of its `UNIFIED` breaks will be broken.
    Unified,
    /// If a `Level` will not fit on one line, its `INDEPENDENT` breaks are broken independently.
    Independent,
    /// Always broken; a `Level` containing one never fits on one line.
    Forced,
}

/// Sentinel width for docs that break unconditionally; small enough to prevent overflow.
pub const MAX_LINE_WIDTH: i32 = 1000;

/// State for writing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub(crate) last_indent: i32,
    pub(crate) indent: i32,
    pub(crate) column: i32,
    pub(crate) must_break: bool,
}

impl State {
    fn with_all(last_indent: i32, indent: i32, column: i32, must_break: bool) -> State {
        State {
            last_indent,
            indent,
            column,
            must_break,
        }
    }

    pub fn new(indent0: i32, column0: i32) -> State {
        State::with_all(indent0, indent0, column0, false)
    }

    pub(crate) fn with_column(self, column: i32) -> State {
        State::with_all(self.last_indent, self.indent, column, self.must_break)
    }

    pub(super) fn with_must_break(self, must_break: bool) -> State {
        State::with_all(self.last_indent, self.indent, self.column, must_break)
    }
}

#[derive(Debug)]
pub struct Doc {
    kind: DocKind,
    width: OnceCell<i32>,
    // Not defined (and never computed) if the doc contains forced breaks.
    flat: OnceCell<String>,
    range: OnceCell<Range>,
}

#[derive(Debug)]
pub enum DocKind {
    Level(Level),
    Token(DocToken),
    Space,
    Break(DocBreak),
    Tok(DocTok),
}

impl Doc {
    pub(crate) fn new(kind: DocKind) -> Doc {
        Doc {
            kind,
            width: OnceCell::new(),
            flat: OnceCell::new(),
            range: OnceCell::new(),
        }
    }

    pub fn kind(&self) -> &DocKind {
        &self.kind
    }

    pub(super) fn kind_mut(&mut self) -> &mut DocKind {
        &mut self.kind
    }

    pub fn get_width(&self) -> i32 {
        *self.width.get_or_init(|| self.compute_width())
    }

    pub fn get_flat(&self) -> &str {
        self.flat.get_or_init(|| self.compute_flat())
    }

    pub fn range(&self) -> Range {
        *self.range.get_or_init(|| self.compute_range())
    }

    fn compute_width(&self) -> i32 {
        match &self.kind {
            DocKind::Level(level) => Level::get_width_of(level.docs.iter()),
            DocKind::Token(token) => token.compute_width(),
            DocKind::Space => 1,
            DocKind::Break(b) => b.compute_width(),
            DocKind::Tok(tok) => tok.compute_width(),
        }
    }

    fn compute_flat(&self) -> String {
        match &self.kind {
            DocKind::Level(level) => level.docs.iter().map(Doc::get_flat).collect(),
            DocKind::Token(token) => token.compute_flat(),
            DocKind::Space => " ".to_string(),
            DocKind::Break(b) => b.compute_flat(),
            DocKind::Tok(tok) => tok.compute_flat(),
        }
    }

    fn compute_range(&self) -> Range {
        match &self.kind {
            DocKind::Level(level) => level
                .docs
                .iter()
                .fold(EMPTY_RANGE, |acc, doc| Level::union(acc, doc.range())),
            DocKind::Token(token) => token.compute_range(),
            DocKind::Space | DocKind::Break(_) => EMPTY_RANGE,
            DocKind::Tok(tok) => tok.compute_range(),
        }
    }

    /// Make breaking decisions for a `Doc`; returns the new output state.
    pub fn compute_breaks(
        &mut self,
        comments_helper: &dyn CommentsHelper,
        max_width: i32,
        state: State,
    ) -> State {
        let this_width = if matches!(self.kind, DocKind::Level(_)) {
            self.get_width()
        } else {
            0
        };
        match &mut self.kind {
            DocKind::Level(level) => {
                level.compute_breaks(comments_helper, max_width, state, this_width)
            }
            DocKind::Token(token) => token.compute_breaks(state),
            DocKind::Space => state.with_column(state.column + 1),
            DocKind::Break(_) => panic!("Did you mean computeBreaks(State, int, boolean)?"),
            DocKind::Tok(tok) => tok.compute_breaks(comments_helper, max_width, state),
        }
    }

    /// Write a `Doc` to an `Output`, after breaking decisions have been made.
    pub fn write(&self, output: &mut dyn Output) {
        match &self.kind {
            DocKind::Level(level) => {
                if level.one_line {
                    // Defined because the width is finite.
                    output.append(self.get_flat(), self.range());
                } else {
                    level.write_filled(output);
                }
            }
            DocKind::Token(token) => token.write(output, self.range()),
            DocKind::Space => output.append(" ", self.range()),
            DocKind::Break(b) => b.write(output, self.range()),
            DocKind::Tok(tok) => tok.write(output, self.range()),
        }
    }
}
