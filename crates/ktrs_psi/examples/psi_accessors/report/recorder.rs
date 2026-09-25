//! Mirrors tools/psi-accessors/src/Recorder.java: records every visit method it receives. Methods the Java
//! recorder can't observe (`visitImportAlias`, `visitBackingField` have no `KtVisitorVoid` bridge) are left
//! at their defaults here too.

use ktrs_psi::*;

use super::Ctx;

struct Recorder<'c, 'p> {
    ctx: &'c Ctx<'p>,
    tree: bool,
    out: String,
}

impl Recorder<'_, '_> {
    fn r(&mut self, name: &str, e: &PsiElement) {
        if self.tree {
            self.out.push_str(&format!(" {name} {}\n", self.ctx.ref_of(e)));
        } else {
            if !self.out.is_empty() {
                self.out.push(',');
            }
            self.out.push_str(name);
        }
    }
}

pub fn dispatch_chain(ctx: &Ctx, e: &PsiElement) -> String {
    let mut recorder = Recorder { ctx, tree: false, out: String::new() };
    e.accept(&mut recorder);
    recorder.out
}

pub fn walk(ctx: &Ctx, file: &KtFile) -> String {
    let mut recorder = Recorder { ctx, tree: true, out: String::new() };
    file.accept(&mut recorder);
    recorder.out
}

macro_rules! record {
    ($($name:ident($t:ty) = $java:literal;)*) => {
        impl KtVisitorVoid for Recorder<'_, '_> {
            fn visit_element(&mut self, e: &PsiElement) {
                self.r("visitElement", e);
                if self.tree {
                    kt_tree_visitor_void::visit_element(self, e);
                }
            }

            $(fn $name(&mut self, e: &$t) {
                self.r($java, e);
                kt_visitor_void::$name(self, e);
            })*
        }
    };
}

