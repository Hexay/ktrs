//! Port of `java/JavaOutput.java` (the replacement computation is in `java_output_replacements`).
//!
//! Throughout, `i` indexes input lines, `j` output lines, `ij` either, and `k` toks.

use super::blank_line_wanted::BlankLineWanted;
use super::comments_helper::CommentsHelper;
use super::input::{Input, Tok, Token};
use super::input_output::InputOutput;
use super::newlines;
use super::output::Output;
use super::range::{EMPTY_RANGE, Range, RangeSet};

pub struct JavaOutput<'a> {
    pub(super) io: InputOutput,
    pub(super) line_separator: String,
    /// Used to follow along while emitting the output.
    pub(super) java_input: &'a dyn Input,
    /// Used to re-flow comments.
    comments_helper: Box<dyn CommentsHelper + 'a>,
    /// Indexed by k, in place of upstream's `Map<Integer, BlankLineWanted>`.
    blank_lines: Vec<Option<BlankLineWanted>>,
    pub(super) partial_format_ranges: RangeSet,
    mutable_lines: Vec<String>,
    /// The number of tokens or comments in the input, excluding the EOF.
    k_n: i32,
    /// Closest corresponding line number on input.
    i_line: i32,
    /// Last tok index output.
    last_k: i32,
    newlines_pending: i32,
    line_builder: String,
    spaces_pending: String,
}

impl<'a> JavaOutput<'a> {
    pub fn new(
        line_separator: &str,
        java_input: &'a dyn Input,
        comments_helper: Box<dyn CommentsHelper + 'a>,
    ) -> Self {
        JavaOutput {
            io: InputOutput::default(),
            line_separator: line_separator.to_string(),
            java_input,
            comments_helper,
            blank_lines: Vec::new(),
            partial_format_ranges: RangeSet::create(),
            mutable_lines: Vec::new(),
            k_n: java_input.get_kn(),
            i_line: 0,
            last_k: -1,
            newlines_pending: 0,
            line_builder: String::new(),
            spaces_pending: String::new(),
        }
    }

    pub fn input_output(&self) -> &InputOutput {
        &self.io
    }

    /// Flush any incomplete last line, then add the EOF token into our data structures.
    pub fn flush(&mut self) {
        // Guava's CharMatcher.whitespace() is exactly Unicode White_Space, like char::is_whitespace.
        if !self.line_builder.chars().all(char::is_whitespace) {
            self.mutable_lines.push(self.line_builder.clone());
        }
        let j_n = self.mutable_lines.len();
        let eof_range = Range::closed_open(self.k_n, self.k_n + 1);
        while self.io.ranges.len() < j_n {
            self.io.ranges.push(EMPTY_RANGE);
        }
        self.io.ranges.push(eof_range);
        self.io.set_lines(std::mem::take(&mut self.mutable_lines));
    }

    /// The earliest position of any Tok in the Token, including leading whitespace.
    pub fn start_position(token: &dyn Token) -> i32 {
        let mut min = token.get_tok().get_position();
        for tok in token.get_toks_before() {
            min = min.min(tok.get_position());
        }
        min
    }

    /// The earliest non-whitespace Tok in the Token.
    pub fn start_tok(token: &dyn Token) -> &dyn Tok {
        for tok in token.get_toks_before() {
            if tok.get_index() >= 0 {
                return &**tok;
            }
        }
        &**token.get_tok()
    }

    /// The last non-whitespace Tok in the Token.
    pub fn end_tok(token: &dyn Token) -> &dyn Tok {
        for tok in token.get_toks_after().iter().rev() {
            if tok.get_index() >= 0 {
                return &**tok;
            }
        }
        &**token.get_tok()
    }

    fn is_comment(text: &str) -> bool {
        text.starts_with("//") || text.starts_with("/*")
    }

    fn union(x: Range, y: Range) -> Range {
        if x.is_empty() {
            y
        } else if y.is_empty() {
            x
        } else {
            x.span(y)
        }
    }

