//! `PsiElement.accept(visitor)`: each class's `accept(KtVisitor, data)` override, or IntelliJ's
//! `visitElement`/`visitComment`/`visitWhiteSpace`/`visitErrorElement` for non-Kotlin PSI.

use ktrs_syntax::SyntaxKind::*;

use super::KtVisitorVoid;
use crate::cast::PsiType;
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
            ERROR_ELEMENT => v.visit_error_element(&view(e)),
            // KDocImpl (LazyParseablePsiElement) and KDocElementImpl (ASTWrapperPsiElement)
            DOC_COMMENT | KDOC_SECTION | KDOC_TAG => v.visit_element(e),
            CLASS => v.visit_class(&view(e)),
            FUN => v.visit_named_function(&view(e)),
            PROPERTY => v.visit_property(&view(e)),
            DESTRUCTURING_DECLARATION => v.visit_destructuring_declaration(&view(e)),
            DESTRUCTURING_DECLARATION_ENTRY => v.visit_destructuring_declaration_entry(&view(e)),
            OBJECT_DECLARATION => v.visit_object_declaration(&view(e)),
            TYPEALIAS => v.visit_type_alias(&view(e)),
            COMPANION_BLOCK => v.visit_companion_block(&view(e)),
            ENUM_ENTRY => v.visit_enum_entry(&view(e)),
            CLASS_INITIALIZER => v.visit_class_initializer(&view(e)),
            SCRIPT_INITIALIZER => v.visit_script_initializer(&view(e)),
            SECONDARY_CONSTRUCTOR => v.visit_secondary_constructor(&view(e)),
            PRIMARY_CONSTRUCTOR => v.visit_primary_constructor(&view(e)),
            CONTEXT_RECEIVER => v.visit_context_receiver(&view(e)),
            // KtContextReceiverList: `visitContextParameterList(this) ?: visitContextReceiverList(this)`
            CONTEXT_PARAMETER_LIST => {
                v.visit_context_parameter_list(&view(e));
                v.visit_context_receiver_list(&view(e));
            }
            TYPE_PARAMETER_LIST => v.visit_type_parameter_list(&view(e)),
            TYPE_PARAMETER => v.visit_type_parameter(&view(e)),
            SUPER_TYPE_LIST => v.visit_super_type_list(&view(e)),
            DELEGATED_SUPER_TYPE_ENTRY => v.visit_delegated_super_type_entry(&view(e)),
            SUPER_TYPE_CALL_ENTRY => v.visit_super_type_call_entry(&view(e)),
            SUPER_TYPE_ENTRY => v.visit_super_type_entry(&view(e)),
            PROPERTY_DELEGATE => v.visit_property_delegate(&view(e)),
            CONSTRUCTOR_CALLEE => v.visit_constructor_callee_expression(&view(e)),
            VALUE_PARAMETER_LIST => v.visit_parameter_list(&view(e)),
            VALUE_PARAMETER => v.visit_parameter(&view(e)),
            CLASS_BODY => v.visit_class_body(&view(e)),
            IMPORT_LIST => v.visit_import_list(&view(e)),
            FILE_ANNOTATION_LIST => v.visit_file_annotation_list(&view(e)),
            IMPORT_DIRECTIVE => v.visit_import_directive(&view(e)),
            IMPORT_ALIAS => v.visit_import_alias(&view(e)),
            MODIFIER_LIST => v.visit_modifier_list(&view(e)),
            ANNOTATION => v.visit_annotation(&view(e)),
            ANNOTATION_ENTRY => v.visit_annotation_entry(&view(e)),
            ANNOTATION_TARGET => v.visit_annotation_use_site_target(&view(e)),
            TYPE_ARGUMENT_LIST => v.visit_type_argument_list(&view(e)),
            VALUE_ARGUMENT_LIST => v.visit_value_argument_list(&view(e)),
            VALUE_ARGUMENT | LAMBDA_ARGUMENT => v.visit_argument(&view(e)),
            TYPE_REFERENCE => v.visit_type_reference(&view(e)),
            USER_TYPE => v.visit_user_type(&view(e)),
            DYNAMIC_TYPE => v.visit_dynamic_type(&view(e)),
            FUNCTION_TYPE => v.visit_function_type(&view(e)),
            NULLABLE_TYPE => v.visit_nullable_type(&view(e)),
            INTERSECTION_TYPE => v.visit_intersection_type(&view(e)),
            TYPE_PROJECTION => v.visit_type_projection(&view(e)),
            PROPERTY_ACCESSOR => v.visit_property_accessor(&view(e)),
            BACKING_FIELD => v.visit_backing_field(&view(e)),
            INITIALIZER_LIST => v.visit_initializer_list(&view(e)),
            TYPE_CONSTRAINT_LIST => v.visit_type_constraint_list(&view(e)),
            TYPE_CONSTRAINT => v.visit_type_constraint(&view(e)),
            CONSTRUCTOR_DELEGATION_CALL => v.visit_constructor_delegation_call(&view(e)),
            NULL | BOOLEAN_CONSTANT | FLOAT_CONSTANT | CHARACTER_CONSTANT | INTEGER_CONSTANT => {
                v.visit_constant_expression(&view(e))
            }
            STRING_TEMPLATE => v.visit_string_template_expression(&view(e)),
            LONG_STRING_TEMPLATE_ENTRY => v.visit_block_string_template_entry(&view(e)),
            SHORT_STRING_TEMPLATE_ENTRY => v.visit_simple_name_string_template_entry(&view(e)),
            LITERAL_STRING_TEMPLATE_ENTRY => v.visit_literal_string_template_entry(&view(e)),
            ESCAPE_STRING_TEMPLATE_ENTRY => v.visit_escape_string_template_entry(&view(e)),
            STRING_INTERPOLATION_PREFIX => v.visit_string_interpolation_prefix(&view(e)),
            PARENTHESIZED => v.visit_parenthesized_expression(&view(e)),
            RETURN => v.visit_return_expression(&view(e)),
            THROW => v.visit_throw_expression(&view(e)),
            CONTINUE => v.visit_continue_expression(&view(e)),
            BREAK => v.visit_break_expression(&view(e)),
            IF => v.visit_if_expression(&view(e)),
            TRY => v.visit_try_expression(&view(e)),
            CATCH => v.visit_catch_section(&view(e)),
            FINALLY => v.visit_finally_section(&view(e)),
            FOR => v.visit_for_expression(&view(e)),
            WHILE => v.visit_while_expression(&view(e)),
            DO_WHILE => v.visit_do_while_expression(&view(e)),
            BLOCK => v.visit_block_expression(&view(e)),
            LAMBDA_EXPRESSION => v.visit_lambda_expression(&view(e)),
            ANNOTATED_EXPRESSION => v.visit_annotated_expression(&view(e)),
            REFERENCE_EXPRESSION | ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION | OPERATION_REFERENCE | LABEL => {
                v.visit_simple_name_expression(&view(e))
            }
            THIS_EXPRESSION => v.visit_this_expression(&view(e)),
            SUPER_EXPRESSION => v.visit_super_expression(&view(e)),
            BINARY_EXPRESSION => v.visit_binary_expression(&view(e)),
            BINARY_WITH_TYPE => v.visit_binary_with_type_rhs_expression(&view(e)),
            IS_EXPRESSION => v.visit_is_expression(&view(e)),
            PREFIX_EXPRESSION => v.visit_prefix_expression(&view(e)),
            POSTFIX_EXPRESSION => v.visit_postfix_expression(&view(e)),
            LABELED_EXPRESSION => v.visit_labeled_expression(&view(e)),
            CALL_EXPRESSION => v.visit_call_expression(&view(e)),
            ARRAY_ACCESS_EXPRESSION => v.visit_array_access_expression(&view(e)),
            DOT_QUALIFIED_EXPRESSION => v.visit_dot_qualified_expression(&view(e)),
            SAFE_ACCESS_EXPRESSION => v.visit_safe_qualified_expression(&view(e)),
            CALLABLE_REFERENCE_EXPRESSION => v.visit_callable_reference_expression(&view(e)),
            CLASS_LITERAL_EXPRESSION => v.visit_class_literal_expression(&view(e)),
            OBJECT_LITERAL => v.visit_object_literal_expression(&view(e)),
            COLLECTION_LITERAL_EXPRESSION => v.visit_collection_literal_expression(&view(e)),
            WHEN => v.visit_when_expression(&view(e)),
            WHEN_ENTRY => v.visit_when_entry(&view(e)),
            WHEN_CONDITION_IN_RANGE => v.visit_when_condition_in_range(&view(e)),
            WHEN_CONDITION_IS_PATTERN => v.visit_when_condition_is_pattern(&view(e)),
            WHEN_CONDITION_EXPRESSION => v.visit_when_condition_with_expression(&view(e)),
            PACKAGE_DIRECTIVE => v.visit_package_directive(&view(e)),
            SCRIPT => v.visit_script(&view(e)),
            // KtExpressionImpl default (`KtFunctionLiteral` is a KtDeclarationImpl -> KtExpressionImpl)
            FUNCTION_LITERAL | CONSTRUCTOR_DELEGATION_REFERENCE => v.visit_expression(&view(e)),
            // KtElementImpl / KtElementImplStub default: containers, guards, KDoc links and names, ...
            _ => v.visit_kt_element(&view(e)),
        }
    }

    /// `acceptChildren(visitor)`: every child, leaves included.
    pub fn accept_children<V: KtVisitorVoid + ?Sized>(&self, v: &mut V) {
        let ignores_leaves = v.ignores_leaves();
        for child in self.child_id_iter().filter(|&c| !(ignores_leaves && self.tree().is_token(c))) {
            self.at(child).accept(v);
        }
    }
}

/// The typed view of an element whose kind the dispatch `match` already checked.
fn view<T: PsiType>(e: &PsiElement) -> T {
    debug_assert!(T::can_cast(e));
    T::cast_unchecked(e.clone())
}
