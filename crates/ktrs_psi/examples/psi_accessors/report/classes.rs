//! Mirrors tools/psi-accessors/src/Classes.java.

use ktrs_psi::*;
use ktrs_syntax::SyntaxKind::SUPER_TYPE_CALL_ENTRY;

use super::{Ctx, str_value};

pub fn describe(ctx: &mut Ctx, e: &PsiElement) {
    if let Some(x) = e.cast::<KtClassOrObject>() {
        ctx.line("KtClassOrObject.declarationKeyword", ctx.opt(x.declaration_keyword()));
        ctx.line("KtClassOrObject.primaryConstructor", ctx.opt(x.primary_constructor()));
        ctx.line("KtClassOrObject.superTypeList", ctx.opt(x.super_type_list()));
        ctx.line("KtClassOrObject.body", ctx.opt(x.body()));
    }
    if let Some(x) = e.cast::<KtClass>() {
        ctx.line("KtClass.isEnum", x.is_enum().to_string());
    }
    if let Some(x) = e.cast::<KtObjectDeclaration>() {
        ctx.line("KtObjectDeclaration.isCompanion", x.is_companion().to_string());
    }
    if let Some(x) = e.cast::<KtEnumEntry>() {
        ctx.line("KtEnumEntry.initializerList", ctx.opt(x.initializer_list()));
    }
    if let Some(x) = e.cast::<KtInitializerList>() {
        ctx.line("KtInitializerList.initializers", ctx.list(x.initializers()));
    }
    if let Some(x) = e.cast::<KtSuperTypeList>() {
        ctx.line("KtSuperTypeList.entries", ctx.list(x.entries()));
    }
    if let Some(x) = e.cast::<KtDelegatedSuperTypeEntry>() {
        ctx.line("KtDelegatedSuperTypeEntry.typeReference", ctx.opt(x.type_reference()));
        ctx.line("KtDelegatedSuperTypeEntry.delegateExpression", ctx.opt(x.delegate_expression()));
    }
    if let Some(x) = e.cast::<KtCallElement>() {
        let callee = x.callee_expression();
        let callee = if e.kind() == SUPER_TYPE_CALL_ENTRY { ctx.req(callee) } else { ctx.opt(callee) };
        ctx.line("KtCallElement.calleeExpression", callee);
        ctx.line("KtCallElement.typeArgumentList", ctx.opt(x.type_argument_list()));
        ctx.line("KtCallElement.valueArgumentList", ctx.opt(x.value_argument_list()));
        ctx.line("KtCallElement.lambdaArguments", ctx.list(x.lambda_arguments()));
    }
    if let Some(x) = e.cast::<KtConstructor>() {
        ctx.line("KtConstructor.hasConstructorKeyword", x.has_constructor_keyword().to_string());
    }
    if let Some(x) = e.cast::<KtSecondaryConstructor>() {
        ctx.line("KtSecondaryConstructor.delegationCall", ctx.req(x.delegation_call()));
    }
    if let Some(x) = e.cast::<KtConstructorDelegationCall>() {
        ctx.line("KtConstructorDelegationCall.isImplicit", x.is_implicit().to_string());
        ctx.line("KtConstructorDelegationCall.isCallToThis", x.is_call_to_this().to_string());
    }
    if let Some(x) = e.cast::<KtAnonymousInitializer>() {
        ctx.line("KtAnonymousInitializer.body", ctx.opt(x.body()));
    }
    if let Some(x) = e.cast::<KtTypeAlias>() {
        ctx.line("KtTypeAlias.typeReference", ctx.opt(x.type_reference()));
    }
    if let Some(x) = e.cast::<KtTypeParameterList>() {
        ctx.line("KtTypeParameterList.parameters", ctx.list(x.parameters()));
        ctx.line("KtTypeParameterList.trailingComma", ctx.opt(x.trailing_comma()));
    }
    if let Some(x) = e.cast::<KtTypeParameter>() {
        ctx.line("KtTypeParameter.extendsBound", ctx.opt(x.extends_bound()));
    }
    if let Some(x) = e.cast::<KtTypeConstraintList>() {
        ctx.line("KtTypeConstraintList.constraints", ctx.list(x.constraints()));
    }
    if let Some(x) = e.cast::<KtTypeConstraint>() {
        ctx.line("KtTypeConstraint.subjectTypeParameterName", ctx.opt(x.subject_type_parameter_name()));
        ctx.line("KtTypeConstraint.boundTypeReference", ctx.opt(x.bound_type_reference()));
    }
    if let Some(x) = e.cast::<KtDestructuringDeclaration>() {
        ctx.line("KtDestructuringDeclaration.valOrVarKeyword", ctx.opt(x.val_or_var_keyword()));
        ctx.line("KtDestructuringDeclaration.trailingComma", ctx.opt(x.trailing_comma()));
        ctx.line("KtDestructuringDeclaration.lPar", ctx.opt(x.l_par()));
        ctx.line("KtDestructuringDeclaration.rPar", ctx.opt(x.r_par()));
        ctx.line("KtDestructuringDeclaration.entries", ctx.list(x.entries()));
        ctx.line("KtDestructuringDeclaration.initializer", ctx.opt(x.initializer()));
    }
    if let Some(x) = e.cast::<KtDestructuringDeclarationEntry>() {
        ctx.line("KtDestructuringDeclarationEntry.initializer", ctx.opt(x.initializer()));
    }
    if let Some(x) = e.cast::<KtContextParameterList>() {
        ctx.line("KtContextParameterList.contextParameters", ctx.list(x.context_parameters()));
        ctx.line("KtContextParameterList.contextReceivers", ctx.list(x.context_receivers()));
    }
    if let Some(x) = e.cast::<KtAnnotation>() {
        ctx.line("KtAnnotation.useSiteTarget", ctx.opt(x.use_site_target()));
        ctx.line("KtAnnotation.entries", ctx.list(x.entries()));
    }
    if let Some(x) = e.cast::<KtAnnotationUseSiteTarget>() {
        let render_name = x.annotation_use_site_target().map_or("!".to_owned(), |t| str_value(Some(t.render_name())));
        ctx.line("KtAnnotationUseSiteTarget.renderName", render_name);
    }
    if let Some(x) = e.cast::<KtAnnotationEntry>() {
        ctx.line("KtAnnotationEntry.atSymbol", ctx.opt(x.at_symbol()));
        ctx.line("KtAnnotationEntry.useSiteTarget", ctx.opt(x.use_site_target()));
    }
}
