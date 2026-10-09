//! Mirrors tools/psi-accessors/src/Members.java.

use ktrs_psi::*;
use ktrs_syntax::SyntaxKind::*;

use super::{Ctx, str_value};

pub fn describe(ctx: &mut Ctx, e: &PsiElement) {
    ctx.line("textOffset", text_offset(ctx, e));
    ctx.line("startOffsetSkippingComments", ctx.off(e.start_offset_skipping_comments()).to_string());
    ctx.line("findDescendantOfType<KtProperty>", ctx.opt(e.find_descendant_of_type::<KtProperty>(|_| true)));
    ctx.line("findChildrenOfType<KtNamedFunction>", ctx.list(e.find_children_of_type::<KtNamedFunction>()));
    if let Some(x) = e.cast::<KtAnnotated>() {
        ctx.line("KtAnnotated.annotationEntries", ctx.list(x.annotation_entries()));
    }
    if let Some(x) = e.cast::<KtModifierListOwner>() {
        let visibility = x.visibility_modifier_type().map_or("null", |kind| kind.debug_name());
        ctx.line("KtModifierListOwner.visibilityModifierType", visibility.to_owned());
        ctx.line("KtModifierListOwner.isPrivate", x.is_private().to_string());
        ctx.line("KtModifierListOwner.hasExpectModifier", x.has_expect_modifier().to_string());
    }
    if let Some(x) = e.cast::<KtNamedDeclaration>()
        && e.kind() != SCRIPT
    {
        ctx.line("KtNamedDeclaration.name", str_value(x.name().as_deref()));
        let safe_name = x.name_as_safe_name();
        ctx.line("KtNamedDeclaration.nameAsSafeName", format!("{} {}", str_value(Some(safe_name.as_string())), safe_name.is_special()));
    }
    if let Some(x) = e.cast::<KtTypeParameterListOwner>() {
        ctx.line("KtTypeParameterListOwner.typeParameters", ctx.list(x.type_parameters()));
    }
    if let Some(x) = e.cast::<KtDeclaration>() {
        ctx.line("KtDeclaration.containingClassOrObject", ctx.opt(x.containing_class_or_object()));
        ctx.line("KtDeclaration.containingClass", ctx.opt(x.containing_class()));
    }
    if let Some(x) = e.cast::<KtClassOrObject>() {
        ctx.line("KtClassOrObject.declarations", ctx.list(x.declarations()));
        ctx.line("KtClassOrObject.secondaryConstructors", ctx.list(x.secondary_constructors()));
        ctx.line("KtClassOrObject.superTypeListEntries", ctx.list(x.super_type_list_entries()));
        ctx.line("KtClassOrObject.isTopLevel", x.is_top_level().to_string());
        ctx.line("KtClassOrObject.isObjectLiteral", x.is_object_literal().to_string());
    }
    if let Some(x) = e.cast::<KtClass>() {
        ctx.line("KtClass.isInterface", x.is_interface().to_string());
    }
    if let Some(x) = e.cast::<KtFile>() {
        ctx.line("KtFile.declarations", ctx.list(x.declarations()));
        ctx.line("KtFile.packageDirective", ctx.opt(x.package_directive()));
        ctx.line("KtFile.packageFqName", str_value(Some(x.package_fq_name().as_string())));
        lines(ctx, &x);
    }
    if let Some(x) = e.cast::<KtNamedFunction>() {
        ctx.line("KtNamedFunction.isTopLevel", x.is_top_level().to_string());
        ctx.line("KtNamedFunction.hasBody", x.has_body().to_string());
        ctx.line("KtNamedFunction.isLocal", x.is_local().to_string());
    }
    if let Some(x) = e.cast::<KtProperty>() {
        ctx.line("KtProperty.isTopLevel", x.is_top_level().to_string());
        ctx.line("KtProperty.isMember", x.is_member().to_string());
        ctx.line("KtProperty.isLocal", x.is_local().to_string());
    }
    if let Some(x) = e.cast::<KtParameter>() {
        ctx.line("KtParameter.ownerFunction", ctx.opt(x.owner_function()));
    }
    if let Some(x) = e.cast::<KtBlockExpression>() {
        ctx.line("KtBlockExpression.statements", ctx.list(x.statements()));
    }
    if let Some(x) = e.cast::<KtReturnExpression>() {
        ctx.line("KtReturnExpression.labeledExpression", ctx.opt(x.labeled_expression()));
    }
    if let Some(x) = e.cast::<KtLambdaArgument>() {
        ctx.line("KtLambdaArgument.lambdaExpression", ctx.opt(x.get_lambda_expression()));
    }
    if let Some(x) = e.cast::<KtCallElement>() {
        ctx.line("KtCallElement.callNameExpression", ctx.opt(x.get_call_name_expression()));
        ctx.line("KtCallElement.valueArguments", ctx.list(x.value_arguments()));
    }
    if let Some(x) = e.cast::<KtExpression>() {
        // Upstream's `parent.receiverExpression` throws where a qualified parent has no receiver.
        let receiver_throws = e.parent().and_then(|p| p.cast::<KtQualifiedExpression>()).is_some_and(|q| q.receiver_expression().is_none());
        let qualified = if receiver_throws { "!".to_owned() } else { ctx.ref_of(&x.get_qualified_expression_for_receiver_or_this()) };
        ctx.line("KtExpression.qualifiedExpressionForReceiverOrThis", qualified);
        ctx.line("KtExpression.lastBlockStatementOrThis", ctx.ref_of(&x.last_block_statement_or_this()));
    }
}

/// `getTextOffset()`; `!` where upstream's `!!` on the `object` / accessor keyword throws.
fn text_offset(ctx: &Ctx, e: &PsiElement) -> String {
    let has = |kind| e.find_child_by_type::<PsiElement>(kind).is_some();
    let throws = !e.is_leaf()
        && !e.is_file()
        && match e.kind() {
            OBJECT_DECLARATION => !has(IDENTIFIER) && !has(OBJECT_KEYWORD),
            PROPERTY_ACCESSOR => !has(GET_KEYWORD) && !has(SET_KEYWORD),
            _ => false,
        };
    if throws { "!".to_owned() } else { ctx.off(e.text_offset()).to_string() }
}

/// Per line of the file: `findElementAt(lineStart)` and `elementsInRange(line)`.
fn lines(ctx: &mut Ctx, file: &KtFile) {
    let text = file.text();
    let mut start = 0;
    for line in text.split('\n') {
        let end = start + line.len();
        let (from, to) = (ctx.off(start), ctx.off(end));
        ctx.line(&format!("KtFile.findElementAt({from})"), ctx.opt(file.find_element_at(start)));
        ctx.line(&format!("KtFile.elementsInRange({from}..{to})"), ctx.list(file.elements_in_range(start, end)));
        start = end + 1;
    }
}
