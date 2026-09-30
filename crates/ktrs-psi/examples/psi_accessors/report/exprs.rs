//! Mirrors tools/psi-accessors/src/Exprs.java.

use ktrs_psi::*;
use ktrs_syntax::SyntaxKind;

use super::{Ctx, str_value};

fn kind(k: Option<SyntaxKind>) -> String {
    k.map_or("!".to_owned(), |k| k.debug_name().to_owned())
}

pub fn describe(ctx: &mut Ctx, e: &PsiElement) {
    if let Some(x) = e.cast::<KtQualifiedExpression>() {
        ctx.line("KtQualifiedExpression.receiverExpression", ctx.req(x.receiver_expression()));
        ctx.line("KtQualifiedExpression.selectorExpression", ctx.opt(x.selector_expression()));
        ctx.line("KtQualifiedExpression.operationSign", kind(x.operation_sign().map(|t| t.0)));
    }
    if let Some(x) = e.cast::<KtArrayAccessExpression>() {
        ctx.line("KtArrayAccessExpression.arrayExpression", ctx.opt(x.array_expression()));
        ctx.line("KtArrayAccessExpression.indexExpressions", ctx.list(x.index_expressions()));
        ctx.line("KtArrayAccessExpression.trailingComma", ctx.opt(x.trailing_comma()));
    }
    if let Some(x) = e.cast::<KtUnaryExpression>() {
        ctx.line("KtUnaryExpression.baseExpression", ctx.opt(x.base_expression()));
        ctx.line("KtUnaryExpression.operationReference", ctx.req(x.operation_reference()));
    }
    if let Some(x) = e.cast::<KtBinaryExpression>() {
        ctx.line("KtBinaryExpression.left", ctx.opt(x.left()));
        ctx.line("KtBinaryExpression.right", ctx.opt(x.right()));
        ctx.line("KtBinaryExpression.operationReference", ctx.req(x.operation_reference()));
        ctx.line("KtBinaryExpression.operationToken", kind(x.operation_token()));
    }
    if let Some(x) = e.cast::<KtValueArgumentList>() {
        ctx.line("KtValueArgumentList.arguments", ctx.list(x.arguments()));
        ctx.line("KtValueArgumentList.trailingComma", ctx.opt(x.trailing_comma()));
        ctx.line("KtValueArgumentList.leftParenthesis", ctx.opt(x.left_parenthesis()));
        ctx.line("KtValueArgumentList.rightParenthesis", ctx.opt(x.right_parenthesis()));
    }
    if let Some(x) = e.cast::<KtValueArgument>() {
        ctx.line("KtValueArgument.argumentExpression", ctx.opt(x.argument_expression()));
        ctx.line("KtValueArgument.argumentName", ctx.opt(x.argument_name()));
        ctx.line("KtValueArgument.isSpread", x.is_spread().to_string());
        ctx.line("KtValueArgument.textOffset", ctx.off(x.text_offset()).to_string());
    }
    if let Some(x) = e.cast::<KtLambdaExpression>() {
        ctx.line("KtLambdaExpression.functionLiteral", ctx.req(x.function_literal()));
        ctx.line("KtLambdaExpression.valueParameters", ctx.list(x.value_parameters()));
        ctx.line("KtLambdaExpression.bodyExpression", ctx.opt(x.body_expression()));
    }
    if let Some(x) = e.cast::<KtFunctionLiteral>() {
        ctx.line("KtFunctionLiteral.arrow", ctx.opt(x.arrow()));
    }
    if let Some(x) = e.cast::<KtExpressionWithLabel>() {
        ctx.line("KtExpressionWithLabel.targetLabel", ctx.opt(x.target_label()));
        ctx.line("KtExpressionWithLabel.labelQualifier", ctx.opt(x.label_qualifier()));
    }
    if let Some(x) = e.cast::<KtReturnExpression>() {
        ctx.line("KtReturnExpression.returnedExpression", ctx.opt(x.returned_expression()));
    }
    if let Some(x) = e.cast::<KtLabeledExpression>() {
        ctx.line("KtLabeledExpression.baseExpression", ctx.opt(x.base_expression()));
    }
    if let Some(x) = e.cast::<KtSimpleNameExpression>() {
        ctx.line("KtSimpleNameExpression.identifier", ctx.opt(x.identifier()));
        ctx.line("KtSimpleNameExpression.referencedName", str_value(Some(&x.referenced_name())));
        ctx.line("KtSimpleNameExpression.referencedNameElementType", kind(x.referenced_name_element_type()));
    }
    if let Some(x) = e.cast::<KtSuperExpression>() {
        ctx.line("KtSuperExpression.superTypeQualifier", ctx.opt(x.super_type_qualifier()));
    }
    if let Some(x) = e.cast::<KtPropertyDelegate>() {
        ctx.line("KtPropertyDelegate.expression", ctx.opt(x.expression()));
    }
    if let Some(x) = e.cast::<KtParenthesizedExpression>() {
        ctx.line("KtParenthesizedExpression.expression", ctx.opt(x.expression()));
    }
    if let Some(x) = e.cast::<KtAnnotatedExpression>() {
        ctx.line("KtAnnotatedExpression.baseExpression", ctx.opt(x.base_expression()));
        ctx.line("KtAnnotatedExpression.annotationEntries", ctx.list(x.annotation_entries()));
    }
    super::control::describe(ctx, e);
    if let Some(x) = e.cast::<KtDoubleColonExpression>() {
        ctx.line("KtDoubleColonExpression.receiverExpression", ctx.opt(x.receiver_expression()));
        ctx.line("KtDoubleColonExpression.hasQuestionMarks", x.has_question_marks().to_string());
    }
    if let Some(x) = e.cast::<KtCallableReferenceExpression>() {
        ctx.line("KtCallableReferenceExpression.callableReference", ctx.req(x.callable_reference()));
    }
    if let Some(x) = e.cast::<KtCollectionLiteralExpression>() {
        ctx.line("KtCollectionLiteralExpression.innerExpressions", ctx.list(x.inner_expressions()));
        ctx.line("KtCollectionLiteralExpression.trailingComma", ctx.opt(x.trailing_comma()));
    }
    if e.is::<KDocImpl>() {
        ctx.line("KDocImpl.childrenOfType<KDocSection>", ctx.list(e.get_children_of_type::<KDocSection>()));
    }
    if e.is::<KDocTag>() {
        ctx.line("KDocTag.childrenOfType<KDocTag>", ctx.list(e.get_children_of_type::<KDocTag>()));
        ctx.line("KDocTag.childrenOfType<KDocLink>", ctx.list(e.get_children_of_type::<KDocLink>()));
    }
    if e.is::<KDocLink>() {
        ctx.line("KDocLink.childrenOfType<KDocName>", ctx.list(e.get_children_of_type::<KDocName>()));
    }
    if let Some(x) = e.cast::<KDocName>() {
        let names: Vec<String> = x.qualified_name().iter().map(|n| str_value(Some(n))).collect();
        ctx.line("KDocName.qualifiedName", format!("[{}]", names.join(", ")));
    }
}
