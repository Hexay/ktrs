//! `KtVisitorVoid` / `KtTreeVisitorVoid` (psi-api `KtVisitor.java`, `KtVisitorVoid.java`,
//! `KtTreeVisitorVoid.java`) and `accept` dispatch.
//!
//! Every `visit_*` default forwards to its `KtVisitor` super method (e.g. `visit_dot_qualified_expression`
//! -> `visit_qualified_expression` -> `visit_expression` -> `visit_kt_element` -> `visit_element`). The
//! defaults are also free functions in [`kt_visitor_void`], which is how an override calls `super.visitX(x)`.
//! A tree visitor (`KtTreeVisitorVoid`) is a visitor whose `visit_element` calls
//! [`kt_tree_visitor_void::visit_element`] (i.e. `element.accept_children(self)`).

mod dispatch;

use crate::element::PsiElement;
use crate::types::*;

macro_rules! visitor {
    (
        forward { $($name:ident($t:ty) => $parent:ident($pt:ty);)* }
        custom { $($cname:ident($ct:ty);)* }
    ) => {
        /// `KtVisitorVoid`: override any method; defaults follow the upstream super chain.
        pub trait KtVisitorVoid {
            /// `PsiElementVisitor.visitElement`: does nothing (see `kt_tree_visitor_void` for recursion).
            fn visit_element(&mut self, _element: &PsiElement) {}
            $(fn $name(&mut self, e: &$t) { kt_visitor_void::$name(self, e) })*
            $(fn $cname(&mut self, e: &$ct) { kt_visitor_void::$cname(self, e) })*
        }

        /// Default bodies of [`KtVisitorVoid`] (`super.visitX(x)` for overrides).
        pub mod kt_visitor_void {
            #[allow(unused_imports)]
            use super::*;

            pub fn visit_element<V: KtVisitorVoid + ?Sized>(_v: &mut V, _e: &PsiElement) {}
            $(pub fn $name<V: KtVisitorVoid + ?Sized>(v: &mut V, e: &$t) { v.$parent(&e.upcast::<$pt>()) })*
            pub use super::custom::*;
        }
    };
}

