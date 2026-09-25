//! Port of `KDocToken.kt` and `NestingCounter.kt` (unused upstream; kept for parity).

use std::fmt;

use super::kstring::{KString, to_string};

/// KDoc token type; every token needing special handling from `KDocWriter` gets its own type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KDocTokenType {
    /// `/**`
    BeginKdoc,
    /// `*/`
    EndKdoc,
    ListItemOpenTag,
    HeaderOpenTag,
    ParagraphOpenTag,
    PreOpenTag,
    PreCloseTag,
    CodeOpenTag,
    CodeCloseTag,
    TableOpenTag,
    TableCloseTag,
    /// Things such as `@param` or `@see`
    Tag,
    /// Code between two markers of three backticks
    Code,
    /// three backticks
    CodeBlockMarker,
    /// A link in brackets such as [KDocToken]
    MarkdownLink,
    BlankLine,
    /// Whitespace outside `<pre>`/`<table>`, rewritten to newlines or spaces on output.
    Whitespace,
    /// Anything else, including whitespace inside `<pre>`/`<table>` (kept verbatim).
    Literal,
}

#[derive(Clone, Debug)]
pub struct KDocToken {
    pub token_type: KDocTokenType,
    pub value: KString,
}

impl KDocToken {
    pub fn new(token_type: KDocTokenType, value: KString) -> Self {
        KDocToken { token_type, value }
    }

    pub fn length(&self) -> i32 {
        self.value.len() as i32
    }
}

impl fmt::Display for KDocToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KDocToken{{{:?}: \"{}\"}}", self.token_type, to_string(&self.value))
    }
}

/// Mutable integer for tracking the level of nesting.
#[derive(Debug, Default)]
pub struct NestingCounter {
    value: i32,
}

impl NestingCounter {
    pub fn is_positive(&self) -> bool {
        self.value > 0
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn increment_if_positive(&mut self) {
        if self.value > 0 {
            self.value += 1;
        }
    }

    pub fn decrement_if_positive(&mut self) {
        if self.value > 0 {
            self.value -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }
}
