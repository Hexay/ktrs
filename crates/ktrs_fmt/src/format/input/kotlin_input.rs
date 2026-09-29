//! Port of ktfmt's `KotlinInput.kt`: what `JavaInput` is for Java, with the Kotlin parse tree as
//! the lexer.
//!
//! The position-to-token and position-to-column maps are built on first use: ktfmt reads neither
//! (the formatter's one character-range lookup binary-searches the tokens instead).

use std::cell::OnceCell;
use std::collections::HashMap;
use std::iter::Peekable;
use std::rc::Rc;

use ktrs_psi::PsiElement;

use crate::doc::{
    EMPTY_RANGE, FormatterException, Input, InputOutput, JavaOutput, Range, RangeMap, RangeSet,
    Tok, Token, newlines,
};

use super::kotlin_tok::KotlinTok;
use super::kotlin_token::KotlinToken;
use super::parse_error::ParseError;
use super::string_util::offset_to_line_column;
use super::tokenizer::Tokenizer;

pub struct KotlinInput {
    io: InputOutput,
    text: Rc<str>,
    /// The Tokens for this input.
    tokens: Vec<Rc<dyn Token>>,
    /// Newline toks dropped after parameter comments: the toks not in any token.
    dropped_toks: Vec<Rc<KotlinTok>>,
    /// Map Tok position to column.
    position_to_column_map: OnceCell<HashMap<i32, i32>>,
    /// Map position to Token.
    position_token_map: OnceCell<RangeMap<Rc<dyn Token>>>,
    /// The number of numbered toks (tokens or comments), excluding the EOF.
    k_n: i32,
    /// Indices into `tokens`.
    k_to_token: Vec<Option<u32>>,
}

impl KotlinInput {
    pub fn new(text: &str, file: &PsiElement) -> Result<KotlinInput, ParseError> {
        let mut io = InputOutput::default();
        io.set_line_count(newlines::line_iterator(text).count());
        let (source, toks, k_n) = Self::build_toks(&mut io, file, text)?;
        let mut input = KotlinInput {
            io,
            text: source,
            tokens: Vec::with_capacity(toks.len() / 2),
            dropped_toks: Vec::new(),
            position_to_column_map: OnceCell::new(),
            position_token_map: OnceCell::new(),
            k_n,
            // adjust kN for EOF
            k_to_token: vec![None; k_n as usize + 1],
        };
        input.build_tokens(toks);
        Ok(input)
    }

    pub fn character_ranges_to_token_ranges(
        &self,
        character_ranges: &[Range],
    ) -> Result<RangeSet, FormatterException> {
        let mut token_range_set = RangeSet::create();
        for character_range in character_ranges {
            token_range_set.add(self.character_range_to_token_range(
                character_range.lower_endpoint(),
                character_range.upper_endpoint() - character_range.lower_endpoint(),
            )?);
        }
        Ok(token_range_set)
    }

    /// Convert from a byte offset and length pair to a 0-based token range.
    pub(crate) fn character_range_to_token_range(
        &self,
        offset: i32,
        length: i32,
    ) -> Result<Range, FormatterException> {
        let required_length = offset + length;
        if required_length > self.text.len() as i32 {
            return Err(FormatterException::new(format!(
                "error: invalid length {length}, offset + length ({required_length}) is outside the file"
            )));
        }
        let expanded_length = match length {
            _ if length < 0 => return Ok(EMPTY_RANGE),
            0 => 1, // 0 stands for "format the line under the cursor"
            _ => length,
        };
        // `positionTokenMap.subRangeMap(closedOpen(offset, offset + expandedLength))`: the token
        // spans are sorted and disjoint, so both bounds are monotone in the token index.
        let tokens = &self.tokens;
        let first = tokens.partition_point(|t| Self::token_span(&**t).1 < offset);
        let end = tokens.partition_point(|t| Self::token_span(&**t).0 < offset + expanded_length);
        if first >= end {
            return Ok(EMPTY_RANGE);
        }
        Ok(Range::closed_open(
            tokens[first].get_tok().get_index(),
            tokens[end - 1].get_tok().get_index() + 1,
        ))
    }

    fn make_position_to_column_map(&self) -> HashMap<i32, i32> {
        let token_toks = self.tokens.iter().flat_map(|token| {
            let tok = std::slice::from_ref(token.get_tok());
            token.get_toks_before().iter().chain(tok).chain(token.get_toks_after())
        });
        let dropped = self.dropped_toks.iter().map(|tok| &**tok as &dyn Tok);
        token_toks
            .map(|tok| &**tok)
            .chain(dropped)
            .map(|tok| (tok.get_position(), tok.get_column()))
            .collect()
    }

    /// Returns the shared source, the toks and `kN`; also computes the input's line ranges.
    fn build_toks(
        io: &mut InputOutput,
        file: &PsiElement,
        file_text: &str,
    ) -> Result<(Rc<str>, Vec<Rc<KotlinTok>>, i32), ParseError> {
        let mut tokenizer = Tokenizer::new(file_text);
        tokenizer.visit_file(file)?;
        let k_n = tokenizer.index();
        let mut toks = tokenizer.toks;
        let eof = file_text.len();
        toks.push(KotlinTok::from_source(k_n, &tokenizer.source, eof..eof, None, 0, true));
        io.compute_ranges(&toks);
        Ok((tokenizer.source, toks.into_iter().map(Rc::new).collect(), k_n))
    }

