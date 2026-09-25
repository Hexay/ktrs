//! Mirrors tools/psi-accessors/src/Decls.java.

use ktrs_psi::*;
use ktrs_syntax::SyntaxKind::*;

use super::{Ctx, str_value};

pub fn describe(ctx: &mut Ctx, e: &PsiElement) {
    if let Some(x) = e.cast::<KtFile>() {
        ctx.line("KtFile.importList", ctx.opt(x.import_list()));
    }
    if let Some(x) = e.cast::<PsiErrorElement>() {
        ctx.line("PsiErrorElement.errorDescription", str_value(Some(x.error_description(ctx.parse))));
    }
    if let Some(x) = e.cast::<KtModifierListOwner>() {
        ctx.line("KtModifierListOwner.modifierList", ctx.opt(x.modifier_list()));
    }
    if let Some(identifier) = name_identifier(e) {
        ctx.line("PsiNameIdentifierOwner.nameIdentifier", ctx.opt(identifier));
    }
    if matches!(e.kind(), FUN | CLASS | OBJECT_DECLARATION | ENUM_ENTRY | SECONDARY_CONSTRUCTOR) && !e.is_leaf() {
        ctx.line(
            "getStubOrPsiChild(CONTEXT_PARAMETER_LIST)",
            ctx.opt(e.get_stub_or_psi_child::<PsiElement>(CONTEXT_PARAMETER_LIST)),
        );
    }
    if let Some(x) = e.cast::<KtTypeParameterListOwner>() {
        ctx.line("KtTypeParameterListOwner.typeParameterList", ctx.opt(x.type_parameter_list()));
        ctx.line("KtTypeParameterListOwner.typeConstraintList", ctx.opt(x.type_constraint_list()));
    }
    if let Some(x) = e.cast::<KtCallableDeclaration>() {
        ctx.line("KtCallableDeclaration.receiverTypeReference", ctx.opt(x.receiver_type_reference()));
        ctx.line("KtCallableDeclaration.typeReference", ctx.opt(x.type_reference()));
        ctx.line("KtCallableDeclaration.valueParameterList", ctx.opt(x.value_parameter_list()));
    }
    if let Some(x) = e.cast::<KtDeclarationWithBody>() {
        ctx.line("KtDeclarationWithBody.bodyExpression", ctx.opt(x.body_expression()));
        ctx.line("KtDeclarationWithBody.bodyBlockExpression", ctx.opt(x.body_block_expression()));
        ctx.line("KtDeclarationWithBody.valueParameters", ctx.list(x.value_parameters()));
    }
    if let Some(x) = e.cast::<KtProperty>() {
        ctx.line("KtProperty.valOrVarKeyword", ctx.opt(x.val_or_var_keyword()));
        ctx.line("KtProperty.delegate", ctx.opt(x.delegate()));
        ctx.line("KtProperty.initializer", ctx.opt(x.initializer()));
        ctx.line("KtProperty.accessors", ctx.list(x.accessors()));
        ctx.line("KtProperty.fieldDeclaration", ctx.opt(x.field_declaration()));
        ctx.line("KtProperty.getter", ctx.opt(x.getter()));
        ctx.line("KtProperty.setter", ctx.opt(x.setter()));
    }
    if let Some(x) = e.cast::<KtPropertyAccessor>() {
        ctx.line("KtPropertyAccessor.namePlaceholder", ctx.req(x.name_placeholder()));
        ctx.line("KtPropertyAccessor.returnTypeReference", ctx.opt(x.return_type_reference()));
        ctx.line("KtPropertyAccessor.parameterList", ctx.opt(x.parameter_list()));
    }
    if let Some(x) = e.cast::<KtBackingField>() {
        ctx.line("KtBackingField.namePlaceholder", ctx.opt(Some(x.name_placeholder())));
        ctx.line("KtBackingField.returnTypeReference", ctx.opt(x.return_type_reference()));
        ctx.line("KtBackingField.initializer", ctx.opt(x.initializer()));
    }
    if let Some(x) = e.cast::<KtParameter>() {
        ctx.line("KtParameter.destructuringDeclaration", ctx.opt(x.destructuring_declaration()));
        ctx.line("KtParameter.valOrVarKeyword", ctx.opt(x.val_or_var_keyword()));
        ctx.line("KtParameter.defaultValue", ctx.opt(x.default_value()));
    }
    if let Some(x) = e.cast::<KtParameterList>() {
        ctx.line("KtParameterList.parameters", ctx.list(x.parameters()));
        ctx.line("KtParameterList.trailingComma", ctx.opt(x.trailing_comma()));
        ctx.line("KtParameterList.leftParenthesis", ctx.opt(x.left_parenthesis()));
        ctx.line("KtParameterList.rightParenthesis", ctx.opt(x.right_parenthesis()));
    }
    if let Some(x) = e.cast::<KtTypeReference>() {
        ctx.line("KtTypeReference.typeElement", ctx.opt(x.type_element()));
    }
    if let Some(x) = e.cast::<KtNullableType>() {
        ctx.line("KtNullableType.modifierList", ctx.opt(x.modifier_list()));
        ctx.line("KtNullableType.innerType", ctx.opt(x.inner_type()));
    }
    if let Some(x) = e.cast::<KtUserType>() {
        ctx.line("KtUserType.qualifier", ctx.opt(x.qualifier()));
        ctx.line("KtUserType.referenceExpression", ctx.opt(x.reference_expression()));
        ctx.line("KtUserType.typeArgumentList", ctx.opt(x.type_argument_list()));
    }
    if let Some(x) = e.cast::<KtIntersectionType>() {
        ctx.line("KtIntersectionType.leftTypeRef", ctx.opt(x.left_type_ref()));
        ctx.line("KtIntersectionType.rightTypeRef", ctx.opt(x.right_type_ref()));
    }
    if let Some(x) = e.cast::<KtTypeArgumentList>() {
        ctx.line("KtTypeArgumentList.arguments", ctx.list(x.arguments()));
        ctx.line("KtTypeArgumentList.trailingComma", ctx.opt(x.trailing_comma()));
    }
    if let Some(x) = e.cast::<KtTypeProjection>() {
        ctx.line("KtTypeProjection.typeReference", ctx.opt(x.type_reference()));
        ctx.line("KtTypeProjection.projectionKind", x.projection_kind().name().to_owned());
    }
    if let Some(x) = e.cast::<KtFunctionType>() {
        ctx.line("KtFunctionType.contextReceiverList", ctx.opt(x.context_receiver_list()));
        ctx.line("KtFunctionType.receiver", ctx.opt(x.receiver()));
        ctx.line("KtFunctionType.parameterList", ctx.opt(x.parameter_list()));
        ctx.line("KtFunctionType.returnTypeReference", ctx.opt(x.return_type_reference()));
    }
    super::classes::describe(ctx, e);
    if let Some(x) = e.cast::<KtPackageDirective>() {
        ctx.line("KtPackageDirective.packageKeyword", ctx.opt(x.package_keyword()));
        ctx.line("KtPackageDirective.packageNames", ctx.list(x.package_names()));
        ctx.line("KtPackageDirective.fqName", str_value(Some(x.fq_name().as_string())));
    }
    if let Some(x) = e.cast::<KtImportList>() {
        ctx.line("KtImportList.imports", ctx.list(x.imports()));
    }
    if let Some(x) = e.cast::<KtImportDirective>() {
        ctx.line("KtImportDirective.importedReference", ctx.opt(x.imported_reference()));
        ctx.line("KtImportDirective.isAllUnder", x.is_all_under().to_string());
        ctx.line("KtImportDirective.alias", ctx.opt(x.alias()));
        ctx.line("KtImportDirective.importedFqName", fq_name(x.imported_fq_name()));
        ctx.line("KtImportDirective.isValidImport", x.is_valid_import().to_string());
        let imported_name = x.import_path().map_or("null".to_owned(), |p| str_value(p.imported_name().as_deref()));
        ctx.line("KtImportDirective.importPath.importedName", imported_name);
    }
    if let Some(x) = e.cast::<KtScript>() {
        ctx.line("KtScript.blockExpression", ctx.req(x.block_expression()));
    }
}

/// `PsiNameIdentifierOwner` implementors: named declarations, import aliases, labeled expressions.
fn name_identifier(e: &PsiElement) -> Option<Option<PsiElement>> {
    if let Some(x) = e.cast::<KtNamedDeclaration>() {
        return Some(x.name_identifier());
    }
    if let Some(x) = e.cast::<KtImportAlias>() {
        return Some(x.name_identifier());
    }
    Some(e.cast::<KtLabeledExpression>()?.name_identifier())
}

fn fq_name(fq: Option<FqName>) -> String {
    let Some(fq) = fq else { return "null".to_owned() };
    let short_name = fq.short_name().map_or("!".to_owned(), |s| str_value(Some(&s)));
    let parent = fq.parent().map_or("!".to_owned(), |p| str_value(Some(p.as_string())));
    format!("{} short={short_name} parent={parent}", str_value(Some(fq.as_string())))
}
