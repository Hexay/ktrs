/// A zero-based line and column.
///
/// Lines end at `\n` (the text of a [`SourceFile`](crate::SourceFile) has no `\r\n`). The unit of `col` depends
/// on the method that produced or consumes the value: UTF-8 bytes, or UTF-16 code units for the `_utf16`
/// methods (what LSP and most editors count).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LineCol {
    /// Zero-based line.
    pub line: u32,
    /// Zero-based column within the line.
    pub col: u32,
}

#[derive(Debug)]
pub(crate) struct LineIndex {
    /// Offset of the first byte of each line; `line_starts[0] == 0`.
    line_starts: Vec<u32>,
}

impl LineIndex {
    pub(crate) fn new(text: &str) -> LineIndex {
        const LOW: u64 = 0x0101_0101_0101_0101;
        let bytes = text.as_bytes();
        let mut line_starts = vec![0];
        let (chunks, rest) = bytes.as_chunks::<8>();
        let mut base = 0;
        for chunk in chunks {
            // Zero bytes of `word` are newlines; the bit trick may also flag a byte after one, hence the check.
            let word = u64::from_le_bytes(*chunk) ^ (LOW * b'\n' as u64);
            let mut hits = word.wrapping_sub(LOW) & !word & (LOW << 7);
            while hits != 0 {
                let i = hits.trailing_zeros() as usize / 8;
                if chunk[i] == b'\n' {
                    line_starts.push((base + i + 1) as u32);
                }
                hits &= hits - 1;
            }
            base += 8;
        }
        line_starts.extend(rest.iter().enumerate().filter(|&(_, &b)| b == b'\n').map(|(i, _)| (base + i + 1) as u32));
        LineIndex { line_starts }
    }

    pub(crate) fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// The line's byte range, without its `\n`.
    pub(crate) fn line_range(&self, text: &str, line: u32) -> Option<std::ops::Range<usize>> {
        let start = *self.line_starts.get(line as usize)? as usize;
        let end = self.line_starts.get(line as usize + 1).map_or(text.len(), |&next| next as usize - 1);
        Some(start..end)
    }

    pub(crate) fn line_col(&self, text: &str, offset: usize) -> Option<LineCol> {
        if offset > text.len() {
            return None;
        }
        let line = self.line_starts.partition_point(|&start| start as usize <= offset) - 1;
        Some(LineCol { line: line as u32, col: (offset - self.line_starts[line] as usize) as u32 })
    }

    pub(crate) fn line_col_utf16(&self, text: &str, offset: usize) -> Option<LineCol> {
        let at = self.line_col(text, offset)?;
        let line_start = offset - at.col as usize;
        let before = text.get(line_start..offset)?;
        Some(LineCol { line: at.line, col: before.encode_utf16().count() as u32 })
    }

    pub(crate) fn offset(&self, text: &str, pos: LineCol) -> Option<usize> {
        let line = self.line_range(text, pos.line)?;
        let offset = line.start + pos.col as usize;
        (offset <= line.end).then_some(offset)
    }

    pub(crate) fn offset_utf16(&self, text: &str, pos: LineCol) -> Option<usize> {
        let line = self.line_range(text, pos.line)?;
        let mut units = 0;
        for (i, c) in text[line.clone()].char_indices() {
            if units >= pos.col {
                return (units == pos.col).then_some(line.start + i);
            }
            units += c.len_utf16() as u32;
        }
        (units == pos.col).then_some(line.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "ab\n\nπ𝄞x\nlast";

    fn lc(line: u32, col: u32) -> LineCol {
        LineCol { line, col }
    }

    #[test]
    fn byte_columns_round_trip_at_every_offset() {
        let index = LineIndex::new(TEXT);
        assert_eq!(index.line_count(), 4);
        for offset in 0..=TEXT.len() {
            let at = index.line_col(TEXT, offset).unwrap();
            assert_eq!(index.offset(TEXT, at), Some(offset));
        }
        assert_eq!(index.line_col(TEXT, 0), Some(lc(0, 0)));
        assert_eq!(index.line_col(TEXT, 2), Some(lc(0, 2)));
        assert_eq!(index.line_col(TEXT, 3), Some(lc(1, 0)));
        assert_eq!(index.line_col(TEXT, 4), Some(lc(2, 0)));
        assert_eq!(index.line_col(TEXT, TEXT.len()), Some(lc(3, 4)));
        assert_eq!(index.line_col(TEXT, TEXT.len() + 1), None);
    }

    #[test]
    fn positions_outside_the_text_have_no_offset() {
        let index = LineIndex::new(TEXT);
        assert_eq!(index.offset(TEXT, lc(0, 3)), None);
        assert_eq!(index.offset(TEXT, lc(4, 0)), None);
        assert_eq!(index.offset(TEXT, lc(1, 0)), Some(3));
        assert_eq!(index.offset_utf16(TEXT, lc(1, 1)), None);
        assert_eq!(index.offset_utf16(TEXT, lc(9, 0)), None);
    }

    #[test]
    fn utf16_columns_count_code_units() {
        let index = LineIndex::new(TEXT);
        let line = TEXT.find('π').unwrap();
        let (pi, clef) = ('π'.len_utf8(), '𝄞'.len_utf8());
        assert_eq!(index.line_col_utf16(TEXT, line), Some(lc(2, 0)));
        assert_eq!(index.line_col_utf16(TEXT, line + pi), Some(lc(2, 1)));
        assert_eq!(index.line_col_utf16(TEXT, line + pi + clef), Some(lc(2, 3)));
        assert_eq!(index.line_col_utf16(TEXT, line + pi + clef + 1), Some(lc(2, 4)));
        assert_eq!(index.line_col_utf16(TEXT, line + 1), None, "inside a character");
        assert_eq!(index.offset_utf16(TEXT, lc(2, 3)), Some(line + pi + clef));
        assert_eq!(index.offset_utf16(TEXT, lc(2, 2)), None, "inside a surrogate pair");
        assert_eq!(index.offset_utf16(TEXT, lc(2, 4)), Some(line + pi + clef + 1));
        assert_eq!(index.offset_utf16(TEXT, lc(2, 5)), None);
    }

    #[test]
    fn line_starts_match_a_byte_by_byte_scan() {
        let texts = ["\n", "\n\n\n\n\n\n\n\n\n", "\n\u{b}\n\u{b}\u{b}\n\u{1}\n\u{b}", "abcdefg\nabcdefgh\n\u{b}abcdefghi\n\nx", "é\né\n\u{b}é\n𝄞\n\u{b}\n"];
        for text in texts {
            for skip in 0..text.len() {
                let Some(text) = text.get(skip..) else { continue };
                let expected: Vec<u32> = std::iter::once(0).chain(text.bytes().enumerate().filter(|&(_, b)| b == b'\n').map(|(i, _)| i as u32 + 1)).collect();
                assert_eq!(LineIndex::new(text).line_starts, expected, "{text:?}");
            }
        }
    }

    #[test]
    fn empty_text_has_one_line() {
        let index = LineIndex::new("");
        assert_eq!(index.line_count(), 1);
        assert_eq!(index.line_col("", 0), Some(lc(0, 0)));
        assert_eq!(index.offset("", lc(0, 0)), Some(0));
        assert_eq!(index.line_range("", 0), Some(0..0));
    }
}