    /// Upstream's index loop over `toks`, which only ever looks at the next tok: a peekable
    /// iterator, so each tok moves into its token.
    fn build_tokens(&mut self, toks: Vec<Rc<KotlinTok>>) {
        let mut toks = toks.into_iter().peekable();

        // Remaining non-tokens before the token, then the token and the non-tokens after it.
        let mut token_toks: Vec<Rc<KotlinTok>> = Vec::new();

        'outermost: while toks.peek().is_some() {
            while let Some(tok) = toks.next_if(|tok| !tok.is_token) {
                let is_param_comment = Self::is_param_comment(&*tok);
                token_toks.push(tok);
                if is_param_comment {
                    self.drop_newlines(&mut toks);
                }
            }
            let tok = toks.next().expect("the EOF tok is a token");
            // Don't attach inline comments to certain leading tokens, e.g. for
            // `f(/*flag1=*/true)`. This barely scratches the surface, but it's enough to do a
            // better job with parameter name comments.
            let text = tok.get_text();
            let (keep_slash_star, keep_javadoc) = (matches!(text, "(" | "<" | "."), text == ";");
            let tok_i = token_toks.len();
            token_toks.push(tok);

            // Non-tokens starting on the same line go here too.
            while let Some(next) = toks.peek().filter(|tok| !tok.is_token) {
                if next.is_slash_star_comment() && keep_slash_star {
                    break;
                }
                if next.is_javadoc_comment() && keep_javadoc {
                    break;
                }
                if Self::is_param_comment(&**next) {
                    self.push_token(&mut token_toks, tok_i);
                    token_toks.extend(toks.next());
                    self.drop_newlines(&mut toks);
                    continue 'outermost;
                }
                let non_token_after = toks.next().expect("peeked");
                let breaks = newlines::contains_breaks(non_token_after.get_text());
                token_toks.push(non_token_after);
                if breaks {
                    break;
                }
            }
            self.push_token(&mut token_toks, tok_i);
        }
    }

    /// Drops the newlines after a parameter comment.
    fn drop_newlines(&mut self, toks: &mut Peekable<impl Iterator<Item = Rc<KotlinTok>>>) {
        while let Some(tok) = toks.next_if(|tok| tok.is_newline()) {
            self.dropped_toks.push(tok);
        }
    }

    /// Adds the token made of (and empties) `token_toks`, and indexes its numbered toks.
    fn push_token(&mut self, token_toks: &mut Vec<Rc<KotlinTok>>, tok_i: usize) {
        let i = self.tokens.len() as u32;
        for tok in token_toks.iter() {
            if tok.get_index() >= 0 {
                self.k_to_token[tok.get_index() as usize] = Some(i);
            }
        }
        let toks = token_toks.drain(..).map(|tok| tok as Rc<dyn Tok>).collect();
        self.tokens.push(Rc::new(KotlinToken::new(toks, tok_i)));
    }

    fn build_token_positions_map(tokens: &[Rc<dyn Token>]) -> RangeMap<Rc<dyn Token>> {
        let entries = tokens
            .iter()
            .map(|token| {
                let (start, end) = Self::token_span(&**token);
                (start, end, token.clone())
            })
            .collect();
        RangeMap::from_closed(entries)
    }

    /// The closed position range a token covers in `positionTokenMap`.
    fn token_span(token: &dyn Token) -> (i32, i32) {
        let end = JavaOutput::end_tok(token);
        // Byte length: this is position arithmetic.
        let end_length = end.get_original_text().len() as i32;
        let end_position = end.get_position()
            + if !end.get_text().is_empty() {
                end_length - 1
            } else {
                0
            };
        (JavaOutput::start_tok(token).get_position(), end_position)
    }

    /// `/\*[A-Za-z0-9\s_\-]+=\s*\*/`, fully matched.
    fn is_param_comment(tok: &dyn Tok) -> bool {
        let is_space = |c: char| matches!(c, ' ' | '\t' | '\n' | '\u{0b}' | '\u{0c}' | '\r');
        if !tok.is_slash_star_comment() {
            return false;
        }
        let Some(body) = tok.get_text().strip_prefix("/*") else {
            return false;
        };
        let name_len = body
            .find(|c: char| !(c.is_ascii_alphanumeric() || is_space(c) || c == '_' || c == '-'));
        match name_len {
            Some(n) if n > 0 => body[n..]
                .strip_prefix('=')
                .is_some_and(|rest| rest.trim_start_matches(is_space) == "*/"),
            _ => false,
        }
    }
}

impl Input for KotlinInput {
    fn input_output(&self) -> &InputOutput {
        &self.io
    }

    fn get_tokens(&self) -> &[Rc<dyn Token>] {
        &self.tokens
    }

    fn get_position_token_map(&self) -> &RangeMap<Rc<dyn Token>> {
        self.position_token_map.get_or_init(|| Self::build_token_positions_map(&self.tokens))
    }

    fn get_position_to_column_map(&self) -> &HashMap<i32, i32> {
        self.position_to_column_map.get_or_init(|| self.make_position_to_column_map())
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn get_kn(&self) -> i32 {
        self.k_n
    }

    fn get_token(&self, k: i32) -> Option<&Rc<dyn Token>> {
        let i = (*self.k_to_token.get(usize::try_from(k).ok()?)?)?;
        Some(&self.tokens[i as usize])
    }

    /// Past the end of the text IntelliJ returns null (an NPE upstream); clamp instead.
    fn get_line_number(&self, input_position: i32) -> i32 {
        self.line_column(input_position).0 + 1
    }

    fn get_column_number(&self, input_position: i32) -> i32 {
        self.line_column(input_position).1
    }
}

impl KotlinInput {
    fn line_column(&self, input_position: i32) -> (i32, i32) {
        let offset = (input_position.max(0) as usize).min(self.text.len());
        offset_to_line_column(&self.text, offset).expect("clamped offset")
    }
}
