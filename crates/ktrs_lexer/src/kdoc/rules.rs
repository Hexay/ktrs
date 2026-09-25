//! The rules of `KDoc.flex`, in spec order, with their state guards and actions.

use ktrs_syntax::SyntaxKind::{self, *};

use super::scan::*;
use super::{BlockType, KDocFlexLexer, LexState};

/// One variant per flex rule, in file order (the order breaks longest-match ties).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Rule {
    Start,
    AsterisksSlash,
    LineBeginningWhiteSpace,
    LineBeginningAsterisks,
    TagName,
    TagLineBreak,
    TagWhiteSpace,
    TagCodeLink,
    TagQualifiedName,
    TagAny,
    TagTextLineBreak,
    TagTextWhiteSpace,
    TagTextCodeLink,
    TagTextBacktickString,
    TagTextAny,
    CodeFenceStart,
    BacktickString,
    CodeSpanLineBreak,
    CodeSpanBacktickString,
    CodeSpanAny,
    CodeSpanLineBeginningWhiteSpace,
    CodeSpanLineBeginningAsterisks,
    LineBreak,
    WhiteSpace,
    EscapedChars,
    Lpar,
    Rpar,
    CodeLink,
    Any,
    CodeBlockLineBeginningWhiteSpace,
    CodeBlockLineBeginningAsterisks,
    CodeFenceEnd,
    CodeBlockLineBreak,
    CodeBlockWhiteSpace,
    CodeBlockAny,
    BadCharacter,
}

fn literal(text: &str, pos: usize, lit: &str) -> Option<usize> {
    text[pos..].starts_with(lit).then_some(pos + lit.len())
}

fn is_reference_required_tag(name: &str) -> bool {
    ["THROWS", "EXCEPTION", "PARAM", "SEE", "PROPERTY", "SAMPLE"]
        .iter()
        .any(|tag| tag.eq_ignore_ascii_case(name))
}

