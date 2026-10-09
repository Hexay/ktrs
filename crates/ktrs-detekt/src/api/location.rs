//! `detekt-api/.../Location.kt`.

use std::fmt;
use std::path::PathBuf;

use ktrs_psi::PsiElement;

use crate::kt_file::{self, FileContext, LineAndColumn};

/// Specifies a position within a source code fragment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub source: SourceLocation,
    pub end_source: SourceLocation,
    pub text: TextLocation,
    pub path: PathBuf,
}

impl Location {
    /// `Location.from(element, offset)`; `offset` in bytes.
    pub fn from(element: &PsiElement, offset: usize) -> Location {
        let file = kt_file::containing_file(element);
        let start = start_line_and_column(&file, element, offset);
        let source_location = SourceLocation::new(start.line, start.column);
        let end = end_line_and_column(&file, element, offset);
        let end_source_location = SourceLocation::new(end.line, end.column);
        let text_location =
            TextLocation::new(file.utf16_offset(element.start_offset() + offset), file.utf16_offset(element.end_offset() + offset));
        Location { source: source_location, end_source: end_source_location, text: text_location, path: file.absolute_path().to_owned() }
    }

    /// `Location(source, endSource, text, path)` over the byte range `start..end` of `file`
    /// (`getLineAndColumnRangeInPsiFile(file, TextRange(start, end))`).
    pub fn of_range(file: &FileContext, start: usize, end: usize) -> Location {
        let (source, end_source) = (file.line_and_column(start), file.line_and_column(end));
        Location {
            source: SourceLocation::new(source.line, source.column),
            end_source: SourceLocation::new(end_source.line, end_source.column),
            text: TextLocation::new(file.utf16_offset(start), file.utf16_offset(end)),
            path: file.absolute_path().to_owned(),
        }
    }
}

fn start_line_and_column(file: &FileContext, element: &PsiElement, offset: usize) -> LineAndColumn {
    line_and_column(file, element.start_offset() + offset)
}

fn end_line_and_column(file: &FileContext, element: &PsiElement, offset: usize) -> LineAndColumn {
    line_and_column(file, element.end_offset() + offset)
}

fn line_and_column(file: &FileContext, offset: usize) -> LineAndColumn {
    if !file.text().is_empty() { file.line_and_column(offset) } else { LineAndColumn { line: 1, column: 1 } }
}

/// Stores line and column information of a location.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> SourceLocation {
        assert!(line > 0, "The source location line must be greater than 0");
        assert!(column > 0, "The source location column must be greater than 0");
        SourceLocation { line, column }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Stores character start and end positions of a text file, in UTF-16 units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextLocation {
    pub start: usize,
    pub end: usize,
}

impl TextLocation {
    pub fn new(start: usize, end: usize) -> TextLocation {
        assert!(end >= start, "end must be greater than or equal to start");
        TextLocation { start, end }
    }
}

impl fmt::Display for TextLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.start, self.end)
    }
}