record! {
    visit_file(KtFile) = "visitFile";
    visit_comment(PsiComment) = "visitComment";
    visit_white_space(PsiWhiteSpace) = "visitWhiteSpace";
    visit_error_element(PsiErrorElement) = "visitErrorElement";
    visit_kt_element(KtElement) = "visitKtElement";
    visit_declaration(KtDeclaration) = "visitDeclaration";
    visit_class(KtClass) = "visitClass";
    visit_class_or_object(KtClassOrObject) = "visitClassOrObject";
    visit_constructor(KtConstructor) = "visitConstructor";
    visit_secondary_constructor(KtSecondaryConstructor) = "visitSecondaryConstructor";
    visit_primary_constructor(KtPrimaryConstructor) = "visitPrimaryConstructor";
    visit_named_function(KtNamedFunction) = "visitNamedFunction";
    visit_property(KtProperty) = "visitProperty";
    visit_type_alias(KtTypeAlias) = "visitTypeAlias";
    visit_destructuring_declaration(KtDestructuringDeclaration) = "visitDestructuringDeclaration";
    visit_destructuring_declaration_entry(KtDestructuringDeclarationEntry) = "visitDestructuringDeclarationEntry";
    visit_kt_file(KtFile) = "visitKtFile";
    visit_script(KtScript) = "visitScript";
    visit_import_directive(KtImportDirective) = "visitImportDirective";
    visit_import_list(KtImportList) = "visitImportList";
    visit_class_body(KtClassBody) = "visitClassBody";
    visit_companion_block(KtCompanionBlock) = "visitCompanionBlock";
    visit_modifier_list(KtModifierList) = "visitModifierList";
    visit_annotation(KtAnnotation) = "visitAnnotation";
    visit_annotation_entry(KtAnnotationEntry) = "visitAnnotationEntry";
    visit_constructor_callee_expression(KtConstructorCalleeExpression) = "visitConstructorCalleeExpression";
    visit_type_parameter_list(KtTypeParameterList) = "visitTypeParameterList";
    visit_type_parameter(KtTypeParameter) = "visitTypeParameter";
    visit_enum_entry(KtEnumEntry) = "visitEnumEntry";
    visit_parameter_list(KtParameterList) = "visitParameterList";
    visit_parameter(KtParameter) = "visitParameter";
    visit_super_type_list(KtSuperTypeList) = "visitSuperTypeList";
    visit_super_type_list_entry(KtSuperTypeListEntry) = "visitSuperTypeListEntry";
    visit_delegated_super_type_entry(KtDelegatedSuperTypeEntry) = "visitDelegatedSuperTypeEntry";
    visit_super_type_call_entry(KtSuperTypeCallEntry) = "visitSuperTypeCallEntry";
    visit_super_type_entry(KtSuperTypeEntry) = "visitSuperTypeEntry";
    visit_context_receiver_list(KtContextReceiverList) = "visitContextReceiverList";
    visit_context_parameter_list(KtContextParameterList) = "visitContextParameterList";
    visit_context_receiver(KtContextReceiver) = "visitContextReceiver";
    visit_constructor_delegation_call(KtConstructorDelegationCall) = "visitConstructorDelegationCall";
    visit_property_delegate(KtPropertyDelegate) = "visitPropertyDelegate";
    visit_type_reference(KtTypeReference) = "visitTypeReference";
    visit_value_argument_list(KtValueArgumentList) = "visitValueArgumentList";
    visit_argument(KtValueArgument) = "visitArgument";
    visit_expression(KtExpression) = "visitExpression";
    visit_loop_expression(KtLoopExpression) = "visitLoopExpression";
    visit_constant_expression(KtConstantExpression) = "visitConstantExpression";
    visit_simple_name_expression(KtSimpleNameExpression) = "visitSimpleNameExpression";
    visit_reference_expression(KtReferenceExpression) = "visitReferenceExpression";
    visit_labeled_expression(KtLabeledExpression) = "visitLabeledExpression";
    visit_prefix_expression(KtPrefixExpression) = "visitPrefixExpression";
    visit_postfix_expression(KtPostfixExpression) = "visitPostfixExpression";
    visit_unary_expression(KtUnaryExpression) = "visitUnaryExpression";
    visit_binary_expression(KtBinaryExpression) = "visitBinaryExpression";
    visit_return_expression(KtReturnExpression) = "visitReturnExpression";
    visit_expression_with_label(KtExpressionWithLabel) = "visitExpressionWithLabel";
    visit_throw_expression(KtThrowExpression) = "visitThrowExpression";
    visit_break_expression(KtBreakExpression) = "visitBreakExpression";
    visit_continue_expression(KtContinueExpression) = "visitContinueExpression";
    visit_if_expression(KtIfExpression) = "visitIfExpression";
    visit_when_expression(KtWhenExpression) = "visitWhenExpression";
    visit_collection_literal_expression(KtCollectionLiteralExpression) = "visitCollectionLiteralExpression";
    visit_try_expression(KtTryExpression) = "visitTryExpression";
    visit_for_expression(KtForExpression) = "visitForExpression";
    visit_while_expression(KtWhileExpression) = "visitWhileExpression";
    visit_do_while_expression(KtDoWhileExpression) = "visitDoWhileExpression";
    visit_lambda_expression(KtLambdaExpression) = "visitLambdaExpression";
    visit_annotated_expression(KtAnnotatedExpression) = "visitAnnotatedExpression";
    visit_call_expression(KtCallExpression) = "visitCallExpression";
    visit_array_access_expression(KtArrayAccessExpression) = "visitArrayAccessExpression";
    visit_qualified_expression(KtQualifiedExpression) = "visitQualifiedExpression";
    visit_double_colon_expression(KtDoubleColonExpression) = "visitDoubleColonExpression";
    visit_callable_reference_expression(KtCallableReferenceExpression) = "visitCallableReferenceExpression";
    visit_class_literal_expression(KtClassLiteralExpression) = "visitClassLiteralExpression";
    visit_dot_qualified_expression(KtDotQualifiedExpression) = "visitDotQualifiedExpression";
    visit_safe_qualified_expression(KtSafeQualifiedExpression) = "visitSafeQualifiedExpression";
    visit_object_literal_expression(KtObjectLiteralExpression) = "visitObjectLiteralExpression";
    visit_block_expression(KtBlockExpression) = "visitBlockExpression";
    visit_catch_section(KtCatchClause) = "visitCatchSection";
    visit_finally_section(KtFinallySection) = "visitFinallySection";
    visit_type_argument_list(KtTypeArgumentList) = "visitTypeArgumentList";
    visit_this_expression(KtThisExpression) = "visitThisExpression";
    visit_super_expression(KtSuperExpression) = "visitSuperExpression";
    visit_parenthesized_expression(KtParenthesizedExpression) = "visitParenthesizedExpression";
    visit_initializer_list(KtInitializerList) = "visitInitializerList";
    visit_anonymous_initializer(KtAnonymousInitializer) = "visitAnonymousInitializer";
    visit_script_initializer(KtScriptInitializer) = "visitScriptInitializer";
    visit_class_initializer(KtClassInitializer) = "visitClassInitializer";
    visit_property_accessor(KtPropertyAccessor) = "visitPropertyAccessor";
    visit_type_constraint_list(KtTypeConstraintList) = "visitTypeConstraintList";
    visit_type_constraint(KtTypeConstraint) = "visitTypeConstraint";
    visit_user_type(KtUserType) = "visitUserType";
    visit_dynamic_type(KtDynamicType) = "visitDynamicType";
    visit_function_type(KtFunctionType) = "visitFunctionType";
    visit_binary_with_type_rhs_expression(KtBinaryExpressionWithTypeRHS) = "visitBinaryWithTypeRHSExpression";
    visit_string_template_expression(KtStringTemplateExpression) = "visitStringTemplateExpression";
    visit_named_declaration(KtNamedDeclaration) = "visitNamedDeclaration";
    visit_nullable_type(KtNullableType) = "visitNullableType";
    visit_intersection_type(KtIntersectionType) = "visitIntersectionType";
    visit_type_projection(KtTypeProjection) = "visitTypeProjection";
    visit_when_entry(KtWhenEntry) = "visitWhenEntry";
    visit_is_expression(KtIsExpression) = "visitIsExpression";
    visit_when_condition_is_pattern(KtWhenConditionIsPattern) = "visitWhenConditionIsPattern";
    visit_when_condition_in_range(KtWhenConditionInRange) = "visitWhenConditionInRange";
    visit_when_condition_with_expression(KtWhenConditionWithExpression) = "visitWhenConditionWithExpression";
    visit_object_declaration(KtObjectDeclaration) = "visitObjectDeclaration";
    visit_string_template_entry(KtStringTemplateEntry) = "visitStringTemplateEntry";
    visit_string_template_entry_with_expression(KtStringTemplateEntryWithExpression) = "visitStringTemplateEntryWithExpression";
    visit_block_string_template_entry(KtBlockStringTemplateEntry) = "visitBlockStringTemplateEntry";
    visit_simple_name_string_template_entry(KtSimpleNameStringTemplateEntry) = "visitSimpleNameStringTemplateEntry";
    visit_literal_string_template_entry(KtLiteralStringTemplateEntry) = "visitLiteralStringTemplateEntry";
    visit_escape_string_template_entry(KtEscapeStringTemplateEntry) = "visitEscapeStringTemplateEntry";
    visit_package_directive(KtPackageDirective) = "visitPackageDirective";
    visit_file_annotation_list(KtFileAnnotationList) = "visitFileAnnotationList";
    visit_annotation_use_site_target(KtAnnotationUseSiteTarget) = "visitAnnotationUseSiteTarget";
    visit_string_interpolation_prefix(KtStringInterpolationPrefix) = "visitStringInterpolationPrefix";
}
