//! `Doc.Level` from `Doc.java`. Upstream's `splits`/`breaks` lists become break indices into
//! `docs`: a split is the index range between two breaks, so nothing is stored per level.

use std::ops::Range as IndexRange;

use super::comments_helper::CommentsHelper;
use super::doc::{Doc, DocKind, FillMode, MAX_LINE_WIDTH, State};
use super::doc_leaves::DocBreak;
use super::indent::Indent;
use super::output::Output;
use super::range::Range;

/// A `Level` inside a `Doc`.
#[derive(Debug)]
pub struct Level {
    plus_indent: Indent,
    pub(super) docs: Vec<Doc>,
    /// True if the entire level fits on one line.
    pub(super) one_line: bool,
}

impl Level {
    pub(crate) fn make(plus_indent: Indent) -> Level {
        Level::with_capacity(plus_indent, 0)
    }

    pub(crate) fn with_capacity(plus_indent: Indent, docs: usize) -> Level {
        Level {
            plus_indent,
            docs: Vec::with_capacity(docs),
            one_line: false,
        }
    }

    pub(crate) fn add(&mut self, doc: Doc) {
        self.docs.push(doc);
    }

    pub fn docs(&self) -> &[Doc] {
        &self.docs
    }

    pub fn plus_indent(&self) -> &Indent {
        &self.plus_indent
    }

    /// `this_width` is the memoized `getWidth()` of the enclosing `Doc`.
    pub(super) fn compute_breaks(
        &mut self,
        comments_helper: &dyn CommentsHelper,
        max_width: i32,
        state: State,
        this_width: i32,
    ) -> State {
        if state.column + this_width <= max_width {
            self.one_line = true;
            return state.with_column(state.column + this_width);
        }
        let broken = self.compute_broken(
            comments_helper,
            max_width,
            State::new(state.indent + self.plus_indent.eval(), state.column),
        );
        state.with_column(broken.column)
    }

    /// The indices of the breaks; split `i` is the range between break `i - 1` and break `i`.
    fn split_by_breaks(docs: &[Doc]) -> Vec<usize> {
        (0..docs.len()).filter(|&i| matches!(docs[i].kind(), DocKind::Break(_))).collect()
    }

    /// Compute breaks for a `Level` that spans multiple lines.
    fn compute_broken(
        &mut self,
        comments_helper: &dyn CommentsHelper,
        max_width: i32,
        mut state: State,
    ) -> State {
        let docs = &mut self.docs;
        let breaks = Self::split_by_breaks(docs);
        let docs_n = docs.len();
        let split_end = |i: usize| breaks.get(i).copied().unwrap_or(docs_n);

        state = Self::compute_break_and_split(comments_helper, max_width, state, docs, None, 0..split_end(0));
        for i in 0..breaks.len() {
            let split = breaks[i] + 1..split_end(i + 1);
            state = Self::compute_break_and_split(comments_helper, max_width, state, docs, Some(breaks[i]), split);
        }
        state
    }

    /// Lay out a Break-separated group of Docs in the current Level.
    fn compute_break_and_split(
        comments_helper: &dyn CommentsHelper,
        max_width: i32,
        mut state: State,
        docs: &mut [Doc],
        opt_break_doc: Option<usize>,
        split: IndexRange<usize>,
    ) -> State {
        let break_width = opt_break_doc.map_or(0, |b| docs[b].get_width());
        let split_width = Self::get_width_of(docs[split.clone()].iter());
        let should_break = opt_break_doc
            .is_some_and(|b| Self::as_break(&mut docs[b]).fill_mode() == FillMode::Unified)
            || state.must_break
            || state.column + break_width + split_width > max_width;

        if let Some(b) = opt_break_doc {
            let last_indent = state.last_indent;
            state =
                Self::as_break(&mut docs[b]).compute_breaks_taken(state, last_indent, should_break);
        }
        let enough_room = state.column + split_width <= max_width;
        state = Self::compute_split(
            comments_helper,
            max_width,
            &mut docs[split],
            state.with_must_break(false),
        );
        if !enough_room {
            state = state.with_must_break(true); // Break after, too.
        }
        state
    }

    fn compute_split(
        comments_helper: &dyn CommentsHelper,
        max_width: i32,
        split: &mut [Doc],
        mut state: State,
    ) -> State {
        for doc in split {
            state = doc.compute_breaks(comments_helper, max_width, state);
        }
        state
    }

    /// The splits and the breaks between them, interleaved, are just `docs` in order.
    pub(super) fn write_filled(&self, output: &mut dyn Output, flat: &mut String) {
        for doc in &self.docs {
            doc.write_with(output, flat);
        }
    }

    /// `getWidth(List<Doc>)`: the summed width, saturating at `MAX_LINE_WIDTH`.
    pub(super) fn get_width_of<'d>(docs: impl Iterator<Item = &'d Doc>) -> i32 {
        let mut width = 0;
        for doc in docs {
            width += doc.get_width();
            if width >= MAX_LINE_WIDTH {
                return MAX_LINE_WIDTH; // Paranoid overflow protection
            }
        }
        width
    }

    pub(super) fn union(x: Range, y: Range) -> Range {
        if x.is_empty() {
            y
        } else if y.is_empty() {
            x
        } else {
            x.span(y)
        }
    }

    fn as_break(doc: &mut Doc) -> &mut DocBreak {
        match doc.kind_mut() {
            DocKind::Break(b) => b,
            _ => unreachable!("split_by_breaks only records breaks"),
        }
    }
}
