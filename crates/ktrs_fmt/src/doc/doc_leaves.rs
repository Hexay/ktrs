//! The leaf classes of `Doc.java`: `Doc.Token` ([`DocToken`]), `Doc.Space` ([`Op::Space`]),
//! `Doc.Break` ([`DocBreak`]) and `Doc.Tok` ([`DocTok`]).

use std::rc::Rc;

use super::comments_helper::{CommentsHelper, reformat_parameter_comment};
use super::doc::{FillMode, MAX_LINE_WIDTH, State};
use super::indent::Indent;
use super::input::{Tok, Token};
use super::newlines;
use super::op::Op;
use super::output::{BreakTag, Output};
use super::range::{EMPTY_RANGE, Range};
use super::utf16::utf16_len;

/// Is a Token a real token, or imaginary (e.g., a token generated incorrectly, or an EOF)?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealOrImaginary {
    Real,
    Imaginary,
}

impl RealOrImaginary {
    pub fn is_real(self) -> bool {
        self == RealOrImaginary::Real
    }
}

/// A leaf `Doc` for a token.
#[derive(Debug)]
pub struct DocToken {
    token: Rc<dyn Token>,
    real_or_imaginary: RealOrImaginary,
    plus_indent_comments_before: Indent,
    break_and_indent_trailing_comment: Option<Indent>,
}

impl DocToken {
    fn tok(&self) -> &dyn Tok {
        &**self.token.get_tok()
    }

    /// How much extra to indent comments before the token.
    pub fn get_plus_indent_comments_before(&self) -> &Indent {
        &self.plus_indent_comments_before
    }

    /// Force a line break and indent trailing javadoc or block comments.
    pub fn break_and_indent_trailing_comment(&self) -> Option<&Indent> {
        self.break_and_indent_trailing_comment.as_ref()
    }

    pub fn make(
        token: Rc<dyn Token>,
        real_or_imaginary: RealOrImaginary,
        plus_indent_comments_before: Indent,
        break_and_indent_trailing_comment: Option<Indent>,
    ) -> Op {
        Op::Token(DocToken {
            token,
            real_or_imaginary,
            plus_indent_comments_before,
            break_and_indent_trailing_comment,
        })
    }

    pub fn get_token(&self) -> &Rc<dyn Token> {
        &self.token
    }

    pub fn real_or_imaginary(&self) -> RealOrImaginary {
        self.real_or_imaginary
    }

    pub(crate) fn compute_width(&self) -> i32 {
        let idx = newlines::first_break(self.tok().get_original_text());
        if idx >= 0 {
            MAX_LINE_WIDTH
        } else {
            self.tok().length()
        }
    }

    pub(crate) fn compute_flat(&self) -> String {
        self.tok().get_original_text().to_string()
    }

    pub(crate) fn compute_range(&self) -> Range {
        Range::singleton(self.tok().get_index())
    }

    pub(crate) fn compute_breaks(&self, state: State) -> State {
        state.with_column(state.column + self.compute_width())
    }

    pub(crate) fn write(&self, output: &mut dyn Output, range: Range) {
        output.append(self.tok().get_original_text(), range);
    }
}

/// A leaf node in a `Doc` for an optional break.
#[derive(Debug)]
pub struct DocBreak {
    fill_mode: FillMode,
    flat: String,
    plus_indent: Indent,
    opt_tag: Option<BreakTag>,
    /// Was this break taken?
    broken: bool,
    /// New indent after this break.
    new_indent: i32,
}

impl DocBreak {
    pub fn make(fill_mode: FillMode, flat: &str, plus_indent: Indent) -> DocBreak {
        DocBreak::make_tagged(fill_mode, flat, plus_indent, None)
    }

    pub fn make_tagged(
        fill_mode: FillMode,
        flat: &str,
        plus_indent: Indent,
        opt_tag: Option<BreakTag>,
    ) -> DocBreak {
        DocBreak {
            fill_mode,
            flat: flat.to_string(),
            plus_indent,
            opt_tag,
            broken: false,
            new_indent: 0,
        }
    }

