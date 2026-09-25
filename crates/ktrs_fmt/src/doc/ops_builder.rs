//! Port of `OpsBuilder.java`; `actualSize`/`actualStartColumn`, the break overloads and `build`
//! are in `ops_builder_positions`, `ops_builder_breaks` and `ops_builder_build`.
//!
//! Where Java throws `FormattingError`, the builder records the first error and keeps going;
//! [`OpsBuilder::build`] returns it. Formatting output is discarded on error either way.

use std::rc::Rc;

use super::blank_line_wanted::BlankLineWanted;
use super::doc_leaves::{DocToken, RealOrImaginary};
use super::formatting_error::{FormatterDiagnostic, FormattingError};
use super::indent::Indent;
use super::input::{Input, Tok, Token};
use super::op::Op;
use super::output::Output;

pub struct OpsBuilder<'a> {
    pub(super) input: &'a dyn Input,
    pub(super) ops: Vec<Op>,
    pub(super) output: &'a mut dyn Output,
    token_i: usize,
    input_position: i32,
    /// The number of unclosed open ops in the input stream.
    depth: i32,
    last_partial_format_boundary: i32,
    pub(super) error: Option<FormattingError>,
}

impl<'a> OpsBuilder<'a> {
    /// Add an `Op`, and record open/close ops for later validation of unclosed levels.
    pub fn add(&mut self, op: Op) {
        match op {
            Op::Open(_) => self.depth += 1,
            Op::Close => {
                self.depth -= 1;
                assert!(self.depth >= 0, "unbalanced close");
            }
            _ => {}
        }
        self.ops.push(op);
    }

    pub fn add_all(&mut self, ops: Vec<Op>) {
        for op in ops {
            self.add(op);
        }
    }