    fn emit_pending_newlines(&mut self) {
        while self.newlines_pending > 0 {
            // drop leading blank lines
            if !self.mutable_lines.is_empty() || !self.line_builder.is_empty() {
                self.mutable_lines
                    .push(std::mem::take(&mut self.line_builder));
            }
            self.line_builder.clear();
            self.newlines_pending -= 1;
        }
    }
}

impl Output for JavaOutput<'_> {
    fn blank_line(&mut self, k: i32, wanted: BlankLineWanted) {
        let k = usize::try_from(k).expect("blank line at a negative tok index");
        if self.blank_lines.len() <= k {
            self.blank_lines.resize(k + 1, None);
        }
        let merged = match self.blank_lines[k].take() {
            Some(existing) => existing.merge(wanted),
            None => wanted,
        };
        self.blank_lines[k] = Some(merged);
    }

    fn mark_for_partial_format(&mut self, start: &dyn Token, end: &dyn Token) {
        let lo = Self::start_tok(start).get_index();
        let hi = Self::end_tok(end).get_index();
        self.partial_format_ranges.add_closed(lo, hi);
    }

    fn append(&mut self, text: &str, range: Range) {
        if !range.is_empty() {
            let mut saw_newlines = false;
            // Skip over input line we've passed.
            let input = self.java_input.input_output();
            let i_n = input.get_line_count();
            while self.i_line < i_n
                && (input.get_ranges(self.i_line).is_empty()
                    || input.get_ranges(self.i_line).upper_endpoint() <= range.lower_endpoint())
            {
                if input.get_ranges(self.i_line).is_empty() {
                    // Skipped over a blank line.
                    saw_newlines = true;
                }
                self.i_line += 1;
            }
            // Output blank line if we've called OpsBuilder.blankLine(true) here, or if there's a
            // blank line here and it's a comment.
            let wanted = usize::try_from(self.last_k)
                .ok()
                .and_then(|k| self.blank_lines.get(k)?.as_ref())
                .map_or(Some(false), BlankLineWanted::wanted);
            if (saw_newlines && Self::is_comment(text)) || wanted.unwrap_or(saw_newlines) {
                self.newlines_pending += 1;
            }
        }
        if newlines::is_newline(text) {
            // Don't update range information, and swallow extra newlines. The case below for '\n'
            // is for block comments.
            if self.newlines_pending == 0 {
                self.newlines_pending += 1;
            }
            self.spaces_pending.clear();
        } else {
            let mut ranges_set = false;
            let mut chars = text.chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    ' ' => self.spaces_pending.push(' '),
                    '\t' => self.spaces_pending.push('\t'),
                    '\r' | '\n' => {
                        if c == '\r' && chars.peek() == Some(&'\n') {
                            chars.next();
                        }
                        self.spaces_pending.clear();
                        self.newlines_pending += 1;
                    }
                    _ => {
                        if self.newlines_pending > 0 {
                            ranges_set = false;
                        }
                        self.emit_pending_newlines();
                        if !self.spaces_pending.is_empty() {
                            self.line_builder.push_str(&self.spaces_pending);
                            self.spaces_pending.clear();
                        }
                        self.line_builder.push(c);
                        if !range.is_empty() && !ranges_set {
                            let j = self.mutable_lines.len();
                            while self.io.ranges.len() <= j {
                                self.io.ranges.push(EMPTY_RANGE);
                            }
                            self.io.ranges[j] = Self::union(self.io.ranges[j], range);
                            ranges_set = true;
                        }
                    }
                }
            }
        }
        if !range.is_empty() {
            self.last_k = range.upper_endpoint();
        }
    }

    fn indent(&mut self, indent: i32) {
        for _ in 0..indent {
            self.spaces_pending.push(' ');
        }
    }

    fn get_comments_helper(&self) -> &dyn CommentsHelper {
        &*self.comments_helper
    }
}
