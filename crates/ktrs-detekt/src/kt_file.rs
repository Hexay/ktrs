//! What a `KtFile` carries upstream besides its tree: `name`, `virtualFilePath` and the IntelliJ `Document`
//! (line table). `ktrs_psi::KtFile` is only the tree, so the analyzer (or a test) registers the rest with
//! [`enter`] for the duration of one file's analysis, and code that upstream calls `containingFile`,
//! `KtFile.name`, `absolutePath()` or `PsiDiagnosticUtils` on reads it back with [`containing_file`].
//!
//! Offsets everywhere in this crate are UTF-8 byte offsets; [`FileContext::line_and_column`] and
//! [`FileContext::utf16_offset`] convert to the UTF-16 units detekt reports.

use std::cell::{OnceCell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ktrs_parser::{FileKind, parse_file};
use ktrs_psi::{KtFile, PsiElement};

use crate::visitor::SparseIndex;

pub struct FileContext {
    file: KtFile,
    path: PathBuf,
    lines: OnceCell<Lines>,
    sparse_index: SparseIndex,
}

struct Lines {
    starts: Vec<usize>,
    /// UTF-16 offset of each line start; `None` for all-ASCII text, where it equals `starts`.
    utf16_starts: Option<Vec<usize>>,
}

/// `PsiDiagnosticUtils.LineAndColumn`, both 1-based, the column in UTF-16 units.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LineAndColumn {
    pub line: usize,
    pub column: usize,
}

thread_local! {
    static CONTEXTS: RefCell<Vec<Rc<FileContext>>> = const { RefCell::new(Vec::new()) };
}

/// Pops its [`FileContext`] when dropped.
pub struct FileContextGuard(());

impl Drop for FileContextGuard {
    fn drop(&mut self) {
        CONTEXTS.with(|contexts| contexts.borrow_mut().pop());
    }
}

/// Registers `file` as living at `path` (its `virtualFilePath`) until the guard drops.
pub fn enter(file: &KtFile, path: impl Into<PathBuf>) -> FileContextGuard {
    let context =
        Rc::new(FileContext { file: file.clone(), path: path.into(), lines: OnceCell::new(), sparse_index: SparseIndex::default() });
    CONTEXTS.with(|contexts| contexts.borrow_mut().push(context));
    FileContextGuard(())
}

/// The context of `element.containingFile`. Panics outside [`enter`] for that file.
pub fn containing_file(element: &PsiElement) -> Rc<FileContext> {
    CONTEXTS.with(|contexts| {
        let contexts = contexts.borrow();
        let found = contexts.iter().rev().find(|c| std::ptr::eq(c.file.tree(), element.tree()));
        found.cloned().expect("no FileContext for this element's file: analyze it inside kt_file::enter")
    })
}

/// How detekt gets a file's text: line separators converted (`StringUtilRt.convertLineSeparators`: `\r\n` and
/// lone `\r` become `\n`) and a leading BOM dropped.
pub fn load_text(raw: &str) -> String {
    let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    if !raw.contains('\r') {
        return raw.to_owned();
    }
    raw.replace("\r\n", "\n").replace('\r', "\n")
}

/// Parses `text` (already through [`load_text`]) as the file `name` names: `.kts` is a script.
pub fn parse_kt_file(text: &str, name: &str) -> KtFile {
    KtFile::with_text(&parse_file(text, FileKind::from_file_name(name)), text)
}

impl FileContext {
    pub fn file(&self) -> &KtFile {
        &self.file
    }

    /// `KtFile.text`.
    pub fn text(&self) -> &str {
        self.file.tree().text()
    }

    /// `KtFile.name`.
    pub fn name(&self) -> String {
        self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
    }

    /// psi-utils `KtFile.absolutePath()`.
    pub fn absolute_path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn sparse_index(&self) -> &SparseIndex {
        &self.sparse_index
    }

    fn lines(&self) -> &Lines {
        self.lines.get_or_init(|| Lines::new(self.text()))
    }

    /// `Document.getLineNumber(offset)`, 0-based; the end of the text is on the last line.
    pub fn line_number(&self, offset: usize) -> usize {
        self.lines().starts.partition_point(|&start| start <= offset) - 1
    }

    /// `Document.getLineStartOffset(line)`.
    pub fn line_start_offset(&self, line: usize) -> usize {
        self.lines().starts[line]
    }

    /// Where the line after `line` starts; past any offset for the last line.
    pub fn next_line_start_offset(&self, line: usize) -> usize {
        self.lines().starts.get(line + 1).copied().unwrap_or(usize::MAX)
    }

    /// The UTF-16 offset of a byte offset.
    pub fn utf16_offset(&self, offset: usize) -> usize {
        let lines = self.lines();
        let Some(utf16_starts) = &lines.utf16_starts else { return offset };
        let line = self.line_number(offset);
        utf16_starts[line] + self.text()[lines.starts[line]..offset].encode_utf16().count()
    }

    /// `PsiDiagnosticUtils.offsetToLineAndColumn(document, offset)` for a non-empty document.
    pub fn line_and_column(&self, offset: usize) -> LineAndColumn {
        let line = self.line_number(offset);
        let line_start = self.line_start_offset(line);
        let column = match self.lines().utf16_starts {
            None => offset - line_start,
            Some(_) => self.text()[line_start..offset].encode_utf16().count(),
        };
        LineAndColumn { line: line + 1, column: column + 1 }
    }
}

impl Lines {
    fn new(text: &str) -> Lines {
        let mut starts = vec![0];
        starts.extend(text.bytes().enumerate().filter(|(_, b)| *b == b'\n').map(|(i, _)| i + 1));
        if text.is_ascii() {
            return Lines { starts, utf16_starts: None };
        }
        let mut utf16_starts = Vec::with_capacity(starts.len());
        let mut units = 0;
        for (line, &start) in starts.iter().enumerate() {
            utf16_starts.push(units);
            let end = starts.get(line + 1).copied().unwrap_or(text.len());
            units += text[start..end].encode_utf16().count();
        }
        Lines { starts, utf16_starts: Some(utf16_starts) }
    }
}
