//! Port of `KotlinCode.kt`: the source text the formatter works on, with its file type.
//! Character offsets are UTF-8 byte offsets into [`KotlinCode::code`] here (UTF-16 units upstream);
//! [`KotlinCode::utf16_ranges_to_char_ranges`] converts what callers hold in upstream's unit.

use std::fmt;
use std::path::Path;

use crate::doc::{Range, RangeSet, newlines};
use crate::kdoc::index_of_comment_escape_sequences;

use super::FormatError;
use super::input::ParseError;
use super::input::whitespace_tombstones::index_of_whitespace_tombstone;
use super::kotlin_text::{convert_line_separators, convert_line_separators_to};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileType {
    Regular,
    Script,
}

impl FileType {
    pub fn extension(self) -> &'static str {
        match self {
            FileType::Regular => "kt",
            FileType::Script => "kts",
        }
    }

    /// `File.kotlinFileType`; any other extension is upstream's `IllegalArgumentException`.
    pub fn of_file(file: &Path) -> Result<FileType, FormatError> {
        let name = file.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        let extension = name.rsplit_once('.').map_or("", |(_, extension)| extension);
        match extension {
            "kt" => Ok(FileType::Regular),
            "kts" => Ok(FileType::Script),
            _ => Err(FormatError::Runtime(format!("java.lang.IllegalArgumentException: Unsupported file type: {extension}"))),
        }
    }

    /// Not upstream, for callers that may have no path (editors, build tools, stdin): a `.kt` file is
    /// regular, anything else a script, as ktfmt's CLI treats stdin.
    pub fn of_file_or_script(file: Option<&Path>) -> FileType {
        file.and_then(|file| FileType::of_file(file).ok()).unwrap_or(FileType::Script)
    }
}

/// Represents a Kotlin source code file for the formatter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KotlinCode {
    /// Source code with normalized line separators (everything is `'\n'`).
    pub code: String,
    /// Whether this is a script or a regular file for the parser.
    pub file_type: FileType,
    /// Original line separator used in the source code.
    pub line_separator: &'static str,
}

impl KotlinCode {
    pub fn from(code: &str, file_type: FileType) -> Result<KotlinCode, ParseError> {
        check_escape_sequences(code)?;
        let original_line_separator = newlines::guess_line_separator(code);

        let normalized_code = convert_line_separators(code);
        Ok(KotlinCode { code: normalized_code, file_type, line_separator: original_line_separator })
    }

    pub fn copy(&self, code: String) -> KotlinCode {
        KotlinCode { code, file_type: self.file_type, line_separator: self.line_separator }
    }

    /// Zero-indexed, closed-open line ranges to the character ranges of those lines (without the
    /// last line's terminator).
    pub fn line_ranges_to_char_ranges(&self, line_ranges: &RangeSet) -> RangeSet {
        let mut line_offsets: Vec<i32> = newlines::line_offset_iterator(&self.code).map(|it| it as i32).collect();
        line_offsets.push(self.code.len() as i32 + 1);

        let mut character_ranges = RangeSet::create();
        for line_range in line_ranges.sub_range_set(Range::closed_open(0, line_offsets.len() as i32 - 1)).as_canonical_ranges() {
            let line_start = line_offsets[line_range.lower_endpoint() as usize];
            let line_end = line_offsets[line_range.upper_endpoint() as usize] - 1;
            let character_range = Range::closed_open(line_start, line_end);
            if !character_range.is_empty() {
                character_ranges.add(character_range);
            }
        }
        character_ranges
    }

    /// Not upstream: closed-open ranges in UTF-16 units of [`Self::code`] as byte ranges. An endpoint
    /// past the end stays as far past it, so `offset + length` is "outside the file" for the same inputs.
    pub fn utf16_ranges_to_char_ranges(&self, utf16_ranges: &RangeSet) -> RangeSet {
        let mut character_ranges = RangeSet::create();
        for range in utf16_ranges.as_canonical_ranges() {
            let lower = self.utf16_offset_to_char_offset(range.lower_endpoint(), false);
            let upper = self.utf16_offset_to_char_offset(range.upper_endpoint(), true);
            character_ranges.add(Range::closed_open(lower, upper.max(lower)));
        }
        character_ranges
    }

    /// An offset inside a surrogate pair goes to the pair's start, or to its end with `round_up`.
    fn utf16_offset_to_char_offset(&self, utf16_offset: i32, round_up: bool) -> i32 {
        if utf16_offset <= 0 {
            return utf16_offset;
        }
        let mut units = 0;
        for (index, c) in self.code.char_indices() {
            if units >= utf16_offset {
                return index as i32;
            }
            units += c.len_utf16() as i32;
            if units > utf16_offset {
                return if round_up { (index + c.len_utf8()) as i32 } else { index as i32 };
            }
        }
        self.code.len() as i32 + (utf16_offset - units)
    }
}

/// `toString()`: the code with its original line separator.
impl fmt::Display for KotlinCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&convert_line_separators_to(&self.code, self.line_separator))
    }
}

fn check_escape_sequences(code: &str) -> Result<(), ParseError> {
    let mut index = index_of_whitespace_tombstone(code);
    if index == -1 {
        index = index_of_comment_escape_sequences(code);
    }
    if index != -1 {
        return Err(ParseError::at_offset(
            "ktfmt does not support code which contains one of {\\u0003, \\u0004, \\u0005} character; escape it",
            code,
            index as usize,
        ));
    }
    Ok(())
}
