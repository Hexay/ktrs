//! Port of ktfmt's `KotlinInput.kt`: what `JavaInput` is for Java, with the Kotlin parse tree as
//! the lexer.

use std::cell::OnceCell;
use std::collections::HashMap;
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
    text: String,
    /// The Tokens for this input.
    tokens: Vec<Rc<dyn Token>>,
    toks: Vec<Rc<KotlinTok>>,
    /// Map Tok position to column. Built on first use: ktfmt never reads it.
    position_to_column_map: OnceCell<HashMap<i32, i32>>,
    /// Map position to Token.
    position_token_map: RangeMap<Rc<dyn Token>>,
    /// The number of numbered toks (tokens or comments), excluding the EOF.
    k_n: i32,
    /// Indices into `tokens`.
    k_to_token: Vec<Option<u32>>,
}

impl KotlinInput {
    pub fn new(text: &str, file: &PsiElement) -> Result<KotlinInput, ParseError> {
        let mut io = InputOutput::default();
        io.set_line_count(newlines::line_iterator(text).count());
        let (toks, k_n) = Self::build_toks(&mut io, file, text)?;
        let tokens = Self::build_tokens(&toks);
        let position_token_map = Self::build_token_positions_map(&tokens);

        // adjust kN for EOF
        let mut k_to_token: Vec<Option<u32>> = vec![None; k_n as usize + 1];
        for (i, token) in tokens.iter().enumerate() {
            let numbered = token
                .get_toks_before()
                .iter()
                .chain([token.get_tok()])
                .chain(token.get_toks_after());
            for tok in numbered {
                if tok.get_index() >= 0 {
                    k_to_token[tok.get_index() as usize] = Some(i as u32);
                }
            }
        }
        Ok(KotlinInput {
            io,
            text: text.to_string(),
            tokens,
            toks,
            position_to_column_map: OnceCell::new(),
            position_token_map,
            k_n,
            k_to_token,
        })
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
        let mut enclosed = self
            .position_token_map
            .sub_range_values_closed_open(offset, offset + expanded_length);
        let first = enclosed.next();
        let last = enclosed.next_back().or(first);
        match (first, last) {
            (Some(first), Some(last)) => Ok(Range::closed_open(
                first.get_tok().get_index(),
                last.get_tok().get_index() + 1,
            )),
            _ => Ok(EMPTY_RANGE),
        }
    }

    fn make_position_to_column_map(toks: &[Rc<KotlinTok>]) -> HashMap<i32, i32> {
        toks.iter()
            .map(|tok| (tok.get_position(), tok.get_column()))
            .collect()
    }

    /// Returns the toks and `kN`; also computes the input's line ranges.
    fn build_toks(
        io: &mut InputOutput,
        file: &PsiElement,
        file_text: &str,
    ) -> Result<(Vec<Rc<KotlinTok>>, i32), ParseError> {
        let mut tokenizer = Tokenizer::new(file_text);
        tokenizer.visit_file(file)?;
        let k_n = tokenizer.index();
        let mut toks: Vec<Rc<KotlinTok>> = tokenizer.toks.into_iter().map(Rc::new).collect();
        toks.push(Rc::new(KotlinTok::new(
            k_n,
            String::new(),
            String::new(),
            file_text.len() as i32,
            0,
            true,
        )));
        io.compute_ranges(&toks);
        Ok((toks, k_n))
    }

    fn build_tokens(toks: &[Rc<KotlinTok>]) -> Vec<Rc<dyn Token>> {
        let as_dyn = |tok: &Rc<KotlinTok>| -> Rc<dyn Tok> { tok.clone() };
        let mut tokens: Vec<Rc<dyn Token>> = Vec::with_capacity(toks.len() / 2);
        let mut k = 0;
        let k_n = toks.len();

        // Remaining non-tokens before the token go here.
        let mut toks_before: Vec<Rc<dyn Tok>> = Vec::new();

        'outermost: while k < k_n {
            while !toks[k].is_token {
                let tok = &toks[k];
                k += 1;
                toks_before.push(as_dyn(tok));
                if Self::is_param_comment(&**tok) {
                    while toks[k].is_newline() {
                        // drop newlines after parameter comments
                        k += 1;
                    }
                }
            }
            let tok = &toks[k];
            k += 1;

            // Non-tokens starting on the same line go here too.
            let mut toks_after: Vec<Rc<dyn Tok>> = Vec::new();
            while k < k_n && !toks[k].is_token {
                // Don't attach inline comments to certain leading tokens, e.g. for
                // `f(/*flag1=*/true)`. This barely scratches the surface, but it's enough to do a
                // better job with parameter name comments.
                let text = tok.get_text();
                if toks[k].is_slash_star_comment() && (text == "(" || text == "<" || text == ".") {
                    break;
                }
                if toks[k].is_javadoc_comment() && text == ";" {
                    break;
                }
                if Self::is_param_comment(&*toks[k]) {
                    tokens.push(Rc::new(KotlinToken::new(
                        std::mem::take(&mut toks_before),
                        as_dyn(tok),
                        toks_after,
                    )));
                    toks_before = vec![as_dyn(&toks[k])];
                    k += 1;
                    // drop newlines after parameter comments
                    while toks[k].is_newline() {
                        k += 1;
                    }
                    continue 'outermost;
                }
                let non_token_after = &toks[k];
                k += 1;
                toks_after.push(as_dyn(non_token_after));
                if newlines::contains_breaks(non_token_after.get_text()) {
                    break;
                }
            }
            tokens.push(Rc::new(KotlinToken::new(
                std::mem::take(&mut toks_before),
                as_dyn(tok),
                toks_after,
            )));
        }
        tokens
    }

    fn build_token_positions_map(tokens: &[Rc<dyn Token>]) -> RangeMap<Rc<dyn Token>> {
        let entries = tokens
            .iter()
            .map(|token| {
                let end = JavaOutput::end_tok(&**token);
                // Byte length: this is position arithmetic.
                let end_length = end.get_original_text().len() as i32;
                let end_position = end.get_position()
                    + if !end.get_text().is_empty() {
                        end_length - 1
                    } else {
                        0
                    };
                (
                    JavaOutput::start_tok(&**token).get_position(),
                    end_position,
                    token.clone(),
                )
            })
            .collect();
        RangeMap::from_closed(entries)
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
        &self.position_token_map
    }

    fn get_position_to_column_map(&self) -> &HashMap<i32, i32> {
        self.position_to_column_map.get_or_init(|| Self::make_position_to_column_map(&self.toks))
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
