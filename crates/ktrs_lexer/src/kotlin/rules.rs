//! Rules of the inclusive states (`YYINITIAL`, `LONG_TEMPLATE_ENTRY`, `UNMATCHED_BACKTICK`),
//! dispatched on the first byte. Where rules share a first byte, longest match wins and ties go to
//! the earlier rule, exactly as in the flex spec.

use ktrs_syntax::SyntaxKind::{self, *};

use super::scan::{
    char_at, character_literal, double_literal, identifier, integer_literal, rest_of_line, run_of,
    white_space,
};
use super::{KotlinLexer, LexState};
use crate::chars::is_kotlin_identifier_part;

/// First entry whose text prefixes the input wins, so longer operators come first.
fn operator(text: &str, start: usize, table: &[(&str, SyntaxKind)]) -> (SyntaxKind, usize) {
    let rest = &text[start..];
    table
        .iter()
        .find(|(op, _)| rest.starts_with(op))
        .map_or((BAD_CHARACTER, start + 1), |&(op, kind)| {
            (kind, start + op.len())
        })
}

impl KotlinLexer<'_> {
    pub(super) fn lex_default(&mut self, start: usize) -> Option<SyntaxKind> {
        let text = self.text;
        let (kind, end) = match self.byte(start) {
            b'"' => {
                self.push_interpolation_prefix(1);
                self.pos = start;
                return None;
            }
            b'$' => {
                let run_end = run_of(text, start, b'$');
                if self.byte(run_end) == b'"' {
                    self.push_interpolation_prefix((run_end - start) as i32);
                    return self.token(INTERPOLATION_PREFIX, run_end);
                }
                match identifier(text, start + 1).filter(|_| run_end == start + 1) {
                    Some(end) => (FIELD_IDENTIFIER, end),
                    None => (BAD_CHARACTER, start + 1),
                }
            }
            b'{' if self.state == LexState::LongTemplateEntry => {
                self.l_brace_count += 1;
                (LBRACE, start + 1)
            }
            b'}' if self.state == LexState::LongTemplateEntry => {
                if self.l_brace_count == 0 {
                    self.pop_state();
                    (LONG_TEMPLATE_ENTRY_END, start + 1)
                } else {
                    self.l_brace_count -= 1;
                    (RBRACE, start + 1)
                }
            }
            b'/' => match self.byte(start + 1) {
                b'*' if text[start..].starts_with("/**/") => (BLOCK_COMMENT, start + 4),
                b'*' => {
                    let doc = self.byte(start + 2) == b'*';
                    self.push_state(if doc {
                        LexState::DocComment
                    } else {
                        LexState::BlockComment
                    });
                    self.comment_depth = 0;
                    self.comment_start = start;
                    self.pos = start + 2 + usize::from(doc);
                    return None;
                }
                b'/' => (EOL_COMMENT, rest_of_line(text, start)),
                b'=' => (DIVEQ, start + 2),
                _ => (DIV, start + 1),
            },
            b' ' | b'\n' | b'\t' | b'\x0c' => (WHITE_SPACE, white_space(text, start)),
            // `zzCurrentPos == 0` fails when the scan hit the buffer end (the refill path moves
            // zzCurrentPos), so a shebang only counts when a `\n` follows it.
            b'#' if self.byte(start + 1) == b'!'
                && start == 0
                && rest_of_line(text, start) < text.len() =>
            {
                (SHEBANG_COMMENT, rest_of_line(text, start))
            }
            b'0'..=b'9' => {
                let int_end = integer_literal(text, start);
                // `{INTEGER_LITERAL}\.\.` counts the dots for longest match, then pushes them back.
                let int_match = if text[int_end..].starts_with("..") {
                    int_end + 2
                } else {
                    int_end
                };
                match double_literal(text, start) {
                    Some(end) if end > int_match => (FLOAT_LITERAL, end),
                    _ => (INTEGER_LITERAL, int_end),
                }
            }
            b'.' => match double_literal(text, start) {
                Some(end) => (FLOAT_LITERAL, end),
                None => operator(
                    text,
                    start,
                    &[
                        ("...", RESERVED),
                        ("..<", RANGE_UNTIL),
                        ("..", RANGE),
                        (".", DOT),
                    ],
                ),
            },
            b'\'' => (CHARACTER_LITERAL, character_literal(text, start)),
            b'`' => match identifier(text, start) {
                Some(end) => (IDENTIFIER, end),
                None => {
                    self.push_state(LexState::UnmatchedBacktick);
                    (BAD_CHARACTER, start + 1)
                }
            },
            b'!' => self.lex_excl(start),
            b'=' => operator(
                text,
                start,
                &[
                    ("===", EQEQEQ),
                    ("==", EQEQ),
                    ("=>", DOUBLE_ARROW),
                    ("=", EQ),
                ],
            ),
            b'<' => operator(text, start, &[("<=", LTEQ), ("<", LT)]),
            b'>' => operator(text, start, &[(">=", GTEQ), (">", GT)]),
            b'+' => operator(
                text,
                start,
                &[("++", PLUSPLUS), ("+=", PLUSEQ), ("+", PLUS)],
            ),
            b'-' => operator(
                text,
                start,
                &[
                    ("--", MINUSMINUS),
                    ("-=", MINUSEQ),
                    ("->", ARROW),
                    ("-", MINUS),
                ],
            ),
            b'*' => operator(text, start, &[("*=", MULTEQ), ("*", MUL)]),
            b'%' => operator(text, start, &[("%=", PERCEQ), ("%", PERC)]),
            b'&' => operator(text, start, &[("&&", ANDAND), ("&", AND)]),
            b'|' => operator(text, start, &[("||", OROR)]),
            b':' => operator(text, start, &[("::", COLONCOLON), (":", COLON)]),
            b';' => operator(text, start, &[(";;", DOUBLE_SEMICOLON), (";", SEMICOLON)]),
            b'[' => (LBRACKET, start + 1),
            b']' => (RBRACKET, start + 1),
            b'{' => (LBRACE, start + 1),
            b'}' => (RBRACE, start + 1),
            b'(' => (LPAR, start + 1),
            b')' => (RPAR, start + 1),
            b'?' => (QUEST, start + 1),
            b',' => (COMMA, start + 1),
            b'#' => (HASH, start + 1),
            b'@' => (AT, start + 1),
            _ => self.lex_identifier_or_bad_character(start),
        };
        self.token(kind, end)
    }

    /// `\!in{IDENTIFIER_PART}` / `\!is{IDENTIFIER_PART}` beat `"!in"` / `"!is"`.
    fn lex_excl(&self, start: usize) -> (SyntaxKind, usize) {
        let rest = &self.text[start + 1..];
        let not_kind = if rest.starts_with("in") {
            Some(NOT_IN)
        } else if rest.starts_with("is") {
            Some(NOT_IS)
        } else {
            None
        };
        if let Some(not_kind) = not_kind {
            return match char_at(self.text, start + 3) {
                // yypushback(3) counts UTF-16 units, so a supplementary char keeps the `i` in the token.
                Some(c) if is_kotlin_identifier_part(c) => {
                    (EXCL, start + if c.len_utf16() == 2 { 2 } else { 1 })
                }
                _ => (not_kind, start + 3),
            };
        }
        operator(
            self.text,
            start,
            &[("!==", EXCLEQEQEQ), ("!=", EXCLEQ), ("!", EXCL)],
        )
    }

    fn lex_identifier_or_bad_character(&self, start: usize) -> (SyntaxKind, usize) {
        let text = self.text;
        // Never at a backtick (see `lex_default`), so `identifier` matches iff `c` is a letter.
        let Some(end) = identifier(text, start) else {
            let c = char_at(text, start).unwrap_or('\0');
            return (BAD_CHARACTER, start + c.len_utf8());
        };
        let word = &text[start..end];
        if word == "as" && self.byte(end) == b'?' {
            return (AS_SAFE, end + 1);
        }
        (SyntaxKind::hard_keyword(word).unwrap_or(IDENTIFIER), end)
    }
}
