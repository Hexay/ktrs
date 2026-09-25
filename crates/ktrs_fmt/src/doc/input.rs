//! Port of `Input.java` (and its nested `Tok`/`Token` interfaces).

use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use super::formatting_error::FormatterDiagnostic;
use super::input_output::InputOutput;
use super::range_map::RangeMap;

/// `Input.Tok`: a token, comment, or whitespace/newline run.
pub trait Tok: Debug {
    /// Index among the numbered toks (tokens and comments), or `-1` for whitespace.
    fn get_index(&self) -> i32;

    /// Byte offset in the input.
    fn get_position(&self) -> i32;

    fn get_column(&self) -> i32;

    fn get_text(&self) -> &str;

    fn get_original_text(&self) -> &str;

    /// `getOriginalText().length()` in UTF-16 units; use `get_original_text().len()` for
    /// position arithmetic.
    fn length(&self) -> i32;

    fn is_newline(&self) -> bool;

    fn is_slash_slash_comment(&self) -> bool;

    fn is_slash_star_comment(&self) -> bool;

    fn is_javadoc_comment(&self) -> bool;

    fn is_comment(&self) -> bool;
}

/// `Input.Token`: a real token with its attached non-tokens.
pub trait Token: Debug {
    fn get_tok(&self) -> &Rc<dyn Tok>;

    fn get_toks_before(&self) -> &[Rc<dyn Tok>];

    fn get_toks_after(&self) -> &[Rc<dyn Tok>];
}

pub trait Input {
    /// The `InputOutput` superclass state (lines and per-line tok ranges).
    fn input_output(&self) -> &InputOutput;

    fn get_tokens(&self) -> &[Rc<dyn Token>];

    fn get_position_token_map(&self) -> &RangeMap<Rc<dyn Token>>;

    fn get_position_to_column_map(&self) -> &HashMap<i32, i32>;

    fn get_text(&self) -> &str;

    fn get_kn(&self) -> i32;

    fn get_token(&self, k: i32) -> Option<&Rc<dyn Token>>;

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
