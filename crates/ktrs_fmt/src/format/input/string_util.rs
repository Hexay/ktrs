//! IntelliJ `StringUtil.offsetToLineColumn`, which ktfmt uses for diagnostics.

use crate::doc::utf16_len;

/// 0-based `(line, column)` of byte `offset`; the column is in UTF-16 units. `None` past the end.
pub fn offset_to_line_column(text: &str, offset: usize) -> Option<(i32, i32)> {
    if offset > text.len() {
        return None;
    }
    let bytes = text.as_bytes();
    let mut cur_line = 0;
    let mut cur_line_start = 0;
    let mut cur_offset = 0;
    while cur_offset < offset {
        match bytes[cur_offset] {
            b'\n' => {
                cur_line += 1;
                cur_line_start = cur_offset + 1;
            }
            b'\r' => {
                cur_line += 1;
                if cur_offset + 1 < bytes.len() && bytes[cur_offset + 1] == b'\n' {
                    cur_offset += 1;
                }
                cur_line_start = cur_offset + 1;
            }
            _ => {}
        }
        cur_offset += 1;
    }
    // An offset inside "\r\n" yields column -1, as in IntelliJ.
    let column = if offset >= cur_line_start {
        utf16_len(&text[cur_line_start..offset])
    } else {
        offset as i32 - cur_line_start as i32
    };
    Some((cur_line, column))
}
