//! Port of `KDoc.flex` (`_KDocLexer`) plus the token merging of `KDocLexer`.
//! All states are inclusive, so rule selection is done generically: every rule active in the current
//! state is tried in spec order, the longest match wins and ties go to the earlier rule.

mod merge;
mod rules;
mod scan;

pub(crate) use merge::merge_tokens;

use ktrs_syntax::SyntaxKind;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LexState {
    Yyinitial,
    LineBeginning,
    ContentsBeginning,
    TagBeginning,
    TagTextBeginning,
    Contents,
    CodeBlock,
    CodeBlockLineBeginning,
    CodeBlockContentsBeginning,
    IndentedCodeBlock,
    CodeSpanContents,
    CodeSpanLineBeginning,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockType {
    Paragraph,
    Code,
}

pub(crate) struct KDocFlexLexer<'a> {
    text: &'a str,
    token_start: usize,
    pos: usize,
    state: LexState,
    consecutive_line_break_count: u32,
    code_fence_char: u8,
    code_fence_length: i32,
    last_block_type: Option<BlockType>,
}

impl<'a> KDocFlexLexer<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        KDocFlexLexer {
            text,
            token_start: 0,
            pos: 0,
            state: LexState::Yyinitial,
            consecutive_line_break_count: 0,
            code_fence_char: 0,
            code_fence_length: -1,
            last_block_type: None,
        }
    }

    fn yybegin_and_update(&mut self, new_state: LexState) {
        self.consecutive_line_break_count = if matches!(
            new_state,
            LexState::LineBeginning | LexState::CodeBlockLineBeginning
        ) {
            self.consecutive_line_break_count + 1
        } else {
            0
        };
        self.state = new_state;
    }

    fn is_last_token(&self) -> bool {
        self.pos == self.text.len()
    }

    /// `countRepeating`: length of the run of `c` at the token start, within the token.
    fn count_repeating(&self, c: u8) -> usize {
        self.text.as_bytes()[self.token_start..self.pos]
            .iter()
            .take_while(|&&b| b == c)
            .count()
    }

    fn has_matching_close_fence(&self, c: u8, length: usize) -> bool {
        scan::has_matching_close_fence(self.text, self.pos, c, length)
    }

    /// Shared action of the code span / code fence end rules: resets the fence when it matches.
    fn closes_code_fence(&mut self) -> bool {
        let ch = self.text.as_bytes()[self.token_start];
        let length = self.count_repeating(ch) as i32;
        if length == self.code_fence_length && ch == self.code_fence_char {
            self.code_fence_length = -1;
            self.code_fence_char = 0;
            true
        } else {
            false
        }
    }

    /// Shared action of the paragraph rules.
    fn paragraph(&mut self, kind: SyntaxKind) -> SyntaxKind {
        self.last_block_type = Some(BlockType::Paragraph);
        self.yybegin_and_update(LexState::Contents);
        kind
    }

    /// `advance()`: the next unmerged token as `(kind, start, end)`, `None` at EOF.
    pub(crate) fn advance(&mut self) -> Option<(SyntaxKind, usize, usize)> {
        let start = self.pos;
        if start >= self.text.len() {
            return None;
        }
        self.token_start = start;
        if let Some((kind, end)) = self.plain_text_run(start) {
            self.pos = end;
            return Some((kind, start, end));
        }
        let (rule, end) = self.select_rule(start);
        self.pos = end;
        let kind = self.run_action(rule);
        Some((kind, start, end))
    }

    /// Fast path: a run of chars on which only the state's catch-all `[^]` rule matches. Its
    /// one-char tokens would be merged anyway, and never precede a code fence `TEXT`.
    fn plain_text_run(&mut self, start: usize) -> Option<(SyntaxKind, usize)> {
        const CONTENTS: [bool; 256] = byte_set(b"*`\r\n \t\x0c\\()[");
        const CODE_BLOCK: [bool; 256] = byte_set(b"*\r\n \t\x0c");
        const CODE_SPAN: [bool; 256] = byte_set(b"*`\r\n");
        let (kind, special) = match self.state {
            LexState::Contents => (SyntaxKind::KDOC_TEXT, &CONTENTS),
            LexState::CodeBlock | LexState::IndentedCodeBlock => {
                (SyntaxKind::KDOC_CODE_BLOCK_TEXT, &CODE_BLOCK)
            }
            LexState::CodeSpanContents => (SyntaxKind::KDOC_CODE_SPAN_TEXT, &CODE_SPAN),
            _ => return None,
        };
        let bytes = self.text.as_bytes();
        let end = start
            + bytes[start..]
                .iter()
                .take_while(|&&b| !special[b as usize])
                .count();
        if end == start {
            return None;
        }
        match self.state {
            LexState::Contents => {
                self.last_block_type = Some(BlockType::Paragraph);
                self.yybegin_and_update(LexState::Contents);
            }
            LexState::CodeSpanContents => {}
            state => self.yybegin_and_update(state),
        }
        Some((kind, end))
    }
}

const fn byte_set(bytes: &[u8]) -> [bool; 256] {
    let mut set = [false; 256];
    let mut i = 0;
    while i < bytes.len() {
        set[bytes[i] as usize] = true;
        i += 1;
    }
    set
}
