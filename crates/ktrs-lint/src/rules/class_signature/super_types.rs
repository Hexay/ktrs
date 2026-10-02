//! `ClassSignatureRule.kt` from `fixWhitespacesInSuperTypeList` to `getPrimaryConstructorParameterListOrNull`.

use ktrs_ast::{Ast, NodeId};

use super::{ClassSignatureRule, get_class_signature_length, has_multiline_super_type_list, super_types};
use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeQueries};
use crate::editorconfig::MAX_LINE_LENGTH_PROPERTY_OFF;
use crate::element_type::{
    CLASS_BODY, COMMA, EOL_COMMENT, PRIMARY_CONSTRUCTOR, RPAR, SUPER_TYPE_CALL_ENTRY, SUPER_TYPE_LIST, VALUE_PARAMETER, VALUE_PARAMETER_LIST,
    WHITE_SPACE,
};
use crate::rule::{AutocorrectDecision, Emit};

impl ClassSignatureRule {
    pub(super) fn fix_whitespaces_in_super_type_list(
        &self,
        ast: &mut Ast,
        node: NodeId,
        emit: &mut Emit<'_>,
        wrapped_primary_constructor: bool,
    ) -> i32 {
        let white_space_correction = 0;

        let Some(super_types) = super_types(ast, node) else { return 0 };
        let first = |ast: &Ast| *super_types.list(ast).first().expect("NoSuchElementException: Sequence is empty.");

        if ast.element_type(first(ast)) != SUPER_TYPE_CALL_ENTRY
            && let Some(super_type_call_entry) = super_types.list(ast).into_iter().find(|&it| ast.element_type(it) == SUPER_TYPE_CALL_ENTRY)
        {
            let decision = emit(ast, ast.start_offset(super_type_call_entry), "Super type call must be first super type", true);
            if decision == AutocorrectDecision::AllowAutocorrect {
                let Some(super_type_list) = ast.find_child_by_type(node, SUPER_TYPE_LIST) else { return 0 };
                let original_first_super_type = first(ast);
                let comma_before_super_type_call = ast
                    .prev_sibling_matching(super_type_call_entry, |it| ast.element_type(it) == COMMA)
                    .expect("IllegalArgumentException: Required value was null.");

                // Remove the whitespace before the super type call and do not insert a new whitespace as it will be fixed later
                if let Some(it) = ast.prev_sibling(super_type_call_entry).filter(|&it| ast.is_white_space(it)) {
                    ast.remove(it);
                }

                let anchor = first(ast);
                ast.add_child(super_type_list, super_type_call_entry, Some(anchor));
                ast.add_child(super_type_list, comma_before_super_type_call, Some(original_first_super_type));
            }
        }

        if super_types.list(ast).len() == 1 {
            let first_super_type = first(ast);
            if wrapped_primary_constructor {
                // Format
                //     class ClassWithPrimaryConstructorWhichWillBeWrapped(...) :
                //         SomeSuperTypeEntry
                // to
                //     class ClassWithPrimaryConstructorWhichWillBeWrapped(
                //         ...
                //     ) : SomeSuperTypeEntry
                if let Some(white_space_before_super_type) = ast
                    .prev_leaf(first_super_type)
                    .filter(|&it| ast.is_white_space_with_newline(it))
                    .filter(|&it| ast.prev_sibling(it).map(|p| ast.element_type(p)) != Some(EOL_COMMENT))
                {
                    let expected_whitespace = " ";
                    if !ast.text_matches(white_space_before_super_type, expected_whitespace) {
                        emit(ast, ast.start_offset(first_super_type), "Expected single space before the super type", true)
                            .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(first_super_type, expected_whitespace));
                    }
                }
            } else if let Some(super_type_first_child_node) = ast.first_child_node(first_super_type) {
                let white_space_before_identifier = ast.prev_leaf(super_type_first_child_node).filter(|&it| ast.is_white_space(it));
                if has_multiline_super_type_list(ast, node)
                    || self.class_signatures_including_first_super_type_exceeds_max_line_length(ast, node, emit)
                {
                    if ast.is_white_space_without_newline_or_null(white_space_before_identifier) {
                        emit(ast, ast.start_offset(super_type_first_child_node), "Super type should start on a newline", true)
                            .if_autocorrect_allowed(|| {
                                // Let IndentationRule determine the exact indent
                                let indent = self.indent_config.child_indent_of(ast, node);
                                ast.upsert_whitespace_before_me(super_type_first_child_node, &indent)
                            });
                    }
                } else if self.ktlint_version.is_1_8() // 1.8 joins the supertype even after an EOL comment (#3312)
                    || ast
                        .prev_leaf(super_type_first_child_node)
                        .and_then(|it| ast.prev_sibling(it))
                        .map(|it| ast.element_type(it))
                        != Some(EOL_COMMENT)
                {
                    let expected_whitespace = " ";
                    if white_space_before_identifier.is_none_or(|it| !ast.text_matches(it, expected_whitespace)) {
                        emit(ast, ast.start_offset(super_type_first_child_node), "Expected single space before the super type", true)
                            .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(super_type_first_child_node, expected_whitespace));
                    }
                }
            }
        } else {
            for (index, super_type) in super_types.list(ast).into_iter().enumerate() {
                let first_child_node_in_super_type = ast.first_child_node(super_type);
                let white_space_before_identifier =
                    first_child_node_in_super_type.and_then(|it| ast.prev_leaf(it)).filter(|&it| ast.is_white_space(it));
                let first_child = || first_child_node_in_super_type.expect("NullPointerException: firstChildNode");
                if index == 0 && has_multiline_primary_constructor(ast, node) {
                    let expected_whitespace = " ";
                    if white_space_before_identifier.and_then(|it| ast.prev_leaf(it)).map(|it| ast.element_type(it)) != Some(EOL_COMMENT)
                        && white_space_before_identifier.is_none_or(|it| !ast.text_matches(it, expected_whitespace))
                    {
                        let first = first_child();
                        emit(ast, ast.start_offset(first), "Expected single space before the first super type", true)
                            .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(first, expected_whitespace));
                    }
                } else if ast.is_white_space_without_newline_or_null(white_space_before_identifier) {
                    let first = first_child();
                    emit(ast, ast.start_offset(first), "Super type should start on a newline", true).if_autocorrect_allowed(|| {
                        // Let IndentationRule determine the exact indent
                        let indent = self.indent_config.child_indent_of(ast, node);
                        ast.upsert_whitespace_before_me(first, &indent)
                    });
                }
            }
        }

        // Disallow:
        //    class Foo : Bar<String> ("foobar")
        let super_type_call_entries: Vec<NodeId> =
            super_types.list(ast).into_iter().filter(|&it| ast.element_type(it) == SUPER_TYPE_CALL_ENTRY).collect();
        for super_type_call_entry in super_type_call_entries {
            if let Some(whitespace) = ast.find_child_by_type(super_type_call_entry, WHITE_SPACE) {
                emit(ast, ast.start_offset(whitespace), "No whitespace expected", true).if_autocorrect_allowed(|| ast.remove(whitespace));
            }
        }

        white_space_correction
    }

    fn class_signatures_including_first_super_type_exceeds_max_line_length(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) -> bool {
        let actual_class_signature_length = get_class_signature_length(ast, node, false);
        // Calculate the length of the class signature in case it, including the super types, would be rewritten as single
        // line (and without a maximum line length). The white space correction will be calculated via a dry run of the
        // actual fix.
        let length = actual_class_signature_length
            // Calculate the white space correction in case the signature would be rewritten to a single line
            + self.fix_white_spaces_in_value_parameter_list(ast, node, emit, false, true);
        ast.has_no_max_line_length_suppression_in(node, self.ktlint_version) && length > self.max_line_length
    }
}