impl KDocFlexLexer<'_> {
    /// Returns the winning rule and the end of its (lookahead-free) match.
    pub(super) fn select_rule(&self, p: usize) -> (Rule, usize) {
        use LexState::*;
        use Rule::*;
        let t = self.text;
        let s = self.state;
        let mut best = (BadCharacter, 0, 0);
        let mut consider = |rule: Rule, matched: Option<(usize, usize)>| {
            if let Some((total, end)) = matched
                && total > best.1
            {
                best = (rule, total, end);
            }
        };
        let plain = |m: Option<usize>| m.map(|end| (end, end));

        if s == Yyinitial {
            consider(Start, plain(literal(t, p, "/**")));
        }
        consider(AsterisksSlash, plain(asterisks_slash(t, p)));
        if s == LineBeginning {
            consider(LineBeginningWhiteSpace, plain(white_space(t, p)));
            consider(LineBeginningAsterisks, plain(asterisks(t, p)));
        }
        if s == ContentsBeginning {
            consider(TagName, plain(tag_name(t, p)));
        }
        if s == TagBeginning {
            consider(TagLineBreak, plain(line_break(t, p)));
            consider(TagWhiteSpace, plain(white_space(t, p)));
            consider(TagCodeLink, plain(code_link(t, p)));
            consider(TagQualifiedName, plain(qualified_name(t, p)));
            consider(TagAny, plain(any_char(t, p)));
        }
        if s == TagTextBeginning {
            consider(TagTextLineBreak, plain(line_break(t, p)));
            consider(TagTextWhiteSpace, plain(white_space(t, p)));
            consider(TagTextCodeLink, plain(code_link(t, p)));
            consider(TagTextBacktickString, plain(backtick_string(t, p)));
            consider(TagTextAny, plain(any_char(t, p)));
        }
        if s == ContentsBeginning {
            consider(CodeFenceStart, code_fence_start(t, p));
        }
        if matches!(s, ContentsBeginning | Contents) {
            consider(BacktickString, plain(backtick_string(t, p)));
        }
        if s == CodeSpanContents {
            consider(CodeSpanLineBreak, plain(line_break(t, p)));
            consider(CodeSpanBacktickString, plain(backtick_string(t, p)));
            consider(CodeSpanAny, plain(any_char(t, p)));
        }
        if s == CodeSpanLineBeginning {
            consider(CodeSpanLineBeginningWhiteSpace, plain(white_space(t, p)));
            consider(CodeSpanLineBeginningAsterisks, plain(asterisks(t, p)));
        }
        if matches!(s, LineBeginning | ContentsBeginning | Contents) {
            consider(LineBreak, plain(line_break(t, p)));
            consider(WhiteSpace, plain(white_space(t, p)));
            consider(EscapedChars, plain(escaped_chars(t, p)));
            consider(Lpar, plain(literal(t, p, "(")));
            consider(Rpar, plain(literal(t, p, ")")));
            consider(CodeLink, code_link_not_followed_by_link(t, p));
            consider(Any, plain(any_char(t, p)));
        }
        if s == CodeBlockLineBeginning {
            consider(CodeBlockLineBeginningWhiteSpace, plain(white_space(t, p)));
            consider(CodeBlockLineBeginningAsterisks, plain(asterisks(t, p)));
        }
        if matches!(s, CodeBlockLineBeginning | CodeBlockContentsBeginning) {
            consider(CodeFenceEnd, code_fence_end(t, p));
        }
        if matches!(
            s,
            IndentedCodeBlock | CodeBlockLineBeginning | CodeBlockContentsBeginning | CodeBlock
        ) {
            consider(CodeBlockLineBreak, plain(line_break(t, p)));
            consider(CodeBlockWhiteSpace, plain(white_space(t, p)));
            consider(CodeBlockAny, plain(any_char(t, p)));
        }
        consider(BadCharacter, plain(any_char(t, p)));
        (best.0, best.2)
    }

    pub(super) fn run_action(&mut self, rule: Rule) -> SyntaxKind {
        use LexState::*;
        match rule {
            Rule::Start => {
                self.yybegin_and_update(ContentsBeginning);
                KDOC_START
            }
            Rule::AsterisksSlash => {
                if self.is_last_token() {
                    KDOC_END
                } else {
                    KDOC_TEXT
                }
            }
            Rule::LineBeginningWhiteSpace
            | Rule::TagWhiteSpace
            | Rule::TagTextWhiteSpace
            | Rule::CodeSpanLineBeginningWhiteSpace
            | Rule::CodeBlockLineBeginningWhiteSpace => WHITE_SPACE,
            Rule::LineBeginningAsterisks => {
                self.state = ContentsBeginning;
                KDOC_LEADING_ASTERISK
            }
            Rule::TagName => {
                self.last_block_type = Some(BlockType::Paragraph);
                let name = &self.text[self.token_start + 1..self.pos];
                let next = if is_reference_required_tag(name) {
                    TagBeginning
                } else {
                    TagTextBeginning
                };
                self.yybegin_and_update(next);
                KDOC_TAG_NAME
            }
            Rule::TagLineBreak | Rule::TagTextLineBreak | Rule::LineBreak => {
                self.yybegin_and_update(LineBeginning);
                WHITE_SPACE
            }
            Rule::TagCodeLink | Rule::TagQualifiedName => {
                self.yybegin_and_update(TagTextBeginning);
                KDOC_MARKDOWN_LINK
            }
            Rule::TagAny | Rule::TagTextAny => {
                self.yybegin_and_update(Contents);
                KDOC_TEXT
            }
            Rule::TagTextCodeLink => {
                self.yybegin_and_update(Contents);
                KDOC_MARKDOWN_LINK
            }
            Rule::TagTextBacktickString | Rule::BacktickString | Rule::CodeSpanBacktickString
                if self.is_last_token() =>
            {
                // Upstream `countRepeating` reads past the buffer end here and throws; FlexAdapter
                // then reports BAD_CHARACTER up to the end of the text.
                BAD_CHARACTER
            }
            Rule::TagTextBacktickString | Rule::BacktickString => {
                self.code_fence_char = self.text.as_bytes()[self.token_start];
                let length = self.count_repeating(self.code_fence_char);
                self.code_fence_length = length as i32;
                if self.has_matching_close_fence(self.code_fence_char, length) {
                    self.yybegin_and_update(CodeSpanContents);
                }
                KDOC_TEXT
            }
            Rule::CodeFenceStart => {
                self.last_block_type = Some(BlockType::Code);
                self.code_fence_char = self.text.as_bytes()[self.token_start];
                self.code_fence_length = self.count_repeating(self.code_fence_char) as i32;
                self.yybegin_and_update(CodeBlockLineBeginning);
                KDOC_TEXT
            }
            Rule::CodeSpanLineBreak => {
                self.yybegin_and_update(CodeSpanLineBeginning);
                WHITE_SPACE
            }
            Rule::CodeSpanBacktickString => {
                if self.closes_code_fence() {
                    self.yybegin_and_update(Contents);
                    KDOC_TEXT
                } else {
                    KDOC_CODE_SPAN_TEXT
                }
            }
            Rule::CodeSpanAny => KDOC_CODE_SPAN_TEXT,
            Rule::CodeSpanLineBeginningAsterisks => {
                self.yybegin_and_update(CodeSpanContents);
                KDOC_LEADING_ASTERISK
            }
            Rule::WhiteSpace => self.contents_white_space(),
            Rule::EscapedChars => self.paragraph(KDOC_MARKDOWN_ESCAPED_CHAR),
            Rule::Lpar => self.paragraph(KDOC_LPAR),
            Rule::Rpar => self.paragraph(KDOC_RPAR),
            Rule::CodeLink => self.paragraph(KDOC_MARKDOWN_LINK),
            Rule::Any => self.paragraph(KDOC_TEXT),
            Rule::CodeBlockLineBeginningAsterisks => {
                self.state = CodeBlockContentsBeginning;
                KDOC_LEADING_ASTERISK
            }
            Rule::CodeFenceEnd => {
                if self.closes_code_fence() {
                    self.yybegin_and_update(Contents);
                    KDOC_TEXT
                } else {
                    KDOC_CODE_BLOCK_TEXT
                }
            }
            Rule::CodeBlockLineBreak => {
                let next = if self.state == IndentedCodeBlock {
                    LineBeginning
                } else {
                    CodeBlockLineBeginning
                };
                self.yybegin_and_update(next);
                WHITE_SPACE
            }
            Rule::CodeBlockWhiteSpace => KDOC_CODE_BLOCK_TEXT,
            Rule::CodeBlockAny => {
                let next = if self.state == IndentedCodeBlock {
                    IndentedCodeBlock
                } else {
                    CodeBlock
                };
                self.yybegin_and_update(next);
                KDOC_CODE_BLOCK_TEXT
            }
            Rule::BadCharacter => {
                self.last_block_type = Some(BlockType::Paragraph);
                BAD_CHARACTER
            }
        }
    }

    /// `<LINE_BEGINNING, CONTENTS_BEGINNING, CONTENTS> {WHITE_SPACE_CHAR}+`.
    fn contents_white_space(&mut self) -> SyntaxKind {
        let bytes = self.text.as_bytes();
        let (start, end) = (self.token_start, self.pos);
        if self.state == LexState::ContentsBeginning
            && (end - start >= 4 || bytes[start] == b'\t' || bytes[end - 1] == b'\t')
            && (self.last_block_type != Some(BlockType::Paragraph)
                || self.consecutive_line_break_count >= 2)
        {
            self.state = LexState::IndentedCodeBlock;
            self.last_block_type = Some(BlockType::Code);
            return KDOC_CODE_BLOCK_TEXT;
        }
        if self.state != LexState::ContentsBeginning {
            self.state = LexState::Contents;
        }
        KDOC_TEXT
    }
}
