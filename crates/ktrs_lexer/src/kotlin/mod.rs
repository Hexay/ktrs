//! Port of `Kotlin.flex` (`_JetLexer`). Each `lex_*` fn handles one JFlex lexical state: it sets
//! `pos` (zzMarkedPos) and returns the token kind, or `None` when the rule action does not return
//! (the scan then continues from `pos`, as in JFlex).

mod rules;
mod scan;

use ktrs_syntax::SyntaxKind::{self, *};
use scan::{escape_sequence, identifier, regular_string_part, run_of};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LexState {
    Yyinitial,
    StringPrefix,
    String,
    RawString,
    ShortTemplateEntry,
    BlockComment,
    DocComment,
    LongTemplateEntry,
    UnmatchedBacktick,
}

struct State {
    l_brace_count: i32,
    required_interpolation_prefix: i32,
    state: LexState,
}

pub(crate) struct KotlinLexer<'a> {
    text: &'a str,
    token_start: usize,
    pos: usize,
    state: LexState,
    states: Vec<State>,
    l_brace_count: i32,
    required_interpolation_prefix: i32,
    comment_start: usize,
    comment_depth: i32,
}

impl<'a> KotlinLexer<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        KotlinLexer {
            text,
            token_start: 0,
            pos: 0,
            state: LexState::Yyinitial,
            states: Vec::new(),
            l_brace_count: 0,
            required_interpolation_prefix: 0,
            comment_start: 0,
            comment_depth: 0,
        }
    }

    fn push_state(&mut self, state: LexState) {
        self.states.push(State {
            state: self.state,
            l_brace_count: self.l_brace_count,
            required_interpolation_prefix: self.required_interpolation_prefix,
        });
        self.l_brace_count = 0;
        self.required_interpolation_prefix = -1;
        self.state = state;
    }

    fn push_interpolation_prefix(&mut self, interpolation_prefix: i32) {
        self.states.push(State {
            state: self.state,
            l_brace_count: self.l_brace_count,
            required_interpolation_prefix: self.required_interpolation_prefix,
        });
        self.l_brace_count = 0;
        self.required_interpolation_prefix = interpolation_prefix;
        self.state = LexState::StringPrefix;
    }

    fn pop_state(&mut self) {
        // Every pop is paired with an earlier push, so the stack is never empty here.
        if let Some(state) = self.states.pop() {
            self.l_brace_count = state.l_brace_count;
            self.required_interpolation_prefix = state.required_interpolation_prefix;
            self.state = state.state;
        }
    }

    fn comment_state_to_token_type(state: LexState) -> SyntaxKind {
        match state {
            LexState::DocComment => DOC_COMMENT,
            _ => BLOCK_COMMENT,
        }
    }

    /// `advance()`: the next token as `(kind, start, end)` byte offsets, `None` at EOF.
    pub(crate) fn advance(&mut self) -> Option<(SyntaxKind, usize, usize)> {
        loop {
            self.token_start = self.pos;
            let start = self.pos;
            if start >= self.text.len() {
                return self.at_eof();
            }
            let kind = match self.state {
                LexState::StringPrefix => self.lex_string_prefix(start),
                LexState::String => self.lex_string(start),
                LexState::RawString => self.lex_raw_string(start),
                LexState::ShortTemplateEntry => self.lex_short_template_entry(start),
                LexState::BlockComment | LexState::DocComment => self.lex_comment(start),
                LexState::Yyinitial | LexState::LongTemplateEntry | LexState::UnmatchedBacktick => {
                    self.lex_default(start)
                }
            };
            if let Some(kind) = kind {
                return Some((kind, self.token_start, self.pos));
            }
        }
    }

    /// `<BLOCK_COMMENT, DOC_COMMENT> <<EOF>>`; every other state just ends.
    fn at_eof(&mut self) -> Option<(SyntaxKind, usize, usize)> {
        match self.state {
            LexState::BlockComment | LexState::DocComment => {
                let state = self.state;
                self.pop_state();
                Some((
                    Self::comment_state_to_token_type(state),
                    self.comment_start,
                    self.text.len(),
                ))
            }
            _ => None,
        }
    }

    fn byte(&self, pos: usize) -> u8 {
        self.text.as_bytes().get(pos).copied().unwrap_or(0)
    }

    fn token(&mut self, kind: SyntaxKind, end: usize) -> Option<SyntaxKind> {
        self.pos = end;
        Some(kind)
    }

    /// Error fallback of the exclusive states: one char of `BAD_CHARACTER`.
    fn bad_character(&mut self, start: usize) -> Option<SyntaxKind> {
        let len = scan::char_at(self.text, start).map_or(1, char::len_utf8);
        self.token(BAD_CHARACTER, start + len)
    }

    fn lex_string_prefix(&mut self, start: usize) -> Option<SyntaxKind> {
        if self.text[start..].starts_with("\"\"\"") {
            self.state = LexState::RawString;
            self.token(OPEN_QUOTE, start + 3)
        } else if self.byte(start) == b'"' {
            self.state = LexState::String;
            self.token(OPEN_QUOTE, start + 1)
        } else {
            self.bad_character(start)
        }
    }

    fn lex_string(&mut self, start: usize) -> Option<SyntaxKind> {
        match self.byte(start) {
            b'\n' => {
                self.pop_state();
                self.token(DANGLING_NEWLINE, start)
            }
            b'"' => {
                self.pop_state();
                self.token(CLOSING_QUOTE, start + 1)
            }
            b'\\' => match escape_sequence(self.text, start) {
                Some(end) => self.token(ESCAPE_SEQUENCE, end),
                None => self.token(BAD_CHARACTER, start + 1),
            },
            b'$' => self.lex_dollars(start),
            _ => self.token(REGULAR_STRING_PART, regular_string_part(self.text, start)),
        }
    }

    fn lex_raw_string(&mut self, start: usize) -> Option<SyntaxKind> {
        match self.byte(start) {
            b'\n' | b'\\' => self.token(REGULAR_STRING_PART, start + 1),
            b'"' => {
                let run = run_of(self.text, start, b'"') - start;
                if run == 3 {
                    self.pop_state();
                    self.token(CLOSING_QUOTE, start + 3)
                } else if run > 3 {
                    self.token(REGULAR_STRING_PART, start + run - 3)
                } else {
                    self.token(REGULAR_STRING_PART, start + 1)
                }
            }
            b'$' => self.lex_dollars(start),
            _ => self.token(REGULAR_STRING_PART, regular_string_part(self.text, start)),
        }
    }

    /// `{SHORT_TEMPLATE_ENTRY}`, `{LONELY_DOLLAR}` and `{LONG_TEMPLATE_ENTRY_START}` in (raw) strings.
    fn lex_dollars(&mut self, start: usize) -> Option<SyntaxKind> {
        let after = run_of(self.text, start, b'$');
        let prefix = (after - start) as i32;
        let required = self.required_interpolation_prefix;
        // Surplus leading dollars are plain text; the last `required` ones are re-lexed.
        let surplus_end = if prefix > required {
            after - required.max(0) as usize
        } else {
            after
        };
        if self.byte(after) == b'{' {
            if prefix == required {
                self.push_state(LexState::LongTemplateEntry);
                return self.token(LONG_TEMPLATE_ENTRY_START, after + 1);
            }
            return self.token(REGULAR_STRING_PART, surplus_end);
        }
        if identifier(self.text, after).is_some() {
            if prefix == required {
                self.push_state(LexState::ShortTemplateEntry);
                return self.token(SHORT_TEMPLATE_ENTRY_START, after);
            }
            return self.token(REGULAR_STRING_PART, surplus_end);
        }
        self.token(REGULAR_STRING_PART, after)
    }

    fn lex_short_template_entry(&mut self, start: usize) -> Option<SyntaxKind> {
        let Some(end) = identifier(self.text, start) else {
            return self.bad_character(start);
        };
        let kind = if &self.text[start..end] == "this" {
            THIS_KEYWORD
        } else {
            IDENTIFIER
        };
        self.pop_state();
        self.token(kind, end)
    }

    /// `<BLOCK_COMMENT, DOC_COMMENT>`: the non-returning rules are folded into one scan.
    fn lex_comment(&mut self, start: usize) -> Option<SyntaxKind> {
        let bytes = self.text.as_bytes();
        let mut p = start;
        while p < bytes.len() {
            match (bytes[p], bytes.get(p + 1)) {
                (b'/', Some(b'*')) => {
                    self.comment_depth += 1;
                    p += 2;
                }
                (b'*', Some(b'/')) => {
                    p += 2;
                    if self.comment_depth > 0 {
                        self.comment_depth -= 1;
                    } else {
                        let state = self.state;
                        self.pop_state();
                        self.token_start = self.comment_start;
                        return self.token(Self::comment_state_to_token_type(state), p);
                    }
                }
                _ => p += 1,
            }
        }
        self.pos = p;
        None
    }
}
