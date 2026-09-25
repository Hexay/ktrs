//! Mirrors tools/psi-accessors/src/Control.java.

use ktrs_psi::*;

use super::Ctx;

pub fn describe(ctx: &mut Ctx, e: &PsiElement) {
    if let Some(x) = e.cast::<KtWhenExpression>() {
        ctx.line("KtWhenExpression.subjectExpression", ctx.opt(x.subject_expression()));
        ctx.line("KtWhenExpression.entries", ctx.list(x.entries()));
    }
    if let Some(x) = e.cast::<KtWhenEntry>() {
        ctx.line("KtWhenEntry.elseKeyword", ctx.opt(x.else_keyword()));
        ctx.line("KtWhenEntry.conditions", ctx.list(x.conditions()));
        ctx.line("KtWhenEntry.guard", ctx.opt(x.guard()));
        ctx.line("KtWhenEntry.expression", ctx.opt(x.expression()));
        ctx.line("KtWhenEntry.trailingComma", ctx.opt(x.trailing_comma()));
    }
    if let Some(x) = e.cast::<KtWhenEntryGuard>() {
        ctx.line("KtWhenEntryGuard.expression", ctx.opt(x.expression()));
    }
    if let Some(x) = e.cast::<KtWhenConditionWithExpression>() {
        ctx.line("KtWhenConditionWithExpression.expression", ctx.opt(x.expression()));
    }
    if let Some(x) = e.cast::<KtWhenConditionIsPattern>() {
        ctx.line("KtWhenConditionIsPattern.isNegated", x.is_negated().to_string());
        ctx.line("KtWhenConditionIsPattern.typeReference", ctx.opt(x.type_reference()));
    }
    if let Some(x) = e.cast::<KtWhenConditionInRange>() {
        ctx.line("KtWhenConditionInRange.rangeExpression", ctx.opt(x.range_expression()));
    }
    if let Some(x) = e.cast::<KtIfExpression>() {
        ctx.line("KtIfExpression.condition", ctx.opt(x.condition()));
        ctx.line("KtIfExpression.then", ctx.opt(x.then()));
        ctx.line("KtIfExpression.elseKeyword", ctx.opt(x.else_keyword()));
        ctx.line("KtIfExpression.else", ctx.opt(x.r#else()));
    }
    if let Some(x) = e.cast::<KtLoopExpression>() {
        ctx.line("KtLoopExpression.body", ctx.opt(x.body()));
    }
    if let Some(x) = e.cast::<KtForExpression>() {
        ctx.line("KtForExpression.loopParameter", ctx.opt(x.loop_parameter()));
        ctx.line("KtForExpression.loopRange", ctx.opt(x.loop_range()));
    }
    if let Some(x) = e.cast::<KtWhileExpressionBase>() {
        ctx.line("KtWhileExpressionBase.condition", ctx.opt(x.condition()));
    }
    if let Some(x) = e.cast::<KtIsExpression>() {
        ctx.line("KtIsExpression.leftHandSide", ctx.opt(x.left_hand_side()));
        ctx.line("KtIsExpression.operationReference", ctx.opt(x.operation_reference()));
        ctx.line("KtIsExpression.typeReference", ctx.opt(x.type_reference()));
    }
    if let Some(x) = e.cast::<KtBinaryExpressionWithTypeRHS>() {
        ctx.line("KtBinaryExpressionWithTypeRHS.left", ctx.opt(x.left()));
        ctx.line("KtBinaryExpressionWithTypeRHS.operationReference", ctx.opt(x.operation_reference()));
        ctx.line("KtBinaryExpressionWithTypeRHS.right", ctx.opt(x.right()));
    }
    if let Some(x) = e.cast::<KtTryExpression>() {
        ctx.line("KtTryExpression.tryBlock", ctx.opt(x.try_block()));
        ctx.line("KtTryExpression.catchClauses", ctx.list(x.catch_clauses()));
        ctx.line("KtTryExpression.finallyBlock", ctx.opt(x.finally_block()));
    }
    if let Some(x) = e.cast::<KtCatchClause>() {
        ctx.line("KtCatchClause.catchParameter", ctx.opt(x.catch_parameter()));
        ctx.line("KtCatchClause.catchBody", ctx.opt(x.catch_body()));
    }
    if let Some(x) = e.cast::<KtFinallySection>() {
        ctx.line("KtFinallySection.finalExpression", ctx.opt(x.final_expression()));
    }
    if let Some(x) = e.cast::<KtThrowExpression>() {
        ctx.line("KtThrowExpression.thrownExpression", ctx.opt(x.thrown_expression()));
    }
}
