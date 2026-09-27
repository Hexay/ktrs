//! Port of `Doc.java`: the abstract `Doc`, `FillMode` and `State`. `Level` is in `level`, the
//! leaf docs (`Token`, `Space`, `Break`, `Tok`) in `doc_leaves`.
//!
//! Java's `Doc` subclasses become [`DocKind`] variants inside [`Doc`], which owns the memoized
//! width supplier. Flat text and range are not memoized: `write` reads each doc's at most once
//! (a one-line level reads its subtree's, and then never writes that subtree).

use std::cell::Cell;

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
    /// Memoized `getWidth()`; `-1` until computed (widths are never negative).
    width: Cell<i32>,
}

#[derive(Debug)]
pub enum DocKind {
    Level(Level),
    Token(DocToken),
    Space,
    Break(DocBreak),
    Tok(DocTok),
}

// `Level` is stored inline (boxing it cost one allocation per level); it must not grow `DocKind`.
const _: () = assert!(std::mem::size_of::<Level>() <= std::mem::size_of::<DocBreak>());

impl Doc {
    pub(crate) fn new(kind: DocKind) -> Doc {
        Doc {
            kind,
            width: Cell::new(-1),
        }
    }

    pub fn kind(&self) -> &DocKind {
        &self.kind
    }

    pub(super) fn kind_mut(&mut self) -> &mut DocKind {
        &mut self.kind
    }

    pub fn get_width(&self) -> i32 {
        let width = self.width.get();
        if width >= 0 {
            return width;
        }
        let width = self.compute_width();
        self.width.set(width);
        width
    }

    pub fn get_flat(&self) -> String {
        let mut flat = String::new();
        self.compute_flat(&mut flat);
        flat
    }

    pub fn range(&self) -> Range {
        self.compute_range()
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

    /// Appends the flat text to `out` and returns `range()`, walking the subtree once for both.
    fn compute_flat(&self, out: &mut String) -> Range {
        match &self.kind {
            DocKind::Level(level) => level
                .docs
                .iter()
                .fold(EMPTY_RANGE, |acc, doc| Level::union(acc, doc.compute_flat(out))),
            DocKind::Token(token) => {
                token.compute_flat(out);
                token.compute_range()
            }
            DocKind::Space => {
                out.push(' ');
                EMPTY_RANGE
            }
            DocKind::Break(b) => {
                b.compute_flat(out);
                EMPTY_RANGE
            }
            DocKind::Tok(tok) => {
                tok.compute_flat(out);
                tok.compute_range()
            }
        }
    }

    fn compute_range(&self) -> Range {
        match &self.kind {
            DocKind::Level(level) => level
                .docs
                .iter()
                .fold(EMPTY_RANGE, |acc, doc| Level::union(acc, doc.compute_range())),
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
        let this_width = if matches!(self.kind, DocKind::Level(_) | DocKind::Token(_)) {
            self.get_width()
        } else {
            0
        };
        match &mut self.kind {
            DocKind::Level(level) => {
                level.compute_breaks(comments_helper, max_width, state, this_width)
            }
            // Token.computeBreaks: its computeWidth() is the memoized width.
            DocKind::Token(_) => state.with_column(state.column + this_width),
            DocKind::Space => state.with_column(state.column + 1),
            DocKind::Break(_) => panic!("Did you mean computeBreaks(State, int, boolean)?"),
            DocKind::Tok(tok) => tok.compute_breaks(comments_helper, max_width, state),
        }
    }

    /// Write a `Doc` to an `Output`, after breaking decisions have been made.
    pub fn write(&self, output: &mut dyn Output) {
        self.write_with(output, &mut String::new());
    }

    /// `write`, building one-line levels' flat text in the reused buffer `flat`.
    pub(super) fn write_with(&self, output: &mut dyn Output, flat: &mut String) {
        match &self.kind {
            DocKind::Level(level) => {
                if level.one_line {
                    // Defined because the width is finite.
                    flat.clear();
                    let range = self.compute_flat(flat);
                    output.append(flat, range);
                } else {
                    level.write_filled(output, flat);
                }
            }
            DocKind::Token(token) => token.write(output, token.compute_range()),
            DocKind::Space => output.append(" ", EMPTY_RANGE),
            DocKind::Break(b) => b.write(output, EMPTY_RANGE),
            DocKind::Tok(tok) => tok.write(output, tok.compute_range()),
        }
    }
}
