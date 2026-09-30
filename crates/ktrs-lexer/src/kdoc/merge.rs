//! `KDocLexer.KDocLexerMergeFunction`, applied to the flex token stream after the fact (the flex
//! lexer's actions never depend on merging, so this equals the lazy `MergingLexerAdapterBase`).

use ktrs_syntax::SyntaxKind::{self, *};

fn is_valid_code_fence(text: &str) -> bool {
    let bytes = text.as_bytes();
    // `str.length() < 3` counts UTF-16 units; any fence char is ASCII, so bytes agree.
    bytes.len() >= 3 && matches!(bytes[0], b'`' | b'~') && bytes.iter().all(|&b| b == bytes[0])
}

fn is_mergeable(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        KDOC_CODE_BLOCK_TEXT | KDOC_CODE_SPAN_TEXT | KDOC_TEXT | WHITE_SPACE
    )
}

/// Merges raw `(kind, start, end)` tokens into `(kind, start, end)` tokens.
pub(crate) fn merge_tokens(
    text: &str,
    raw: &[(SyntaxKind, usize, usize)],
) -> Vec<(SyntaxKind, usize, usize)> {
    let mut merged = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        let (kind, start, mut end) = raw[i];
        i += 1;
        let mut merged_kind = kind;
        match raw.get(i) {
            Some(&(KDOC_TEXT, next_start, next_end))
                if kind == KDOC_CODE_BLOCK_TEXT
                    && is_valid_code_fence(&text[next_start..next_end]) =>
            {
                end = next_end;
                i += 1;
                merged_kind = KDOC_TEXT;
            }
            _ if is_mergeable(kind) => {
                while let Some(&(next_kind, _, next_end)) = raw.get(i) {
                    if next_kind != kind {
                        break;
                    }
                    end = next_end;
                    i += 1;
                }
            }
            _ => {}
        }
        merged.push((merged_kind, start, end));
    }
    merged
}
