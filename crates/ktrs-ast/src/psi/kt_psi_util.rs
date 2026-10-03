//! psiUtil `ktPsiUtil.kt` helpers over names and expressions: `isIdentifier`, `quoteIfNeeded`,
//! `referenceExpression`, `isDotSelector`.

use ktrs_parser::kt_tokens::KEYWORDS;
use ktrs_psi::single_value;
use ktrs_syntax::SyntaxKind::*;

use super::EmbeddedKotlin;
use super::classes::*;
use crate::arena::{Ast, NodeId};

/// psiUtil `String?.isIdentifier()`. Beyond U+00FF, `Character.isLetter`/`isDigit` are approximated by
/// `char::is_alphabetic`/`is_numeric`, which agree on every code point the lexer accepts in an identifier.
// Gotcha: below U+0100, 2.2.21 takes only ASCII letters and digits, 2.4.10 also the Latin-1 letters (é, ß, ...).
pub fn is_identifier(name: &str, kotlin: EmbeddedKotlin) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.starts_with('`') {
        let surrounded = name.len() >= 2 && name.ends_with('`');
        let unescaped = if surrounded { &name[1..name.len() - 1] } else { name };
        return !unescaped.is_empty() && unescaped.chars().all(|c| c != '`' && c != '\n');
    }
    if KEYWORDS.types().filter_map(single_value).any(|k| k == name) {
        return false;
    }
    name.chars().enumerate().all(|(index, c)| {
        if c.is_ascii() || (kotlin == EmbeddedKotlin::V2_2_21 && (c as u32) < 256) {
            c == '_' || c.is_ascii_alphabetic() || (index > 0 && c.is_ascii_digit())
        } else {
            c.is_alphabetic() || (index > 0 && c.is_numeric())
        }
    })
}

/// psiUtil `String.quoteIfNeeded()`.
pub fn quote_if_needed(name: &str, kotlin: EmbeddedKotlin) -> String {
    if is_identifier(name, kotlin) { name.to_owned() } else { format!("`{name}`") }
}

/// psiUtil `KtExpression.referenceExpression()`: a call's callee, else the expression, if a `KtReferenceExpression`.
pub fn reference_expression(ast: &Ast, expression: NodeId) -> Option<KtReferenceExpression> {
    let target = match KtCallExpression::cast(ast, expression) {
        Some(call) => call.callee_expression(ast)?,
        None => expression,
    };
    KtReferenceExpression::cast(ast, target)
}

/// psiUtil `KtExpression.isDotSelector()`: the selector of its parent `KtDotQualifiedExpression`.
pub fn is_dot_selector(ast: &Ast, expression: NodeId) -> bool {
    ast.tree_parent(expression)
        .filter(|&p| ast.element_type(p) == DOT_QUALIFIED_EXPRESSION)
        .and_then(|p| KtDotQualifiedExpression(p).selector_expression(ast))
        == Some(expression)
}
