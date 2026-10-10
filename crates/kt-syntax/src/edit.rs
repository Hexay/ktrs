use std::fmt;
use std::ops::Range;

/// A replacement of a byte range of a text. Edits are plain values: collect them from nodes
/// ([`Node::replace`](crate::Node::replace), [`Node::remove`](crate::Node::remove), ..) or build them from
/// ranges, then turn the lot into a new text with [`apply_edits`]. The tree is never mutated; parse the result to
/// continue.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextEdit {
    /// The bytes to replace; empty for an insertion.
    pub range: Range<usize>,
    /// The replacement; empty for a deletion.
    pub text: String,
}

impl TextEdit {
    /// Replaces `range` with `text`.
    pub fn replace(range: Range<usize>, text: impl Into<String>) -> TextEdit {
        TextEdit { range, text: text.into() }
    }

    /// Inserts `text` at `offset`.
    pub fn insert(offset: usize, text: impl Into<String>) -> TextEdit {
        TextEdit { range: offset..offset, text: text.into() }
    }

    /// Deletes `range`.
    pub fn delete(range: Range<usize>) -> TextEdit {
        TextEdit { range, text: String::new() }
    }
}

/// Why [`apply_edits`] refused a set of edits.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditError {
    /// Two edits touch the same bytes: their ranges intersect, or one inserts strictly inside the other's range.
    Overlapping {
        /// The edit that starts first.
        first: Range<usize>,
        /// The edit that starts inside it.
        second: Range<usize>,
    },
    /// A range is reversed, ends past the text, or splits a UTF-8 character.
    InvalidRange {
        /// The offending range.
        range: Range<usize>,
    },
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::Overlapping { first, second } => write!(f, "edits {first:?} and {second:?} overlap"),
            EditError::InvalidRange { range } => write!(f, "edit range {range:?} is not a range of the text"),
        }
    }
}

impl std::error::Error for EditError {}

/// Applies `edits` to `text` and returns the new text.
///
/// Edits may come in any order; all ranges refer to `text` as given. Edits that overlap are rejected, and
/// nothing is applied. Edits that only touch are fine: a replacement may end where the next one starts, and
/// several insertions at one offset are inserted in the order given.
///
/// ```
/// use kt_syntax::{TextEdit, apply_edits};
///
/// let edits = [TextEdit::replace(4..5, "y"), TextEdit::insert(0, "const ")];
/// assert_eq!(apply_edits("val x = 1", edits).unwrap(), "const val y = 1");
/// ```
pub fn apply_edits(text: &str, edits: impl IntoIterator<Item = TextEdit>) -> Result<String, EditError> {
    let mut edits: Vec<TextEdit> = edits.into_iter().collect();
    for edit in &edits {
        let Range { start, end } = edit.range;
        if start > end || !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            return Err(EditError::InvalidRange { range: edit.range.clone() });
        }
    }
    edits.sort_by_key(|edit| (edit.range.start, edit.range.end));
    if let Some(pair) = edits.windows(2).find(|pair| pair[1].range.start < pair[0].range.end) {
        return Err(EditError::Overlapping { first: pair[0].range.clone(), second: pair[1].range.clone() });
    }
    let added: usize = edits.iter().map(|edit| edit.text.len()).sum();
    let mut out = String::with_capacity(text.len() + added);
    let mut copied = 0;
    for edit in &edits {
        out.push_str(&text[copied..edit.range.start]);
        out.push_str(&edit.text);
        copied = edit.range.end;
    }
    out.push_str(&text[copied..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "fun f() = 1";

    #[test]
    fn no_edits_is_the_identity() {
        assert_eq!(apply_edits(TEXT, []).unwrap(), TEXT);
        assert_eq!(apply_edits("", []).unwrap(), "");
    }

    #[test]
    fn edits_apply_in_any_order() {
        let edits = [TextEdit::replace(10..11, "2"), TextEdit::delete(0..4), TextEdit::insert(6, "x: Int")];
        assert_eq!(apply_edits(TEXT, edits).unwrap(), "f(x: Int) = 2");
    }

    #[test]
    fn touching_edits_are_allowed() {
        let edits = [TextEdit::replace(4..5, "g"), TextEdit::replace(5..7, "<T>()"), TextEdit::insert(4, "T.")];
        assert_eq!(apply_edits(TEXT, edits).unwrap(), "fun T.g<T>() = 1");
        assert_eq!(apply_edits(TEXT, [TextEdit::insert(TEXT.len(), "\n")]).unwrap(), "fun f() = 1\n");
    }

    #[test]
    fn insertions_at_one_offset_keep_their_order() {
        let edits = [TextEdit::insert(0, "a"), TextEdit::insert(0, "b"), TextEdit::insert(0, "c")];
        assert_eq!(apply_edits("-", edits).unwrap(), "abc-");
    }

    #[test]
    fn overlapping_edits_are_rejected() {
        let overlap = apply_edits(TEXT, [TextEdit::delete(4..7), TextEdit::replace(0..5, "val x")]);
        assert_eq!(overlap, Err(EditError::Overlapping { first: 0..5, second: 4..7 }));
        let same = apply_edits(TEXT, [TextEdit::replace(4..5, "a"), TextEdit::replace(4..5, "b")]);
        assert_eq!(same, Err(EditError::Overlapping { first: 4..5, second: 4..5 }));
        let inside = apply_edits(TEXT, [TextEdit::delete(0..7), TextEdit::insert(3, "x")]);
        assert_eq!(inside, Err(EditError::Overlapping { first: 0..7, second: 3..3 }));
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn invalid_ranges_are_rejected() {
        assert_eq!(apply_edits(TEXT, [TextEdit::delete(5..50)]), Err(EditError::InvalidRange { range: 5..50 }));
        assert_eq!(apply_edits(TEXT, [TextEdit::delete(5..4)]), Err(EditError::InvalidRange { range: 5..4 }));
        assert_eq!(apply_edits("π", [TextEdit::insert(1, "x")]), Err(EditError::InvalidRange { range: 1..1 }));
    }
}
