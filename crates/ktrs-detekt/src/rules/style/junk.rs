//! `detekt-rules-style/.../Junk.kt`: helpers shared by the style rules.

use ktrs_psi::{KtBinaryExpression, KtDotQualifiedExpression, KtExpression, KtFile, KtIfExpression, KtNamedFunction, KtProperty, KtSuperExpression, PsiElement, PsiType};
use ktrs_syntax::SyntaxKind::ELVIS;

/// Util function to search for the elements in the parents of the given line from a given offset in a `KtFile`:
/// the outermost elements inside the line's range, then the leaf at `offset`. `line_length` in bytes.
pub(super) fn find_kt_element_in_parents(file: &KtFile, offset: usize, line_length: usize) -> Vec<PsiElement> {
    let mut elements = file.elements_in_range(offset, offset + line_length);
    elements.extend(file.find_element_at(offset));
    elements
}

/// `KtNamedFunction.yieldStatementsSkippingGuardClauses<T>()`.
pub(super) fn yield_statements_skipping_guard_clauses<T: PsiType>(function: &KtNamedFunction) -> Vec<KtExpression> {
    let mut statements = Vec::new();
    let mut first_non_guard_found = false;
    for it in function.body_block_expression().map(|block| block.statements()).unwrap_or_default() {
        if first_non_guard_found {
            statements.push(it);
        } else if !is_guard_clause::<T>(&it) && !is_super_call(&it) && !is_local_variable_declaration(&it) {
            first_non_guard_found = true;
            statements.push(it);
        }
    }
    statements
}

pub(super) fn is_super_call(expression: &KtExpression) -> bool {
    expression.cast::<KtDotQualifiedExpression>().and_then(|it| it.receiver_expression()).is_some_and(|receiver| receiver.is::<KtSuperExpression>())
}

pub(super) fn is_local_variable_declaration(expression: &KtExpression) -> bool {
    expression.is::<KtProperty>()
}

pub(super) fn is_guard_clause<T: PsiType>(expression: &KtExpression) -> bool {
    let Some(descendant_expr) = expression.find_descendant_of_type::<T>(|_| true) else { return false };
    is_if_condition_guard_clause(expression, descendant_expr.psi()) || is_elvis_operator_guard_clause(expression, descendant_expr.psi())
}

pub(super) fn is_if_condition_guard_clause(expression: &KtExpression, descendant_expr: &PsiElement) -> bool {
    let Some(if_expr) = expression.cast::<KtIfExpression>() else { return false };
    if_expr.r#else().is_none() && if_expr.then().is_some_and(|then| then.last_block_statement_or_this().psi() == descendant_expr)
}

pub(super) fn is_elvis_operator_guard_clause(expression: &KtExpression, descendant_expr: &PsiElement) -> bool {
    expression.any_descendant_of_type::<KtBinaryExpression>(|it| {
        it.operation_token() == Some(ELVIS) && it.right().is_some_and(|right| right.psi() == descendant_expr)
    })
}
