//! `PsiElement.accept(visitor)`: each class's `accept(KtVisitor, data)` override, or IntelliJ's
//! `visitElement`/`visitComment`/`visitWhiteSpace`/`visitErrorElement` for non-Kotlin PSI.

use ktrs_syntax::SyntaxKind::*;

use super::KtVisitorVoid;
use crate::element::PsiElement;

impl PsiElement {
    pub fn accept<V: KtVisitorVoid + ?Sized>(&self, v: &mut V) {
        if self.is_leaf() {
            match self.kind() {
                WHITE_SPACE => v.visit_white_space(&self.upcast()),
                EOL_COMMENT | BLOCK_COMMENT | SHEBANG_COMMENT => v.visit_comment(&self.upcast()),
                _ => v.visit_element(self),
            }
            return;
        }
        if self.is_file() {
            return v.visit_kt_file(&self.upcast());
        }
        let e = self;
        match self.kind() {
            ERROR_ELEMENT => v.visit_error_element(&e.upcast()),
            // KDocImpl (LazyParseablePsiElement) and KDocElementImpl (ASTWrapperPsiElement)
            DOC_COMMENT | KDOC_SECTION | KDOC_TAG => v.visit_element(e),
            CLASS => v.visit_class(&e.upcast()),
            FUN => v.visit_named_function(&e.upcast()),
            PROPERTY => v.visit_property(&e.upcast()),
            DESTRUCTURING_DECLARATION => v.visit_destructuring_declaration(&e.upcast()),
            DESTRUCTURING_DECLARATION_ENTRY => v.visit_destructuring_declaration_entry(&e.upcast()),
            OBJECT_DECLARATION => v.visit_object_declaration(&e.upcast()),
            TYPEALIAS => v.visit_type_alias(&e.upcast()),
            COMPANION_BLOCK => v.visit_companion_block(&e.upcast()),
            ENUM_ENTRY => v.visit_enum_entry(&e.upcast()),
            CLASS_INITIALIZER => v.visit_class_initializer(&e.upcast()),
            SCRIPT_INITIALIZER => v.visit_script_initializer(&e.upcast()),
            SECONDARY_CONSTRUCTOR => v.visit_secondary_constructor(&e.upcast()),
            PRIMARY_CONSTRUCTOR => v.visit_primary_constructor(&e.upcast()),
            CONTEXT_RECEIVER => v.visit_context_receiver(&e.upcast()),
            // KtContextReceiverList: `visitContextParameterList(this) ?: visitContextReceiverList(this)`
            CONTEXT_PARAMETER_LIST => {
                v.visit_context_parameter_list(&e.upcast());
                v.visit_context_receiver_list(&e.upcast());
            }
            TYPE_PARAMETER_LIST => v.visit_type_parameter_list(&e.upcast()),
            TYPE_PARAMETER => v.visit_type_parameter(&e.upcast()),
            SUPER_TYPE_LIST => v.visit_super_type_list(&e.upcast()),
            DELEGATED_SUPER_TYPE_ENTRY => v.visit_delegated_super_type_entry(&e.upcast()),
            SUPER_TYPE_CALL_ENTRY => v.visit_super_type_call_entry(&e.upcast()),
            SUPER_TYPE_ENTRY => v.visit_super_type_entry(&e.upcast()),
            PROPERTY_DELEGATE => v.visit_property_delegate(&e.upcast()),
            CONSTRUCTOR_CALLEE => v.visit_constructor_callee_expression(&e.upcast()),
            VALUE_PARAMETER_LIST => v.visit_parameter_list(&e.upcast()),
            VALUE_PARAMETER => v.visit_parameter(&e.upcast()),
            CLASS_BODY => v.visit_class_body(&e.upcast()),
            IMPORT_LIST => v.visit_import_list(&e.upcast()),
            FILE_ANNOTATION_LIST => v.visit_file_annotation_list(&e.upcast()),
            IMPORT_DIRECTIVE => v.visit_import_directive(&e.upcast()),
            IMPORT_ALIAS => v.visit_import_alias(&e.upcast()),
            MODIFIER_LIST => v.visit_modifier_list(&e.upcast()),
            ANNOTATION => v.visit_annotation(&e.upcast()),
            ANNOTATION_ENTRY => v.visit_annotation_entry(&e.upcast()),
            ANNOTATION_TARGET => v.visit_annotation_use_site_target(&e.upcast()),
            TYPE_ARGUMENT_LIST => v.visit_type_argument_list(&e.upcast()),
            VALUE_ARGUMENT_LIST => v.visit_value_argument_list(&e.upcast()),
            VALUE_ARGUMENT | LAMBDA_ARGUMENT => v.visit_argument(&e.upcast()),
            TYPE_REFERENCE => v.visit_type_reference(&e.upcast()),
            USER_TYPE => v.visit_user_type(&e.upcast()),
            DYNAMIC_TYPE => v.visit_dynamic_type(&e.upcast()),
            FUNCTION_TYPE => v.visit_function_type(&e.upcast()),
            NULLABLE_TYPE => v.visit_nullable_type(&e.upcast()),
            INTERSECTION_TYPE => v.visit_intersection_type(&e.upcast()),
            TYPE_PROJECTION => v.visit_type_projection(&e.upcast()),
            PROPERTY_ACCESSOR => v.visit_property_accessor(&e.upcast()),
            BACKING_FIELD => v.visit_backing_field(&e.upcast()),
            INITIALIZER_LIST => v.visit_initializer_list(&e.upcast()),
            TYPE_CONSTRAINT_LIST => v.visit_type_constraint_list(&e.upcast()),
            TYPE_CONSTRAINT => v.visit_type_constraint(&e.upcast()),
            CONSTRUCTOR_DELEGATION_CALL => v.visit_constructor_delegation_call(&e.upcast()),
            NULL | BOOLEAN_CONSTANT | FLOAT_CONSTANT | CHARACTER_CONSTANT | INTEGER_CONSTANT => {
                v.visit_constant_expression(&e.upcast())
            }
            STRING_TEMPLATE => v.visit_string_template_expression(&e.upcast()),
            LONG_STRING_TEMPLATE_ENTRY => v.visit_block_string_template_entry(&e.upcast()),
            SHORT_STRING_TEMPLATE_ENTRY => v.visit_simple_name_string_template_entry(&e.upcast()),
            LITERAL_STRING_TEMPLATE_ENTRY => v.visit_literal_string_template_entry(&e.upcast()),
            ESCAPE_STRING_TEMPLATE_ENTRY => v.visit_escape_string_template_entry(&e.upcast()),
            STRING_INTERPOLATION_PREFIX => v.visit_string_interpolation_prefix(&e.upcast()),
            PARENTHESIZED => v.visit_parenthesized_expression(&e.upcast()),
            RETURN => v.visit_return_expression(&e.upcast()),
            THROW => v.visit_throw_expression(&e.upcast()),
            CONTINUE => v.visit_continue_expression(&e.upcast()),
            BREAK => v.visit_break_expression(&e.upcast()),
            IF => v.visit_if_expression(&e.upcast()),
            TRY => v.visit_try_expression(&e.upcast()),
            CATCH => v.visit_catch_section(&e.upcast()),
            FINALLY => v.visit_finally_section(&e.upcast()),
            FOR => v.visit_for_expression(&e.upcast()),
            WHILE => v.visit_while_expression(&e.upcast()),
            DO_WHILE => v.visit_do_while_expression(&e.upcast()),
            BLOCK => v.visit_block_expression(&e.upcast()),
            LAMBDA_EXPRESSION => v.visit_lambda_expression(&e.upcast()),
            ANNOTATED_EXPRESSION => v.visit_annotated_expression(&e.upcast()),
            REFERENCE_EXPRESSION | ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION | OPERATION_REFERENCE | LABEL => {
                v.visit_simple_name_expression(&e.upcast())
            }
            THIS_EXPRESSION => v.visit_this_expression(&e.upcast()),
            SUPER_EXPRESSION => v.visit_super_expression(&e.upcast()),
            BINARY_EXPRESSION => v.visit_binary_expression(&e.upcast()),
            BINARY_WITH_TYPE => v.visit_binary_with_type_rhs_expression(&e.upcast()),
            IS_EXPRESSION => v.visit_is_expression(&e.upcast()),
            PREFIX_EXPRESSION => v.visit_prefix_expression(&e.upcast()),
            POSTFIX_EXPRESSION => v.visit_postfix_expression(&e.upcast()),
            LABELED_EXPRESSION => v.visit_labeled_expression(&e.upcast()),
            CALL_EXPRESSION => v.visit_call_expression(&e.upcast()),
            ARRAY_ACCESS_EXPRESSION => v.visit_array_access_expression(&e.upcast()),
            DOT_QUALIFIED_EXPRESSION => v.visit_dot_qualified_expression(&e.upcast()),
            SAFE_ACCESS_EXPRESSION => v.visit_safe_qualified_expression(&e.upcast()),
            CALLABLE_REFERENCE_EXPRESSION => v.visit_callable_reference_expression(&e.upcast()),
            CLASS_LITERAL_EXPRESSION => v.visit_class_literal_expression(&e.upcast()),
            OBJECT_LITERAL => v.visit_object_literal_expression(&e.upcast()),
            COLLECTION_LITERAL_EXPRESSION => v.visit_collection_literal_expression(&e.upcast()),
            WHEN => v.visit_when_expression(&e.upcast()),
            WHEN_ENTRY => v.visit_when_entry(&e.upcast()),
            WHEN_CONDITION_IN_RANGE => v.visit_when_condition_in_range(&e.upcast()),
            WHEN_CONDITION_IS_PATTERN => v.visit_when_condition_is_pattern(&e.upcast()),
            WHEN_CONDITION_EXPRESSION => v.visit_when_condition_with_expression(&e.upcast()),
            PACKAGE_DIRECTIVE => v.visit_package_directive(&e.upcast()),
            SCRIPT => v.visit_script(&e.upcast()),
            // KtExpressionImpl default (`KtFunctionLiteral` is a KtDeclarationImpl -> KtExpressionImpl)
            FUNCTION_LITERAL | CONSTRUCTOR_DELEGATION_REFERENCE => v.visit_expression(&e.upcast()),
            // KtElementImpl / KtElementImplStub default: containers, guards, KDoc links and names, ...
            _ => v.visit_kt_element(&e.upcast()),
        }
    }

    /// `acceptChildren(visitor)`: every child, leaves included.
    pub fn accept_children<V: KtVisitorVoid + ?Sized>(&self, v: &mut V) {
        for child in self.all_children() {
            child.accept(v);
        }
    }
}
