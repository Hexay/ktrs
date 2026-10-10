//! Port of `visitor/PsiUtils.kt` (`isLambda`, `hasEmptyParenthesis`, `callExpression`,
//! `topLevelAnnotations`; its scoping and chain helpers are in the visitor's `scoping.rs` and `qualified.rs`).
//! `KtParameterList.hasEmptyParenthesis` is the visitor's `ParameterList::has_empty_parens`.

use ktrs_psi::{
    KtAnnotatedExpression, KtAnnotation, KtAnnotationEntry, KtCallExpression, KtExpression, KtQualifiedExpression,
    KtValueArgumentList, PsiElement,
};

/// Returns true if the expression represents an invocation that is also a lambda.
pub fn is_lambda(expression: &KtExpression) -> bool {
    call_expression(expression).is_some_and(|c| !c.lambda_arguments().is_empty())
}

/// Does this list have parens with only whitespace between them?
pub fn value_argument_list_has_empty_parens(list: &KtValueArgumentList) -> bool {
    parens_have_only_whitespace_between(list.left_parenthesis(), list.right_parenthesis())
}

/// The shared body of both `hasEmptyParens` extensions.
pub fn parens_have_only_whitespace_between(left: Option<PsiElement>, right: Option<PsiElement>) -> bool {
    let (Some(left), Some(right)) = (left, right) else { return false };
    left.get_next_sibling_ignoring_whitespace(false) == Some(right)
}

/// `KtAnnotatedExpression.topLevelAnnotations`: for `@[A B] @C foo()`, the `KtAnnotation` `@[A B]`
/// and the `KtAnnotationEntry` `@C` (`annotationEntries` would flatten the former).
pub fn top_level_annotations(expression: &KtAnnotatedExpression) -> Vec<PsiElement> {
    expression.children().into_iter().filter(|it| it.is::<KtAnnotation>() || it.is::<KtAnnotationEntry>()).collect()
}

/// Call expressions standing alone or as the selector of a qualified expression.
fn call_expression(expression: &KtExpression) -> Option<KtCallExpression> {
    match expression.cast::<KtQualifiedExpression>() {
        Some(qualified) => qualified.selector_expression()?.cast(),
        None => expression.cast(),
    }
}
