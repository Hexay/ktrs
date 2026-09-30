//! `ClassSignatureRule.kt` from `fixWhiteSpacesInValueParameterList` to `fixWhiteSpaceBeforeClosingParenthesis`.

use ktrs_ast::{Ast, NodeId};

use super::ClassSignatureRule;
use super::super_types::get_primary_constructor_parameter_list_or_null;
use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::element_type::{
    CLASS_BODY, CONSTRUCTOR_DELEGATION_CALL, CONSTRUCTOR_DELEGATION_REFERENCE, CONSTRUCTOR_KEYWORD, EXPECT_KEYWORD, RPAR,
    SECONDARY_CONSTRUCTOR, VALUE_PARAMETER,
};
use crate::rule::Emit;

fn text_length(ast: &Ast, node: Option<NodeId>) -> i32 {
    node.map_or(0, |it| ast.text_length_utf16(it) as i32)
}

fn utf16_len(text: &str) -> i32 {
    text.encode_utf16().count() as i32
}

/// `valueParameterList?.children?.first { it.elementType == VALUE_PARAMETER } ?: return 0`.
fn first_value_parameter(ast: &Ast, value_parameter_list: Option<NodeId>) -> Option<NodeId> {
    let list = value_parameter_list?;
    Some(ast.children(list).find(|&it| ast.element_type(it) == VALUE_PARAMETER).expect("NoSuchElementException: No element matching predicate"))
}

impl ClassSignatureRule {
    pub(super) fn fix_white_spaces_in_value_parameter_list(
        &self,
        ast: &mut Ast,
        node: NodeId,
        emit: &mut Emit<'_>,
        multiline: bool,
        dry_run: bool,
    ) -> i32 {
        let mut white_space_correction = 0;

        let primary_constructor_parameter_list = get_primary_constructor_parameter_list_or_null(ast, node);
        let has_no_value_parameters =
            primary_constructor_parameter_list.is_none_or(|list| !ast.children(list).any(|it| ast.element_type(it) == VALUE_PARAMETER));

        white_space_correction += if has_no_value_parameters {
            fix_white_spaces_in_empty_value_parameter_list(ast, node, emit, dry_run)
        } else {
            self.fix_white_spaces_before_first_parameter_in_value_parameter_list(ast, node, emit, multiline, dry_run)
                + self.fix_white_spaces_between_parameters_in_value_parameter_list(ast, node, emit, multiline, dry_run)
                + fix_white_space_before_closing_parenthesis(ast, node, emit, multiline, dry_run)
        };

        white_space_correction
    }
}

fn fix_white_spaces_in_empty_value_parameter_list(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>, dry_run: bool) -> i32 {
    let mut white_space_correction = 0;

    let parameter_list = Some(node)
        // https://kotlinlang.org/docs/multiplatform-expect-actual.html#rules-for-expected-and-actual-declarations
        .filter(|&it| !ast.has_modifier(it, EXPECT_KEYWORD))
        .and_then(|it| get_primary_constructor_parameter_list_or_null(ast, it))
        .filter(|&it| !contains_comment(ast, it))
        // Allow:
        //     class Foo constructor() { ... }
        .filter(|&it| ast.prev_code_sibling(it).map(|p| ast.element_type(p)) != Some(CONSTRUCTOR_KEYWORD))
        // Allow
        //     class Foo() {
        //         constructor(foo: String): this() {
        //             println(foo)
        //         }
        //     }
        .filter(|_| {
            ast.find_child_by_type(node, CLASS_BODY)
                .and_then(|it| ast.find_child_by_type(it, SECONDARY_CONSTRUCTOR))
                .and_then(|it| ast.find_child_by_type(it, CONSTRUCTOR_DELEGATION_CALL))
                .and_then(|it| ast.first_child_node(it))
                .map(|it| ast.element_type(it))
                != Some(CONSTRUCTOR_DELEGATION_REFERENCE)
        });
    if let Some(parameter_list) = parameter_list {
        if !dry_run {
            emit(ast, ast.start_offset(parameter_list), "No parenthesis expected", true).if_autocorrect_allowed(|| ast.remove(parameter_list));
        } else {
            white_space_correction -= ast.text_length_utf16(parameter_list) as i32;
        }
    }

    white_space_correction
}

fn contains_comment(ast: &Ast, node: NodeId) -> bool {
    ast.children(node).any(|it| ast.is_part_of_comment(it))
}

