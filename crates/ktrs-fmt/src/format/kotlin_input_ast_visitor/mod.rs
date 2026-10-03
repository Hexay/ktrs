//! Port of `KotlinInputAstVisitor.kt`: an AST visitor that builds a stream of `Op`s to format. This
//! file covers lines 141-168 (state) and routes each `override fun visitX` to the inherent method of
//! the same name; the methods live in the sibling files in upstream order (each file names its range).
//! Upstream exceptions become `OpsBuilder::fail` plus an early return (see `format` module docs).

mod annotations;
mod calls;
mod classes;
mod comma_separated;
mod control_flow;
mod declarations;
mod expressions;
mod function_like;
mod helpers;
mod misc;
mod operators;
mod qualified;
mod scoping;
mod statements;
mod types;

use ktrs_psi::*;

use crate::doc::{FormatterDiagnostic, Indent, OpsBuilder};

use super::FormatError;
use super::formatting_options::FormattingOptions;

pub struct KotlinInputAstVisitor<'b, 'a, 'o> {
    options: FormattingOptions,
    builder: &'b mut OpsBuilder<'a, 'o>,
    /// Standard indentation for a block.
    block_indent: Indent,
    /// Indentation for a long expression or function call; differs from block indentation on purpose.
    expression_break_indent: Indent,
    block_plus_expression_break_indent: Indent,
    double_expression_break_indent: Indent,
    expression_break_negative_indent: Indent,
    /// A record of whether we have visited into an expression.
    in_expression: Vec<bool>,
    /// Tracks whether we are handling an import directive.
    in_import: bool,
    /// The non-`FormattingError` exception upstream throws first, if any, with the diagnostic of
    /// the `FormattingError` an enclosing `visitElement` turns it into (see [`Self::throw`]).
    exception: Option<(FormatError, FormatterDiagnostic)>,
}

impl<'b, 'a, 'o> KotlinInputAstVisitor<'b, 'a, 'o> {
    pub fn new(options: FormattingOptions, builder: &'b mut OpsBuilder<'a, 'o>) -> Self {
        KotlinInputAstVisitor {
            block_indent: Indent::make_const(options.block_indent, 1),
            expression_break_indent: Indent::make_const(options.continuation_indent, 1),
            block_plus_expression_break_indent: Indent::make_const(options.block_indent + options.continuation_indent, 1),
            double_expression_break_indent: Indent::make_const(options.continuation_indent, 2),
            expression_break_negative_indent: Indent::make_const(-options.continuation_indent, 1),
            options,
            builder,
            in_expression: vec![false],
            in_import: false,
            exception: None,
        }
    }

    /// What upstream's visit threw other than a `FormattingError`: it escapes `format` as is.
    pub fn take_exception(&mut self) -> Option<FormatError> {
        self.exception.take().map(|(exception, _)| exception)
    }
}

macro_rules! forward {
    ($($name:ident($t:ty);)*) => {
        impl KtVisitorVoid for KotlinInputAstVisitor<'_, '_, '_> {
            /// A leaf reaches only `visit_element`, which for a leaf pushes and pops `in_expression`.
            fn ignores_leaves(&self) -> bool {
                true
            }
            $(fn $name(&mut self, e: &$t) { KotlinInputAstVisitor::$name(self, e) })*
        }
    };
}

forward! {
    visit_named_function(KtNamedFunction);
    visit_type_reference(KtTypeReference);
    visit_dynamic_type(KtDynamicType);
    visit_nullable_type(KtNullableType);
    visit_user_type(KtUserType);
    visit_intersection_type(KtIntersectionType);
    visit_type_argument_list(KtTypeArgumentList);
    visit_type_projection(KtTypeProjection);
    visit_property(KtProperty);
    visit_qualified_expression(KtQualifiedExpression);
    visit_call_expression(KtCallExpression);
    visit_value_argument_list(KtValueArgumentList);
    visit_lambda_expression(KtLambdaExpression);
    visit_this_expression(KtThisExpression);
    visit_simple_name_expression(KtSimpleNameExpression);
    visit_parameter_list(KtParameterList);
    visit_argument(KtValueArgument);
    visit_reference_expression(KtReferenceExpression);
    visit_return_expression(KtReturnExpression);
    visit_binary_expression(KtBinaryExpression);
    visit_postfix_expression(KtPostfixExpression);
    visit_prefix_expression(KtPrefixExpression);
    visit_labeled_expression(KtLabeledExpression);
    visit_class_or_object(KtClassOrObject);
    visit_primary_constructor(KtPrimaryConstructor);
    visit_secondary_constructor(KtSecondaryConstructor);
    visit_constructor_delegation_call(KtConstructorDelegationCall);
    visit_class_initializer(KtClassInitializer);
    visit_constant_expression(KtConstantExpression);
    visit_parenthesized_expression(KtParenthesizedExpression);
    visit_package_directive(KtPackageDirective);
    visit_import_list(KtImportList);
    visit_import_directive(KtImportDirective);
    visit_context_receiver_list(KtContextReceiverList);
    visit_modifier_list(KtModifierList);
    visit_annotated_expression(KtAnnotatedExpression);
    visit_annotation(KtAnnotation);
    visit_annotation_use_site_target(KtAnnotationUseSiteTarget);
    visit_annotation_entry(KtAnnotationEntry);
    visit_file_annotation_list(KtFileAnnotationList);
    visit_super_type_list(KtSuperTypeList);
    visit_super_type_call_entry(KtSuperTypeCallEntry);
    visit_delegated_super_type_entry(KtDelegatedSuperTypeEntry);
    visit_when_expression(KtWhenExpression);
    visit_class_body(KtClassBody);
    visit_block_expression(KtBlockExpression);
    visit_when_condition_with_expression(KtWhenConditionWithExpression);
    visit_when_condition_is_pattern(KtWhenConditionIsPattern);
    visit_when_condition_in_range(KtWhenConditionInRange);
    visit_if_expression(KtIfExpression);
    visit_array_access_expression(KtArrayAccessExpression);
    visit_destructuring_declaration(KtDestructuringDeclaration);
    visit_destructuring_declaration_entry(KtDestructuringDeclarationEntry);
    visit_string_template_expression(KtStringTemplateExpression);
    visit_super_expression(KtSuperExpression);
    visit_type_parameter_list(KtTypeParameterList);
    visit_type_parameter(KtTypeParameter);
    visit_type_constraint_list(KtTypeConstraintList);
    visit_type_constraint(KtTypeConstraint);
    visit_for_expression(KtForExpression);
    visit_while_expression(KtWhileExpression);
    visit_do_while_expression(KtDoWhileExpression);
    visit_break_expression(KtBreakExpression);
    visit_continue_expression(KtContinueExpression);
    visit_parameter(KtParameter);
    visit_callable_reference_expression(KtCallableReferenceExpression);
    visit_class_literal_expression(KtClassLiteralExpression);
    visit_function_type(KtFunctionType);
    visit_is_expression(KtIsExpression);
    visit_binary_with_type_rhs_expression(KtBinaryExpressionWithTypeRHS);
    visit_collection_literal_expression(KtCollectionLiteralExpression);
    visit_try_expression(KtTryExpression);
    visit_catch_section(KtCatchClause);
    visit_finally_section(KtFinallySection);
    visit_throw_expression(KtThrowExpression);
    visit_enum_entry(KtEnumEntry);
    visit_type_alias(KtTypeAlias);
    visit_element(PsiElement);
    visit_kt_file(KtFile);
    visit_script(KtScript);
}