    /// `output` is used here only to record blank-line and partial-format information.
    pub fn new(input: &'a dyn Input, output: &'a mut dyn Output) -> OpsBuilder<'a> {
        OpsBuilder {
            input,
            ops: Vec::new(),
            output,
            token_i: 0,
            input_position: i32::MIN,
            depth: 0,
            last_partial_format_boundary: -1,
            error: None,
        }
    }

    pub fn get_input(&self) -> &'a dyn Input {
        self.input
    }

    pub fn depth(&self) -> i32 {
        self.depth
    }

    /// The first `FormattingError` Java would have thrown, if any.
    pub fn error(&self) -> Option<&FormattingError> {
        self.error.as_ref()
    }

    /// Records `error` unless an earlier one was recorded (Java would have thrown that one).
    pub fn fail(&mut self, error: FormattingError) {
        self.error.get_or_insert(error);
    }

    /// Checks that all open ops in the op stream have matching close ops.
    pub fn check_closed(&mut self, previous: i32) {
        if self.depth != previous {
            let diagnostic = self.diagnostic(format!("saw {} unclosed ops", self.depth));
            self.fail(FormattingError::new(diagnostic));
        }
    }

    /// Create a `FormatterDiagnostic` at the current position.
    pub fn diagnostic(&self, message: String) -> FormatterDiagnostic {
        self.input.create_diagnostic(self.input_position, message)
    }

    /// Sync to position in the input; complains if any input token was skipped.
    pub fn sync(&mut self, input_position: i32) {
        if input_position > self.input_position {
            let tokens = self.input.get_tokens();
            self.input_position = input_position;
            if self.token_i < tokens.len()
                && input_position > tokens[self.token_i].get_tok().get_position()
            {
                // Found a missing input token. Insert it and mark it missing (usually not good).
                let token = &tokens[self.token_i];
                self.token_i += 1;
                let diagnostic = self.diagnostic(format!(
                    "did not generate token \"{}\"",
                    token.get_tok().get_text()
                ));
                self.fail(FormattingError::new(diagnostic));
            }
        }
    }

    /// Output any remaining tokens from the input stream (e.g. terminal whitespace).
    pub fn drain(&mut self) {
        let input_position = self.input.get_text().len() as i32 + 1;
        if input_position > self.input_position {
            let tokens = self.input.get_tokens();
            while self.token_i < tokens.len()
                && input_position > tokens[self.token_i].get_tok().get_position()
            {
                let token = tokens[self.token_i].clone();
                self.token_i += 1;
                self.add(DocToken::make(
                    token,
                    RealOrImaginary::Imaginary,
                    Indent::ZERO,
                    None,
                ));
            }
        }
        self.input_position = input_position;
        self.check_closed(0);
    }

    /// Open a new level by emitting an `OpenOp`.
    pub fn open(&mut self, plus_indent: Indent) {
        self.add(Op::Open(plus_indent));
    }

    /// Close the current level, by emitting a `CloseOp`.
    pub fn close(&mut self) {
        self.add(Op::Close);
    }

    /// Return the text of the next input token, or `None` if there is none.
    pub fn peek_token(&self) -> Option<&'a str> {
        self.peek_token_skip(0)
    }

    /// Return the text of an upcoming input token, or `None` if there is none.
    pub fn peek_token_skip(&self, skip: usize) -> Option<&'a str> {
        let tokens: &'a [Rc<dyn Token>] = self.input.get_tokens();
        tokens
            .get(self.token_i + skip)
            .map(|t| t.get_tok().get_original_text())
    }

    /// The toks starting at the current source position that satisfy `predicate`.
    pub fn peek_tokens(
        &self,
        start_position: i32,
        predicate: impl Fn(&dyn Tok) -> bool,
    ) -> Vec<Rc<dyn Tok>> {
        let tokens = self.input.get_tokens();
        assert!(
            tokens[self.token_i].get_tok().get_position() == start_position,
            "Expected the current token to be at position {start_position}, found: {:?}",
            tokens[self.token_i]
        );
        tokens[self.token_i..]
            .iter()
            .map(|t| t.get_tok())
            .take_while(|tok| predicate(&***tok))
            .cloned()
            .collect()
    }

    /// Emit an optional token iff it exists on the input.
    pub fn guess_token(&mut self, token: &str) {
        self.token(token, RealOrImaginary::Imaginary, Indent::ZERO, None);
    }

    pub fn token(
        &mut self,
        token: &str,
        real_or_imaginary: RealOrImaginary,
        plus_indent_comments_before: Indent,
        break_and_indent_trailing_comment: Option<Indent>,
    ) {
        let tokens = self.input.get_tokens();
        if self.peek_token() == Some(token) {
            // Found the input token. Output it.
            let input_token = tokens[self.token_i].clone();
            self.token_i += 1;
            self.add(DocToken::make(
                input_token,
                RealOrImaginary::Real,
                plus_indent_comments_before,
                break_and_indent_trailing_comment,
            ));
        } else if real_or_imaginary.is_real() {
            // A "bad" token, which doesn't exist on the input.
            let message = format!(
                "expected token: '{}'; generated {} instead",
                self.peek_token().unwrap_or("null"),
                token
            );
            let diagnostic = self.diagnostic(message);
            self.fail(FormattingError::new(diagnostic));
        }
    }

    /// Emit a single- or multi-character op by breaking it into single-character tokens.
    pub fn op(&mut self, op: &str) {
        for c in op.chars() {
            self.token(
                c.encode_utf8(&mut [0; 4]),
                RealOrImaginary::Real,
                Indent::ZERO,
                None,
            );
        }
    }

    /// Mark the boundary of a partially formattable region: `[[b0, b1), [b1, b2), ...]`.
    pub fn mark_for_partial_format(&mut self) {
        if self.last_partial_format_boundary == -1 {
            self.last_partial_format_boundary = self.token_i as i32;
            return;
        }
        if self.token_i as i32 == self.last_partial_format_boundary {
            return;
        }
        let tokens = self.input.get_tokens();
        let start = &tokens[self.last_partial_format_boundary as usize];
        let end = &tokens[self.token_i - 1];
        self.output.mark_for_partial_format(&**start, &**end);
        self.last_partial_format_boundary = self.token_i as i32;
    }

    /// Force or suppress a blank line here in the output.
    pub fn blank_line_wanted(&mut self, wanted: BlankLineWanted) {
        match self.input.get_tokens().get(self.token_i) {
            Some(token) => self.output.blank_line(Self::get_i(&**token), wanted),
            None => {
                let diagnostic = self.diagnostic(format!(
                    "Index {} out of bounds for blankLineWanted",
                    self.token_i
                ));
                self.fail(FormattingError::new(diagnostic));
            }
        }
    }

    fn get_i(token: &dyn Token) -> i32 {
        token
            .get_toks_before()
            .iter()
            .map(|tok| tok.get_index())
            .find(|&i| i >= 0)
            .unwrap_or(token.get_tok().get_index())
    }
}