fn has_multiline_primary_constructor(ast: &Ast, node: NodeId) -> bool {
    ast.is_white_space_with_newline(
        ast.find_child_by_type(node, PRIMARY_CONSTRUCTOR)
            .and_then(|it| ast.find_child_by_type(it, VALUE_PARAMETER_LIST))
            .and_then(|it| ast.find_child_by_type(it, RPAR))
            .and_then(|it| ast.prev_leaf_matching(it, |l| !ast.is_part_of_comment(l))),
    )
}

pub(super) fn fix_class_body(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    if let Some(class_body) = ast.find_child_by_type(node, CLASS_BODY)
        && !ast.prev_leaf(class_body).is_some_and(|it| ast.text_matches(it, " "))
    {
        emit(ast, ast.start_offset(class_body), "Expected a single space before class body", true).if_autocorrect_allowed(|| {
            if let Some(prev_leaf) = ast.prev_leaf(class_body) {
                ast.upsert_whitespace_after_me(prev_leaf, " ");
            }
        });
    }
}

impl ClassSignatureRule {
    pub(super) fn is_max_line_length_set(&self) -> bool {
        self.max_line_length != MAX_LINE_LENGTH_PROPERTY_OFF
    }

    pub(super) fn has_too_many_parameters(&self, ast: &Ast, node: NodeId) -> bool {
        count_parameters(ast, node) >= self.class_signature_wrapping_minimum_parameters
    }
}

fn count_parameters(ast: &Ast, node: NodeId) -> i32 {
    get_primary_constructor_parameter_list_or_null(ast, node)
        .map_or(0, |list| ast.children(list).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).count() as i32)
}

pub(super) fn get_primary_constructor_parameter_list_or_null(ast: &Ast, node: NodeId) -> Option<NodeId> {
    ast.find_child_by_type(node, PRIMARY_CONSTRUCTOR).and_then(|it| ast.find_child_by_type(it, VALUE_PARAMETER_LIST))
}