    pub fn make_forced() -> DocBreak {
        DocBreak::make(FillMode::Forced, "", Indent::ZERO)
    }

    pub fn get_plus_indent(&self) -> i32 {
        self.plus_indent.eval()
    }

    pub fn is_forced(&self) -> bool {
        self.fill_mode == FillMode::Forced
    }

    pub fn fill_mode(&self) -> FillMode {
        self.fill_mode
    }

    pub fn flat(&self) -> &str {
        &self.flat
    }

    pub(crate) fn compute_width(&self) -> i32 {
        if self.is_forced() {
            MAX_LINE_WIDTH
        } else {
            utf16_len(&self.flat)
        }
    }

    pub(crate) fn compute_flat(&self) -> String {
        self.flat.clone()
    }

    /// `computeBreaks(State, int lastIndent, boolean broken)`.
    pub(crate) fn compute_breaks_taken(
        &mut self,
        state: State,
        last_indent: i32,
        broken: bool,
    ) -> State {
        if let Some(tag) = &self.opt_tag {
            tag.record_broken(broken);
        }

        if broken {
            self.broken = true;
            self.new_indent = (last_indent + self.plus_indent.eval()).max(0);
            state.with_column(self.new_indent)
        } else {
            self.broken = false;
            self.new_indent = -1;
            state.with_column(state.column + utf16_len(&self.flat))
        }
    }

    pub(crate) fn write(&self, output: &mut dyn Output, range: Range) {
        if self.broken {
            output.append("\n", EMPTY_RANGE);
            output.indent(self.new_indent);
        } else {
            output.append(&self.flat, range);
        }
    }
}

/// A leaf node in a `Doc` for a non-token.
#[derive(Debug)]
pub struct DocTok {
    tok: Rc<dyn Tok>,
    text: Option<String>,
}

impl DocTok {
    pub fn make(tok: Rc<dyn Tok>) -> DocTok {
        DocTok { tok, text: None }
    }

    pub fn tok(&self) -> &Rc<dyn Tok> {
        &self.tok
    }

    pub(crate) fn compute_width(&self) -> i32 {
        let tok = &*self.tok;
        let original = tok.get_original_text();
        let idx = newlines::first_break(original);
        // only count the first line of multi-line block comments
        if tok.is_comment() {
            if idx > 0 {
                return utf16_len(&original[..idx as usize]);
            } else if tok.is_slash_slash_comment() && !original.starts_with("// ") {
                // Account for line comments with missing spaces, see compute_flat.
                return tok.length() + 1;
            } else {
                return reformat_parameter_comment(tok).map_or(tok.length(), |s| utf16_len(&s));
            }
        }
        if idx != -1 {
            MAX_LINE_WIDTH
        } else {
            tok.length()
        }
    }

    pub(crate) fn compute_flat(&self) -> String {
        let tok = &*self.tok;
        let original = tok.get_original_text();
        if tok.is_slash_slash_comment() && !original.starts_with("// ") {
            return format!("// {}", &original["//".len()..]);
        }
        reformat_parameter_comment(tok).unwrap_or_else(|| original.to_string())
    }

    pub(crate) fn compute_range(&self) -> Range {
        Range::singleton(self.tok.get_index())
    }

    pub(crate) fn compute_breaks(
        &mut self,
        comments_helper: &dyn CommentsHelper,
        max_width: i32,
        state: State,
    ) -> State {
        let text = comments_helper.rewrite(&*self.tok, max_width, state.column);
        let last_line_start = newlines::line_offset_iterator(&text).last().unwrap_or(0);
        let first_line_length = utf16_len(&text[last_line_start..]);
        self.text = Some(text);
        state.with_column(state.column + first_line_length)
    }

    pub(crate) fn write(&self, output: &mut dyn Output, range: Range) {
        output.append(
            self.text.as_deref().expect("write before compute_breaks"),
            range,
        );
    }
}
