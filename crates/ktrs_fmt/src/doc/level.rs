//! `Doc.Level` from `Doc.java`. `splits`/`breaks` hold indices into `docs` instead of aliasing
//! references.

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
    /// Groups of child indices separated by breaks.
    splits: Vec<Vec<usize>>,
    /// Indices of the breaks between the splits.
    breaks: Vec<usize>,
}

impl Level {
    pub(crate) fn make(plus_indent: Indent) -> Level {
        Level {
            plus_indent,
            docs: Vec::new(),
            one_line: false,
            splits: Vec::new(),
            breaks: Vec::new(),
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

    fn split_by_breaks(docs: &[Doc], splits: &mut Vec<Vec<usize>>, breaks: &mut Vec<usize>) {
        splits.clear();
        breaks.clear();
        splits.push(Vec::new());
        for (i, doc) in docs.iter().enumerate() {
            if matches!(doc.kind(), DocKind::Break(_)) {
                breaks.push(i);
                splits.push(Vec::new());
            } else {
                splits.last_mut().unwrap().push(i);
            }
        }
    }

    /// Compute breaks for a `Level` that spans multiple lines.
    fn compute_broken(
        &mut self,
        comments_helper: &dyn CommentsHelper,
        max_width: i32,
        mut state: State,
    ) -> State {
        let Level {
            docs,
            splits,
            breaks,
            ..
        } = self;
        Self::split_by_breaks(docs, splits, breaks);

        state = Self::compute_break_and_split(
            comments_helper,
            max_width,
            state,
            docs,
            None,
            &splits[0],
        );
        for i in 0..breaks.len() {
            state = Self::compute_break_and_split(
                comments_helper,
                max_width,
                state,
                docs,
                Some(breaks[i]),
                &splits[i + 1],
            );
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
        split: &[usize],
    ) -> State {
        let break_width = opt_break_doc.map_or(0, |b| docs[b].get_width());
        let split_width = Self::get_width_of(split.iter().map(|&i| &docs[i]));
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
            docs,
            split,
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
        docs: &mut [Doc],
        split: &[usize],
        mut state: State,
    ) -> State {
        for &i in split {
            state = docs[i].compute_breaks(comments_helper, max_width, state);
        }
        state
    }

    pub(super) fn write_filled(&self, output: &mut dyn Output) {
        for &i in &self.splits[0] {
            self.docs[i].write(output);
        }
        for (b, split) in self.breaks.iter().zip(&self.splits[1..]) {
            self.docs[*b].write(output);
            for &i in split {
                self.docs[i].write(output);
            }
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