impl ClassSignatureRule {
    fn fix_white_spaces_before_first_parameter_in_value_parameter_list(
        &self,
        ast: &mut Ast,
        node: NodeId,
        emit: &mut Emit<'_>,
        multiline: bool,
        dry_run: bool,
    ) -> i32 {
        let mut white_space_correction = 0;

        let value_parameter_list = get_primary_constructor_parameter_list_or_null(ast, node);
        let Some(first_parameter_in_list) = first_value_parameter(ast, value_parameter_list) else { return 0 };

        let first_parameter = ast.first_child_node(first_parameter_in_list);
        let white_space_before_identifier = first_parameter.and_then(|it| ast.prev_leaf(it)).filter(|&it| ast.is_white_space(it));
        if multiline {
            if ast.is_white_space_without_newline_or_null(white_space_before_identifier) {
                // Let indent rule determine the exact indent
                let expected_parameter_indent = self.indent_config.child_indent_of(ast, node);
                if !dry_run {
                    emit(ast, ast.start_offset(first_parameter_in_list), "Newline expected after opening parenthesis", true)
                        .if_autocorrect_allowed(|| {
                            let lpar = value_parameter_list.and_then(|it| ast.first_child_node(it)).expect("NullPointerException");
                            ast.upsert_whitespace_after_me(lpar, &expected_parameter_indent)
                        });
                } else {
                    white_space_correction += utf16_len(&expected_parameter_indent) - text_length(ast, white_space_before_identifier);
                }
            }
        } else if let Some(white_space_before_identifier) = white_space_before_identifier {
            if !dry_run {
                let first_parameter = first_parameter.expect("NullPointerException: firstParameter!!");
                emit(ast, ast.start_offset(first_parameter), "No whitespace expected between opening parenthesis and first parameter name", true)
                    .if_autocorrect_allowed(|| ast.remove(white_space_before_identifier));
            } else {
                white_space_correction -= ast.text_length_utf16(white_space_before_identifier) as i32;
            }
        }

        white_space_correction
    }

    fn fix_white_spaces_between_parameters_in_value_parameter_list(
        &self,
        ast: &mut Ast,
        node: NodeId,
        emit: &mut Emit<'_>,
        multiline: bool,
        dry_run: bool,
    ) -> i32 {
        let mut white_space_correction = 0;

        let value_parameter_list = get_primary_constructor_parameter_list_or_null(ast, node);
        let Some(first_parameter_in_list) = first_value_parameter(ast, value_parameter_list) else { return 0 };
        let value_parameter_list = value_parameter_list.unwrap();

        let value_parameters: Vec<NodeId> = ast
            .children(value_parameter_list)
            .filter(|&it| ast.element_type(it) == VALUE_PARAMETER)
            .filter(|&it| it != first_parameter_in_list)
            .collect();
        for value_parameter in value_parameters {
            let first_child_node_in_value_parameter = ast.first_child_node(value_parameter);
            let white_space_before_identifier =
                first_child_node_in_value_parameter.and_then(|it| ast.prev_leaf(it)).filter(|&it| ast.is_white_space(it));
            if multiline {
                if ast.is_white_space_without_newline_or_null(white_space_before_identifier) {
                    // Let IndentationRule determine the exact indent
                    let expected_parameter_indent = self.indent_config.child_indent_of(ast, node);
                    if !dry_run {
                        emit(ast, ast.start_offset(value_parameter), "Parameter should start on a newline", true).if_autocorrect_allowed(|| {
                            let first = first_child_node_in_value_parameter.expect("NullPointerException: firstChildNode");
                            ast.upsert_whitespace_before_me(first, &expected_parameter_indent)
                        });
                    } else {
                        white_space_correction += utf16_len(&expected_parameter_indent) - text_length(ast, white_space_before_identifier);
                    }
                }
            } else if white_space_before_identifier.is_none_or(|it| !ast.text_matches(it, " ")) {
                if !dry_run {
                    let first = first_child_node_in_value_parameter.expect("NullPointerException: firstChildNodeInValueParameter!!");
                    emit(ast, ast.start_offset(first), "Single whitespace expected before parameter", true)
                        .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(first, " "));
                } else {
                    white_space_correction += 1 - text_length(ast, white_space_before_identifier);
                }
            }
        }

        white_space_correction
    }
}

fn fix_white_space_before_closing_parenthesis(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>, multiline: bool, dry_run: bool) -> i32 {
    let mut white_space_correction = 0;

    let closing_parenthesis_primary_constructor =
        get_primary_constructor_parameter_list_or_null(ast, node).and_then(|it| ast.find_child_by_type(it, RPAR));
    let white_space_before_closing_parenthesis =
        closing_parenthesis_primary_constructor.and_then(|it| ast.prev_sibling(it)).filter(|&it| ast.is_white_space(it));
    if multiline {
        if ast.is_white_space_without_newline_or_null(white_space_before_closing_parenthesis) {
            // Let IndentationRule determine the exact indent
            let expected_parameter_indent = ast.indent(node);
            if !dry_run {
                let closing_parenthesis = closing_parenthesis_primary_constructor.expect("NullPointerException: closingParenthesisPrimaryConstructor!!");
                emit(ast, ast.start_offset(closing_parenthesis), "Newline expected before closing parenthesis", true)
                    .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(closing_parenthesis, &expected_parameter_indent));
            } else {
                white_space_correction += utf16_len(&expected_parameter_indent) - text_length(ast, white_space_before_closing_parenthesis);
            }
        }
    } else if let Some(white_space_before_closing_parenthesis) = white_space_before_closing_parenthesis
        && ast.next_leaf(white_space_before_closing_parenthesis).map(|it| ast.element_type(it)) == Some(RPAR)
    {
        if !dry_run {
            emit(ast, ast.start_offset(white_space_before_closing_parenthesis), "No whitespace expected between last parameter and closing parenthesis", true)
                .if_autocorrect_allowed(|| ast.remove(white_space_before_closing_parenthesis));
        } else {
            white_space_correction -= ast.text_length_utf16(white_space_before_closing_parenthesis) as i32;
        }
    }
    white_space_correction
}
