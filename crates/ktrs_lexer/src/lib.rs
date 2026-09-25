//! Ports of `Kotlin.flex` and `KDoc.flex` (third_party/kotlin/compiler/psi/parser/src/.../lexer).
//! Both lexers are lossless: token lengths always sum to the input length. Tokens may be empty
//! (`DANGLING_NEWLINE`), exactly as upstream.

mod chars;
mod kdoc;
mod kotlin;
mod unicode_tables;

use ktrs_syntax::SyntaxKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub len: u32,
}

fn to_tokens(spans: impl IntoIterator<Item = (SyntaxKind, usize, usize)>) -> Vec<Token> {
    spans
        .into_iter()
        .map(|(kind, start, end)| Token {
            kind,
            len: (end - start) as u32,
        })
        .collect()
}

/// Lexes Kotlin source. Hard keywords get their keyword kind; soft and modifier keywords are
/// `IDENTIFIER` (the parser remaps them). A whole `/** ... */` is one `DOC_COMMENT` token.
pub fn tokenize(text: &str) -> Vec<Token> {
    // Real Kotlin averages ~2.5 bytes per token; reserving up front avoids repeated regrowth.
    let mut tokens = Vec::with_capacity(text.len() / 2 + 8);
    tokens.extend(tokens_of(text));
    tokens
}

/// [`tokenize`] as an iterator, for callers that store tokens in their own layout.
pub fn tokens_of(text: &str) -> impl Iterator<Item = Token> + '_ {
    let mut lexer = kotlin::KotlinLexer::new(text);
    std::iter::from_fn(move || lexer.advance()).map(|(kind, start, end)| Token { kind, len: (end - start) as u32 })
}

/// Lexes the text of one `DOC_COMMENT` token into KDoc tokens.
pub fn tokenize_kdoc(text: &str) -> Vec<Token> {
    let mut lexer = kdoc::KDocFlexLexer::new(text);
    let raw: Vec<_> = std::iter::from_fn(|| lexer.advance()).collect();
    to_tokens(kdoc::merge_tokens(text, &raw))
}