visitor! {
    forward {
        visit_file(KtFile) => visit_element(PsiElement);
        visit_comment(PsiComment) => visit_element(PsiElement);
        visit_white_space(PsiWhiteSpace) => visit_element(PsiElement);
        visit_error_element(PsiErrorElement) => visit_element(PsiElement);
        visit_kt_element(KtElement) => visit_element(PsiElement);
        visit_declaration(KtDeclaration) => visit_expression(KtExpression);
        visit_class(KtClass) => visit_class_or_object(KtClassOrObject);
        visit_object_declaration(KtObjectDeclaration) => visit_class_or_object(KtClassOrObject);
        visit_class_or_object(KtClassOrObject) => visit_named_declaration(KtNamedDeclaration);
        visit_constructor(KtConstructor) => visit_named_declaration(KtNamedDeclaration);
        visit_secondary_constructor(KtSecondaryConstructor) => visit_constructor(KtConstructor);
        visit_primary_constructor(KtPrimaryConstructor) => visit_constructor(KtConstructor);
        visit_named_function(KtNamedFunction) => visit_named_declaration(KtNamedDeclaration);
        visit_property(KtProperty) => visit_named_declaration(KtNamedDeclaration);
        visit_destructuring_declaration(KtDestructuringDeclaration) => visit_declaration(KtDeclaration);
        visit_destructuring_declaration_entry(KtDestructuringDeclarationEntry) => visit_named_declaration(KtNamedDeclaration);
        visit_type_alias(KtTypeAlias) => visit_named_declaration(KtNamedDeclaration);
        visit_kt_file(KtFile) => visit_file(KtFile);
        visit_script(KtScript) => visit_declaration(KtDeclaration);
        visit_import_alias(KtImportAlias) => visit_kt_element(KtElement);
        visit_import_directive(KtImportDirective) => visit_kt_element(KtElement);
        visit_import_list(KtImportList) => visit_kt_element(KtElement);
        visit_file_annotation_list(KtFileAnnotationList) => visit_kt_element(KtElement);
        visit_class_body(KtClassBody) => visit_kt_element(KtElement);
        visit_companion_block(KtCompanionBlock) => visit_kt_element(KtElement);
        visit_modifier_list(KtModifierList) => visit_kt_element(KtElement);
        visit_annotation(KtAnnotation) => visit_kt_element(KtElement);
        visit_annotation_entry(KtAnnotationEntry) => visit_kt_element(KtElement);
        visit_annotation_use_site_target(KtAnnotationUseSiteTarget) => visit_kt_element(KtElement);
        visit_constructor_callee_expression(KtConstructorCalleeExpression) => visit_kt_element(KtElement);
        visit_type_parameter_list(KtTypeParameterList) => visit_kt_element(KtElement);
        visit_type_parameter(KtTypeParameter) => visit_named_declaration(KtNamedDeclaration);
        visit_enum_entry(KtEnumEntry) => visit_class(KtClass);
        visit_parameter_list(KtParameterList) => visit_kt_element(KtElement);
        visit_parameter(KtParameter) => visit_named_declaration(KtNamedDeclaration);
        visit_super_type_list(KtSuperTypeList) => visit_kt_element(KtElement);
        visit_super_type_list_entry(KtSuperTypeListEntry) => visit_kt_element(KtElement);
        visit_delegated_super_type_entry(KtDelegatedSuperTypeEntry) => visit_super_type_list_entry(KtSuperTypeListEntry);
        visit_super_type_call_entry(KtSuperTypeCallEntry) => visit_super_type_list_entry(KtSuperTypeListEntry);
        visit_super_type_entry(KtSuperTypeEntry) => visit_super_type_list_entry(KtSuperTypeListEntry);
        visit_context_parameter_list(KtContextParameterList) => visit_kt_element(KtElement);
        visit_context_receiver(KtContextReceiver) => visit_kt_element(KtElement);
        visit_constructor_delegation_call(KtConstructorDelegationCall) => visit_kt_element(KtElement);
        visit_property_delegate(KtPropertyDelegate) => visit_kt_element(KtElement);
        visit_type_reference(KtTypeReference) => visit_kt_element(KtElement);
        visit_value_argument_list(KtValueArgumentList) => visit_kt_element(KtElement);
        visit_argument(KtValueArgument) => visit_kt_element(KtElement);
        visit_expression(KtExpression) => visit_kt_element(KtElement);
        visit_loop_expression(KtLoopExpression) => visit_expression(KtExpression);
        visit_constant_expression(KtConstantExpression) => visit_expression(KtExpression);
        visit_simple_name_expression(KtSimpleNameExpression) => visit_reference_expression(KtReferenceExpression);
        visit_reference_expression(KtReferenceExpression) => visit_expression(KtExpression);
        visit_labeled_expression(KtLabeledExpression) => visit_expression_with_label(KtExpressionWithLabel);
        visit_prefix_expression(KtPrefixExpression) => visit_unary_expression(KtUnaryExpression);
        visit_postfix_expression(KtPostfixExpression) => visit_unary_expression(KtUnaryExpression);
        visit_unary_expression(KtUnaryExpression) => visit_expression(KtExpression);
        visit_return_expression(KtReturnExpression) => visit_expression_with_label(KtExpressionWithLabel);
        visit_expression_with_label(KtExpressionWithLabel) => visit_expression(KtExpression);
        visit_throw_expression(KtThrowExpression) => visit_expression(KtExpression);
        visit_break_expression(KtBreakExpression) => visit_expression_with_label(KtExpressionWithLabel);
        visit_continue_expression(KtContinueExpression) => visit_expression_with_label(KtExpressionWithLabel);
        visit_if_expression(KtIfExpression) => visit_expression(KtExpression);
        visit_when_expression(KtWhenExpression) => visit_expression(KtExpression);
        visit_collection_literal_expression(KtCollectionLiteralExpression) => visit_expression(KtExpression);
        visit_try_expression(KtTryExpression) => visit_expression(KtExpression);
        visit_for_expression(KtForExpression) => visit_loop_expression(KtLoopExpression);
        visit_while_expression(KtWhileExpression) => visit_loop_expression(KtLoopExpression);
        visit_do_while_expression(KtDoWhileExpression) => visit_loop_expression(KtLoopExpression);
        visit_lambda_expression(KtLambdaExpression) => visit_expression(KtExpression);
        visit_annotated_expression(KtAnnotatedExpression) => visit_expression(KtExpression);
        visit_call_expression(KtCallExpression) => visit_reference_expression(KtReferenceExpression);
        visit_array_access_expression(KtArrayAccessExpression) => visit_reference_expression(KtReferenceExpression);
        visit_qualified_expression(KtQualifiedExpression) => visit_expression(KtExpression);
        visit_double_colon_expression(KtDoubleColonExpression) => visit_expression(KtExpression);
        visit_callable_reference_expression(KtCallableReferenceExpression) => visit_double_colon_expression(KtDoubleColonExpression);
        visit_class_literal_expression(KtClassLiteralExpression) => visit_double_colon_expression(KtDoubleColonExpression);
        visit_dot_qualified_expression(KtDotQualifiedExpression) => visit_qualified_expression(KtQualifiedExpression);
        visit_safe_qualified_expression(KtSafeQualifiedExpression) => visit_qualified_expression(KtQualifiedExpression);
        visit_object_literal_expression(KtObjectLiteralExpression) => visit_expression(KtExpression);
        visit_block_expression(KtBlockExpression) => visit_expression(KtExpression);
        visit_catch_section(KtCatchClause) => visit_kt_element(KtElement);
        visit_finally_section(KtFinallySection) => visit_kt_element(KtElement);
        visit_type_argument_list(KtTypeArgumentList) => visit_kt_element(KtElement);
        visit_this_expression(KtThisExpression) => visit_expression_with_label(KtExpressionWithLabel);
        visit_super_expression(KtSuperExpression) => visit_expression_with_label(KtExpressionWithLabel);
        visit_parenthesized_expression(KtParenthesizedExpression) => visit_expression(KtExpression);
        visit_initializer_list(KtInitializerList) => visit_kt_element(KtElement);
        visit_anonymous_initializer(KtAnonymousInitializer) => visit_declaration(KtDeclaration);
        visit_script_initializer(KtScriptInitializer) => visit_anonymous_initializer(KtAnonymousInitializer);
        visit_class_initializer(KtClassInitializer) => visit_anonymous_initializer(KtAnonymousInitializer);
        visit_property_accessor(KtPropertyAccessor) => visit_declaration(KtDeclaration);
        visit_backing_field(KtBackingField) => visit_declaration(KtDeclaration);
        visit_type_constraint_list(KtTypeConstraintList) => visit_kt_element(KtElement);
        visit_type_constraint(KtTypeConstraint) => visit_kt_element(KtElement);
        // Upstream routes the type elements through a private `visitTypeElement` -> `visitKtElement`.
        visit_user_type(KtUserType) => visit_kt_element(KtElement);
        visit_dynamic_type(KtDynamicType) => visit_kt_element(KtElement);
        visit_function_type(KtFunctionType) => visit_kt_element(KtElement);
        visit_nullable_type(KtNullableType) => visit_kt_element(KtElement);
        visit_intersection_type(KtIntersectionType) => visit_kt_element(KtElement);
        visit_binary_with_type_rhs_expression(KtBinaryExpressionWithTypeRHS) => visit_expression(KtExpression);
        visit_string_template_expression(KtStringTemplateExpression) => visit_expression(KtExpression);
        visit_string_interpolation_prefix(KtStringInterpolationPrefix) => visit_kt_element(KtElement);
        visit_named_declaration(KtNamedDeclaration) => visit_declaration(KtDeclaration);
        visit_type_projection(KtTypeProjection) => visit_kt_element(KtElement);
        visit_when_entry(KtWhenEntry) => visit_kt_element(KtElement);
        visit_is_expression(KtIsExpression) => visit_expression(KtExpression);
        visit_when_condition_is_pattern(KtWhenConditionIsPattern) => visit_kt_element(KtElement);
        visit_when_condition_in_range(KtWhenConditionInRange) => visit_kt_element(KtElement);
        visit_when_condition_with_expression(KtWhenConditionWithExpression) => visit_kt_element(KtElement);
        visit_string_template_entry(KtStringTemplateEntry) => visit_kt_element(KtElement);
        visit_string_template_entry_with_expression(KtStringTemplateEntryWithExpression) => visit_string_template_entry(KtStringTemplateEntry);
        visit_block_string_template_entry(KtBlockStringTemplateEntry) => visit_string_template_entry_with_expression(KtStringTemplateEntryWithExpression);
        visit_simple_name_string_template_entry(KtSimpleNameStringTemplateEntry) => visit_string_template_entry_with_expression(KtStringTemplateEntryWithExpression);
        visit_literal_string_template_entry(KtLiteralStringTemplateEntry) => visit_string_template_entry(KtStringTemplateEntry);
        visit_escape_string_template_entry(KtEscapeStringTemplateEntry) => visit_string_template_entry(KtStringTemplateEntry);
        visit_package_directive(KtPackageDirective) => visit_kt_element(KtElement);
    }
    custom {
        visit_binary_expression(KtBinaryExpression);
        visit_context_receiver_list(KtContextReceiverList);
    }
}

mod custom {
    use super::*;
    use crate::tree_util::try_flatten_string_concatenation_descendants;

    /// Flattens string concatenations (`"a" + "b" + ...`) and accepts their parts instead of recursing
    /// through the nested binary expressions.
    pub fn visit_binary_expression<V: KtVisitorVoid + ?Sized>(v: &mut V, e: &KtBinaryExpression) {
        match try_flatten_string_concatenation_descendants(e) {
            Some(children) => {
                for child in children {
                    child.accept(v);
                }
            }
            None => v.visit_expression(&e.upcast()),
        }
    }

    /// Returns without visiting: `KtContextReceiverList.accept` already went through `visitContextParameterList`.
    pub fn visit_context_receiver_list<V: KtVisitorVoid + ?Sized>(_v: &mut V, _e: &KtContextReceiverList) {}
}

/// `KtTreeVisitorVoid`.
pub mod kt_tree_visitor_void {
    use super::KtVisitorVoid;
    use crate::element::PsiElement;

    /// `KtTreeVisitorVoid.visitElement`: `element.acceptChildren(this)`.
    pub fn visit_element<V: KtVisitorVoid + ?Sized>(v: &mut V, element: &PsiElement) {
        element.accept_children(v);
    }
}
