//! Port of `Input.java`. Its nested `Tok`/`Token` interfaces are concrete structs: their only
//! implementations are ktfmt's `KotlinTok`/`KotlinToken` (built in `format::input`), and concrete
//! types let the doc engine's per-tok calls inline. The `kind` field is omitted: ktfmt always sets
//! it to `KtTokens.EOF`.

use std::collections::HashMap;
use std::fmt;

use super::formatting_error::FormatterDiagnostic;
use super::input_output::InputOutput;
use super::newlines;
use super::range_map::RangeMap;
use super::utf16::utf16_len;

/// `Input.Tok` (ktfmt's `KotlinTok`): a token, comment, or whitespace/newline run. `text` is a
/// slice of the file text.
#[derive(Clone, PartialEq, Eq)]
pub struct Tok<'s> {
    index: i32,
    /// `originalText` where it differs from `text` (string templates with tombstones).
    original_text: Option<Box<str>>,
    text: &'s str,
    position: i32,
    column: i32,
    pub is_token: bool,
}

impl<'s> Tok<'s> {
    /// `KotlinTok(index, originalText, text, position, column, isToken)`, `originalText` defaulting
    /// to `text`.
    pub fn new(
        index: i32,
        original_text: Option<String>,
        text: &'s str,
        position: i32,
        column: i32,
        is_token: bool,
    ) -> Self {
        let original_text = original_text.filter(|o| o != text).map(String::into_boxed_str);
        Tok { index, original_text, text, position, column, is_token }
    }

    /// Index among the numbered toks (tokens and comments), or `-1` for whitespace.
    #[inline]
    pub fn get_index(&self) -> i32 {
        self.index
    }

    /// Byte offset in the input.
    #[inline]
    pub fn get_position(&self) -> i32 {
        self.position
    }

    #[inline]
    pub fn get_column(&self) -> i32 {
        self.column
    }

    #[inline]
    pub fn get_text(&self) -> &'s str {
        self.text
    }

    #[inline]
    pub fn get_original_text(&self) -> &str {
        self.original_text.as_deref().unwrap_or(self.text)
    }

    /// `getOriginalText().length()` in UTF-16 units; use `get_original_text().len()` for
    /// position arithmetic.
    pub fn length(&self) -> i32 {
        utf16_len(self.get_original_text())
    }

    pub fn is_newline(&self) -> bool {
        newlines::is_newline(self.text)
    }

    #[inline]
    pub fn is_slash_slash_comment(&self) -> bool {
        self.text.starts_with("//")
    }

    #[inline]
    pub fn is_slash_star_comment(&self) -> bool {
        self.text.starts_with("/*")
    }

    pub fn is_javadoc_comment(&self) -> bool {
        self.text.starts_with("/**") && utf16_len(self.text) > 4
    }

    #[inline]
    pub fn is_comment(&self) -> bool {
        self.is_slash_slash_comment() || self.is_slash_star_comment()
    }
}

impl fmt::Debug for Tok<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KotlinTok")
            .field("index", &self.index)
            .field("original_text", &self.get_original_text())
            .field("text", &self.text)
            .field("position", &self.position)
            .field("column", &self.column)
            .field("is_token", &self.is_token)
            .finish()
    }
}

/// `Input.Token` (ktfmt's `KotlinToken`): a real token with its attached non-tokens.
/// `toksBefore`, the tok and `toksAfter` share one allocation.
pub struct Token<'s> {
    /// `toksBefore ++ [kotlinTok] ++ toksAfter`.
    toks: Box<[Tok<'s>]>,
    /// Where `kotlinTok` is in `toks`.
    tok_i: usize,
}

impl<'s> Token<'s> {
    pub fn new(toks: Box<[Tok<'s>]>, tok_i: usize) -> Self {
        assert!(tok_i < toks.len());
        Token { toks, tok_i }
    }

    #[inline]
    pub fn get_tok(&self) -> &Tok<'s> {
        &self.toks[self.tok_i]
    }

    #[inline]
    pub fn get_toks_before(&self) -> &[Tok<'s>] {
        &self.toks[..self.tok_i]
    }

    #[inline]
    pub fn get_toks_after(&self) -> &[Tok<'s>] {
        &self.toks[self.tok_i + 1..]
    }

    /// The toks before, the tok, and the toks after, in order.
    pub fn toks(&self) -> &[Tok<'s>] {
        &self.toks
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KotlinToken")
            .field("toks_before", &self.get_toks_before())
            .field("kotlin_tok", self.get_tok())
            .field("toks_after", &self.get_toks_after())
            .finish()
    }
}

pub trait Input {
    /// The `InputOutput` superclass state (lines and per-line tok ranges).
    fn input_output(&self) -> &InputOutput;

    fn get_tokens(&self) -> &[Token<'_>];

    /// Values are indices into `get_tokens()`.
    fn get_position_token_map(&self) -> &RangeMap<usize>;

    fn get_position_to_column_map(&self) -> &HashMap<i32, i32>;

    fn get_text(&self) -> &str;

    fn get_kn(&self) -> i32;

    fn get_token(&self, k: i32) -> Option<&Token<'_>>;

    fn get_line_number(&self, input_position: i32) -> i32;

    fn get_column_number(&self, input_position: i32) -> i32;

    fn create_diagnostic(&self, input_position: i32, message: String) -> FormatterDiagnostic {
        FormatterDiagnostic::create_at(
            self.get_line_number(input_position),
            self.get_column_number(input_position),
            message,
        )
    }
}
