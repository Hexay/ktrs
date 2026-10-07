//! LSP positions (0-based line, UTF-16 column) over a document's text, and text replacements as edits.

use lsp_types::{Position, Range, TextEdit};

/// Line starts of a text; lines end at `\n` (a `\r` before it counts as part of the line's content).
pub(crate) struct LineIndex<'a> {
    text: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub(crate) fn new(text: &'a str) -> LineIndex<'a> {
        let line_starts = std::iter::once(0).chain(text.match_indices('\n').map(|(i, _)| i + 1)).collect();
        LineIndex { text, line_starts }
    }

    pub(crate) fn position(&self, offset: usize) -> Position {
        let line = self.line_starts.partition_point(|&start| start <= offset) - 1;
        let character = self.text[self.line_starts[line]..offset].encode_utf16().count();
        Position::new(line as u32, character as u32)
    }

    /// The byte offset of `position`, clamped to its line's end (before the `\n`) and to the text's end.
    pub(crate) fn offset(&self, position: Position) -> usize {
        let Some(&start) = self.line_starts.get(position.line as usize) else {
            return self.text.len();
        };
        let end = self.line_end(start);
        let mut units = 0;
        for (i, c) in self.text[start..end].char_indices() {
            if units >= position.character as usize {
                return start + i;
            }
            units += c.len_utf16();
        }
        end
    }

    /// The offset of the `\n` ending the line that contains `offset` (or the text's end).
    pub(crate) fn line_end(&self, offset: usize) -> usize {
        self.text[offset..].find('\n').map_or(self.text.len(), |i| offset + i)
    }
}

/// The edits turning `old` into `new`: none when equal, else one replacing the lines between their common
/// leading and trailing lines.
pub(crate) fn text_edits(old: &str, new: &str) -> Vec<TextEdit> {
    if old == new {
        return Vec::new();
    }
    let (old_lines, new_lines): (Vec<&str>, Vec<&str>) = (old.split_inclusive('\n').collect(), new.split_inclusive('\n').collect());
    let prefix = old_lines.iter().zip(&new_lines).take_while(|(a, b)| a == b).count();
    let max_suffix = old_lines.len().min(new_lines.len()) - prefix;
    let suffix = old_lines.iter().rev().zip(new_lines.iter().rev()).take(max_suffix).take_while(|(a, b)| a == b).count();
    let start: usize = old_lines[..prefix].iter().map(|l| l.len()).sum();
    let old_end = old.len() - old_lines[old_lines.len() - suffix..].iter().map(|l| l.len()).sum::<usize>();
    let new_end = new.len() - new_lines[new_lines.len() - suffix..].iter().map(|l| l.len()).sum::<usize>();
    let index = LineIndex::new(old);
    let range = Range::new(index.position(start), index.position(old_end));
    vec![TextEdit::new(range, new[start..new_end].to_owned())]
}

/// `edits` applied to `text` (non-overlapping, any order).
pub fn apply_edits(text: &str, edits: &[TextEdit]) -> String {
    let index = LineIndex::new(text);
    let mut spans: Vec<(usize, usize, &str)> =
        edits.iter().map(|e| (index.offset(e.range.start), index.offset(e.range.end), e.new_text.as_str())).collect();
    spans.sort_by_key(|&(start, _, _)| std::cmp::Reverse(start));
    let mut result = text.to_owned();
    for (start, end, new_text) in spans {
        result.replace_range(start..end, new_text);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_count_utf16_units() {
        let index = LineIndex::new("a\n😀b\n");
        assert_eq!(index.position(7), Position::new(1, 3));
        assert_eq!(index.offset(Position::new(1, 2)), 6);
        assert_eq!(index.offset(Position::new(1, 9)), 7);
        assert_eq!(index.offset(Position::new(5, 0)), 8);
    }

    #[test]
    fn edits_replace_only_the_changed_lines() {
        for (old, new) in [("a\nb\nc\n", "a\nx\ny\nc\n"), ("a\nb", "a\nc"), ("a\n", "a\nb\n"), ("x\ny\n", ""), ("", "z")] {
            let edits = text_edits(old, new);
            assert_eq!(apply_edits(old, &edits), new, "{old:?} -> {new:?}");
        }
        assert_eq!(text_edits("a\nb\nc\n", "a\nx\nc\n")[0].range, Range::new(Position::new(1, 0), Position::new(2, 0)));
        assert!(text_edits("same", "same").is_empty());
    }
}
