//! The stack guard of the parse entry point. The parser is recursive descent and a stack overflow aborts the
//! process, so the entry refuses input nested deeper than [`MAX_NESTING_DEPTH`] and runs the parser with a
//! stack budget it knows is available. Measurements and the reasoning behind the constants:
//! research/36-parser-library.md, "Very deep nesting".

use ktrs_lexer::tokens_of;
use ktrs_syntax::SyntaxKind as Raw;

use crate::ParseError;

/// The deepest nesting of `(`, `[`, `{` and `${` that [`parse`](crate::parse) accepts.
pub const MAX_NESTING_DEPTH: usize = 1000;

const STACK_PER_LEVEL: usize = if cfg!(debug_assertions) { 64 << 10 } else { 8 << 10 };
/// Recursion that opens no bracket (`else if` chains, `List<List<..`, prefix operators) needs input bytes.
const STACK_PER_BYTE: usize = if cfg!(debug_assertions) { 2 << 10 } else { 256 };
const BASE_LEVELS: usize = 32;
const MAX_STACK: usize = if cfg!(target_pointer_width = "64") { 256 << 20 } else { 64 << 20 };

/// An upper bound of the bracket nesting of `text`, or the error for nesting past the limit.
pub(crate) fn nesting_depth(text: &str) -> Result<usize, ParseError> {
    let openers = text.bytes().filter(|b| matches!(b, b'(' | b'[' | b'{')).count();
    if openers <= MAX_NESTING_DEPTH {
        return Ok(openers);
    }
    let mut open: Vec<Raw> = Vec::new();
    let (mut deepest, mut offset) = (0, 0);
    for token in tokens_of(text) {
        match token.kind {
            Raw::LPAR => open.push(Raw::RPAR),
            Raw::LBRACKET => open.push(Raw::RBRACKET),
            Raw::LBRACE => open.push(Raw::RBRACE),
            Raw::LONG_TEMPLATE_ENTRY_START => open.push(Raw::LONG_TEMPLATE_ENTRY_END),
            // A closer that doesn't match stays open: the parser's recovery may skip it and nest deeper.
            closer if open.last() == Some(&closer) => {
                open.pop();
            }
            _ => {}
        }
        if open.len() > MAX_NESTING_DEPTH {
            return Err(ParseError::TooDeeplyNested { offset });
        }
        deepest = deepest.max(open.len());
        offset += token.len as usize;
    }
    Ok(deepest)
}

/// Runs `parser` with enough stack for `depth` bracket levels over `len` bytes: on the current stack when it
/// has that much left, otherwise on a fresh one.
pub(crate) fn with_parser_stack<R>(depth: usize, len: usize, parser: impl FnOnce() -> R) -> R {
    let budget = ((depth + BASE_LEVELS) * STACK_PER_LEVEL).max(len.saturating_mul(STACK_PER_BYTE)).min(MAX_STACK);
    stacker::maybe_grow(budget, budget, parser)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested(open: &str, close: &str, depth: usize) -> String {
        format!("val x = {}1{}", open.repeat(depth), close.repeat(depth))
    }

    #[test]
    fn shallow_text_is_bounded_by_its_opening_brackets() {
        assert_eq!(nesting_depth("fun f() { g(a[0]) }"), Ok(4));
        assert_eq!(nesting_depth(""), Ok(0));
    }

    #[test]
    fn deep_text_is_measured_on_tokens() {
        let flat = "val x = f()\n".repeat(MAX_NESTING_DEPTH + 1);
        assert_eq!(nesting_depth(&flat), Ok(1));
        assert_eq!(nesting_depth(&nested("(", ")", MAX_NESTING_DEPTH + 1)), Err(ParseError::TooDeeplyNested { offset: 8 + MAX_NESTING_DEPTH }));
        let at_limit = format!("{}{}", "val y = f()\n".repeat(MAX_NESTING_DEPTH), nested("[", "]", MAX_NESTING_DEPTH));
        assert_eq!(nesting_depth(&at_limit), Ok(MAX_NESTING_DEPTH));
    }

    #[test]
    fn brackets_in_strings_and_comments_do_not_nest() {
        let text = format!("// {0}\nval s = \"{0}\"\n/* {0} */\n", "(".repeat(MAX_NESTING_DEPTH + 1));
        assert_eq!(nesting_depth(&text), Ok(0));
    }

    #[test]
    fn template_entries_nest() {
        let text = format!("val s = {}1{}", "\"${".repeat(MAX_NESTING_DEPTH + 1), "}\"".repeat(MAX_NESTING_DEPTH + 1));
        assert!(matches!(nesting_depth(&text), Err(ParseError::TooDeeplyNested { .. })));
    }

    #[test]
    fn unmatched_closers_do_not_hide_depth() {
        let text = "{ ) ".repeat(MAX_NESTING_DEPTH + 1);
        assert!(matches!(nesting_depth(&text), Err(ParseError::TooDeeplyNested { .. })));
    }

    #[test]
    fn budget_is_capped() {
        assert_eq!(with_parser_stack(MAX_NESTING_DEPTH, usize::MAX, || 7), 7);
    }
}
