//! Port of `KDocWriter.kt` (unused upstream; kept for parity).

use super::kdoc_token::{KDocToken, KDocTokenType};
use super::kstring::{KChar, KString, to_string, w};

/// The kind of whitespace requested between the previous and next tokens, lowest priority first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestedWhitespace {
    None,
    /// Add one space, only if the next token seems like a word.
    ConditionalWhitespace,
    /// Add one space, e.g. " "
    Whitespace,
    /// Break to the next line
    Newline,
    /// Add a whole blank line between the two lines of content
    BlankLine,
}

/// Stateful object that accepts "requests" and "writes," producing formatted Javadoc.
pub struct KDocWriter {
    max_line_length: i32,
    output: KString,
    block_indent: KString,
    remaining_on_line: i32,
    at_start_of_line: bool,
    in_code_block: bool,
    requested_whitespace: RequestedWhitespace,
}

/// Tokens that are always pinned to the following token, e.g. `<p>` in `<p>Foo bar`.
const START_OF_LINE_TOKENS: [KDocTokenType; 3] =
    [KDocTokenType::ListItemOpenTag, KDocTokenType::ParagraphOpenTag, KDocTokenType::HeaderOpenTag];

impl KDocWriter {
    pub fn new(block_indent_count: i32, max_line_length: i32) -> Self {
        KDocWriter {
            max_line_length,
            output: KString::new(),
            block_indent: vec![' ' as u16; (block_indent_count + 1) as usize],
            remaining_on_line: 0,
            at_start_of_line: false,
            in_code_block: false,
            requested_whitespace: RequestedWhitespace::None,
        }
    }

    pub fn request_whitespace(&mut self) {
        self.request_whitespace_kind(RequestedWhitespace::Whitespace);
    }

    pub fn write_begin_javadoc(&mut self) {
        self.append_tracking_length(w!("/**"));
    }

    pub fn write_end_javadoc(&mut self) {
        self.request_close_code_block_marker();
        self.output.push('\n' as u16);
        let indent = self.block_indent.clone();
        self.append_tracking_length(&indent);
        self.append_tracking_length(w!("*/"));
    }

    pub fn write_list_item_open(&mut self, token: &KDocToken) {
        self.request_close_code_block_marker();
        self.request_newline();
        self.write_token(token);
    }

    pub fn write_pre_open(&mut self, token: &KDocToken) {
        self.request_blank_line();
        self.write_token(token);
    }

    pub fn write_pre_close(&mut self, token: &KDocToken) {
        self.write_token(token);
        self.request_blank_line();
    }

    pub fn write_code_open(&mut self, token: &KDocToken) {
        self.write_token(token);
    }

    pub fn write_code_close(&mut self, token: &KDocToken) {
        self.write_token(token);
    }

    pub fn write_table_open(&mut self, token: &KDocToken) {
        self.request_blank_line();
        self.write_token(token);
    }

    pub fn write_table_close(&mut self, token: &KDocToken) {
        self.write_token(token);
        self.request_blank_line();
    }

    pub fn write_tag(&mut self, token: &KDocToken) {
        self.request_newline();
        self.write_token(token);
    }

    pub fn write_code_line(&mut self, token: &KDocToken) {
        self.request_open_code_block_marker();
        self.request_newline();
        if !token.value.is_empty() {
            self.write_token(token);
        }
    }

    /// Adds a code block marker if we are in a code block currently
    fn request_close_code_block_marker(&mut self) {
        if self.in_code_block {
            self.requested_whitespace = RequestedWhitespace::Newline;
            self.write_explicit_code_block_marker(&KDocToken::new(KDocTokenType::CodeBlockMarker, w!("```").to_vec()));
        }
    }

    /// Adds a code block marker if we are not in a code block currently
    fn request_open_code_block_marker(&mut self) {
        if !self.in_code_block {
            self.requested_whitespace = RequestedWhitespace::Newline;
            self.write_explicit_code_block_marker(&KDocToken::new(KDocTokenType::CodeBlockMarker, w!("```").to_vec()));
        }
    }

    pub fn write_explicit_code_block_marker(&mut self, token: &KDocToken) {
        self.request_newline();
        self.write_token(token);
        self.request_newline();
        self.in_code_block = !self.in_code_block;
    }

    pub fn write_literal(&mut self, token: &KDocToken) {
        self.request_close_code_block_marker();
        self.write_token(token);
    }

    pub fn write_markdown_link(&mut self, token: &KDocToken) {
        self.write_token(token);
    }

    pub fn request_blank_line(&mut self) {
        self.request_whitespace_kind(RequestedWhitespace::BlankLine);
    }

    pub fn request_newline(&mut self) {
        self.request_whitespace_kind(RequestedWhitespace::Newline);
    }

    fn request_whitespace_kind(&mut self, requested_whitespace: RequestedWhitespace) {
        self.requested_whitespace = requested_whitespace.max(self.requested_whitespace);
    }

    fn write_token(&mut self, token: &KDocToken) {
        if self.requested_whitespace == RequestedWhitespace::BlankLine {
            self.write_blank_line();
            self.requested_whitespace = RequestedWhitespace::None;
        } else if self.requested_whitespace == RequestedWhitespace::Newline {
            self.write_newline();
            self.requested_whitespace = RequestedWhitespace::None;
        }

        let need_whitespace = match self.requested_whitespace {
            RequestedWhitespace::Whitespace => true,
            RequestedWhitespace::ConditionalWhitespace => {
                token.value.first().expect("NoSuchElementException").is_letter_or_digit()
            }
            _ => false,
        };
        // Break to respect the line limit, unless at line start (where it wouldn't help).
        if !self.at_start_of_line && token.length() + i32::from(need_whitespace) > self.remaining_on_line {
            self.write_newline();
        }
        if !self.at_start_of_line && need_whitespace {
            self.append_tracking_length(w!(" "));
        }

        self.append_tracking_length(&token.value);
        self.requested_whitespace = RequestedWhitespace::None;

        if !START_OF_LINE_TOKENS.contains(&token.token_type) {
            self.at_start_of_line = false;
        }
    }

    fn write_blank_line(&mut self) {
        self.output.push('\n' as u16);
        let indent = self.block_indent.clone();
        self.append_tracking_length(&indent);
        self.append_tracking_length(w!("*"));
        self.write_newline();
    }

    fn write_newline(&mut self) {
        self.output.push('\n' as u16);
        self.remaining_on_line = self.max_line_length;
        let indent = self.block_indent.clone();
        self.append_tracking_length(&indent);
        self.append_tracking_length(w!("* "));
        self.at_start_of_line = true;
    }

    /// Tracks width in UTF-16 units, like upstream (see its TODO about graphemes).
    fn append_tracking_length(&mut self, s: &[u16]) {
        self.output.extend_from_slice(s);
        self.remaining_on_line -= s.len() as i32;
    }
}

impl std::fmt::Display for KDocWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&to_string(&self.output))
    }
}
