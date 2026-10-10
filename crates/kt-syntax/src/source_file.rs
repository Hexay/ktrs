use std::borrow::Cow;
use std::fmt;
use std::ops::Range;
use std::rc::Rc;

use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::{ElementId, SyntaxKind as Raw, Tree};

use crate::line_index::LineIndex;
use crate::{EditError, LineCol, Node, ParseError, SyntaxError, TextEdit, guard};

/// Parses a Kotlin source file (`.kt`).
///
/// Syntax errors do not fail the parse: the tree covers the whole text and holds them as nodes
/// ([`SourceFile::errors`]). `Err` is for input no tree is built for; see [`ParseError`].
///
/// A leading byte order mark is dropped and `\r\n` becomes `\n` before parsing, as the Kotlin compiler's own
/// tooling does. **Every offset of the result refers to [`SourceFile::text`]**, the normalized text, not to the
/// argument. A lone `\r` is kept.
pub fn parse(text: &str) -> Result<SourceFile, ParseError> {
    parse_as(text, FileKind::Source)
}

/// Parses a Kotlin script (`.kts`), where statements are allowed at the top level. Otherwise as [`parse`].
pub fn parse_script(text: &str) -> Result<SourceFile, ParseError> {
    parse_as(text, FileKind::Script)
}

fn parse_as(text: &str, kind: FileKind) -> Result<SourceFile, ParseError> {
    let without_bom = text.strip_prefix('\u{feff}');
    let had_bom = without_bom.is_some();
    let text = without_bom.unwrap_or(text);
    let had_crlf = text.contains("\r\n");
    let text = if had_crlf { Cow::Owned(text.replace("\r\n", "\n")) } else { Cow::Borrowed(text) };
    if text.len() >= u32::MAX as usize {
        return Err(ParseError::TooLarge { len: text.len() });
    }
    let depth = guard::nesting_depth(&text)?;
    let parsed = guard::with_parser_stack(depth, text.len(), || parse_file(&text, kind));
    if let Some(missed) = parsed.first_missed_tokens() {
        return Err(ParseError::TokensNotInserted { tokens: missed.tokens.iter().map(|t| t.to_string()).collect() });
    }
    let tree = Rc::unwrap_or_clone(parsed.tree);
    let error_nodes = tree.find_kinds(Tree::ROOT, [Raw::ERROR_ELEMENT], []);
    let errors: Vec<(ElementId, Box<str>)> = error_nodes.zip(parsed.error_messages.into_iter().map(String::into_boxed_str)).collect();
    let line_index = LineIndex::new(tree.text());
    Ok(SourceFile { tree, errors, line_index, script: kind == FileKind::Script, had_bom, had_crlf })
}

/// A parsed file: the text and its syntax tree.
///
/// The tree is the one the Kotlin compiler builds for the same text (same kinds, same nesting, same error
/// elements), and it is lossless: the tokens, in order, spell [`text`](SourceFile::text) exactly, white space and
/// comments included.
///
/// A `SourceFile` is immutable, `Send` and `Sync`. [`Node`]s borrow it.
pub struct SourceFile {
    tree: Tree,
    /// `ERROR_ELEMENT` ids, ascending, with their messages.
    errors: Vec<(ElementId, Box<str>)>,
    // Built with the tree, not on first use: a `OnceLock` here would make `Node` a key with interior mutability.
    line_index: LineIndex,
    script: bool,
    had_bom: bool,
    had_crlf: bool,
}

impl SourceFile {
    pub(crate) fn tree(&self) -> &Tree {
        &self.tree
    }

    /// The text the tree spells and all offsets refer to: the input without a byte order mark, with `\r\n`
    /// replaced by `\n`.
    pub fn text(&self) -> &str {
        self.tree.text()
    }

    /// The root node, of kind `KT_FILE`. Its range is the whole text.
    pub fn root(&self) -> Node<'_> {
        Node::new(self, Tree::ROOT)
    }

    /// Whether this was parsed with [`parse_script`].
    pub fn is_script(&self) -> bool {
        self.script
    }

    /// Whether the input started with a byte order mark (not part of [`text`](SourceFile::text)).
    pub fn had_bom(&self) -> bool {
        self.had_bom
    }

    /// Whether the input had `\r\n` line breaks ([`text`](SourceFile::text) has `\n`). A tool that writes edited
    /// text back can use this to restore them.
    pub fn had_crlf(&self) -> bool {
        self.had_crlf
    }

    /// Whether the tree has syntax errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// The syntax errors, in source order.
    pub fn errors(&self) -> impl ExactSizeIterator<Item = SyntaxError<'_>> + DoubleEndedIterator {
        self.errors.iter().map(|(id, message)| SyntaxError { node: Node::new(self, *id), message })
    }

    pub(crate) fn error_message(&self, id: ElementId) -> Option<&str> {
        let at = self.errors.binary_search_by_key(&id, |&(error, _)| error).ok()?;
        Some(&self.errors[at].1)
    }

    /// The token that contains the byte at `offset`; `None` at or past the end of the text.
    pub fn token_at(&self, offset: usize) -> Option<Node<'_>> {
        if offset >= self.text().len() {
            return None;
        }
        // Starts never decrease in preorder, and the last element that starts at or before a byte is its token.
        let (mut low, mut high) = (0, self.tree.len() as ElementId);
        while low < high {
            let mid = low + (high - low) / 2;
            if usize::from(self.tree.text_range(mid).start()) <= offset { low = mid + 1 } else { high = mid }
        }
        Some(Node::new(self, low - 1))
    }

    /// The smallest node or token whose range contains `range`; `None` if `range` is not within the text.
    /// For an empty range that is the token starting there (the root at the end of the text).
    pub fn covering(&self, range: Range<usize>) -> Option<Node<'_>> {
        if range.start > range.end || range.end > self.text().len() {
            return None;
        }
        let token = self.token_at(range.start).unwrap_or(self.root());
        std::iter::once(token).chain(token.ancestors()).find(|node| node.range().end >= range.end)
    }

    /// Number of lines; a text that ends in `\n` has an empty last line.
    pub fn line_count(&self) -> usize {
        self.line_index.line_count()
    }

    /// The byte range of a zero-based line, without its `\n`.
    pub fn line_range(&self, line: u32) -> Option<Range<usize>> {
        self.line_index.line_range(self.text(), line)
    }

    /// Line and column (in UTF-8 bytes) of a byte offset; `None` past the end of the text.
    pub fn line_col(&self, offset: usize) -> Option<LineCol> {
        self.line_index.line_col(self.text(), offset)
    }

    /// Line and column in UTF-16 code units; `None` past the end of the text or inside a character.
    pub fn line_col_utf16(&self, offset: usize) -> Option<LineCol> {
        self.line_index.line_col_utf16(self.text(), offset)
    }

    /// The byte offset of a line and column in UTF-8 bytes; `None` if the line does not exist or is shorter.
    pub fn offset(&self, pos: LineCol) -> Option<usize> {
        self.line_index.offset(self.text(), pos)
    }

    /// The byte offset of a line and column in UTF-16 code units; `None` if there is no such position.
    pub fn offset_utf16(&self, pos: LineCol) -> Option<usize> {
        self.line_index.offset_utf16(self.text(), pos)
    }

    /// [`apply_edits`](crate::apply_edits) on [`text`](SourceFile::text).
    pub fn apply_edits(&self, edits: impl IntoIterator<Item = TextEdit>) -> Result<String, EditError> {
        crate::apply_edits(self.text(), edits)
    }
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceFile").field("len", &self.text().len()).field("errors", &self.errors.len()).finish_non_exhaustive()
    }
}
