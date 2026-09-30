//! `FunctionSignatureRule.kt` from `fixWhiteSpacesInValueParameterList` to `fixWhiteSpaceBeforeClosingParenthesis`.

use ktrs_ast::{Ast, NodeId};

use super::{FunctionSignatureRule, utf16_len};
use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeQueries};
use crate::element_type::{LPAR, RPAR, VALUE_PARAMETER, VALUE_PARAMETER_LIST};
use crate::rule::Emit;

fn value_parameter_list(ast: &Ast, node: NodeId) -> NodeId {
    ast.find_child_by_type(node, VALUE_PARAMETER_LIST).expect("IllegalArgumentException: Required value was null.")
}

fn value_parameters(ast: &Ast, value_parameter_list: NodeId) -> Vec<NodeId> {
    ast.children(value_parameter_list).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).collect()
}

fn text_length(ast: &Ast, node: Option<NodeId>) -> i32 {
    node.map_or(0, |it| ast.text_length_utf16(it) as i32)
}

impl FunctionSignatureRule {
    pub(super) fn fix_white_spaces_in_value_parameter_list(
        &self,
        ast: &mut Ast,
        node: NodeId,
        emit: &mut Emit<'_>,
        multiline: bool,
        dry_run: bool,
    ) -> i32 {
        let mut white_space_correction = 0;

        let value_parameter_list = value_parameter_list(ast, node);
        let first_parameter_in_list = value_parameters(ast, value_parameter_list).first().copied();

        white_space_correction += if first_parameter_in_list.is_none() {
            // handle empty parameter list
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

    let value_parameter_list = value_parameter_list(ast, node);

    let elements_in_value_parameter_list: Vec<NodeId> =
        ast.children(value_parameter_list).filter(|&it| !matches!(ast.element_type(it), LPAR | RPAR)).collect();
    // Functions with comments in the value parameter list are excluded from processing before. So an "empty" value
    // parameter list should only contain a single whitespace element
    assert!(elements_in_value_parameter_list.len() <= 1, "IllegalArgumentException: Failed requirement.");
    if let Some(&white_space) = elements_in_value_parameter_list.first() {
        if !dry_run {
            emit(ast, ast.start_offset(white_space), "No whitespace expected in empty parameter list", true)
                .if_autocorrect_allowed(|| ast.remove(white_space));
        } else {
            white_space_correction -= ast.text_length_utf16(white_space) as i32;
        }
    }

    white_space_correction
}

impl FunctionSignatureRule {
    fn fix_white_spaces_before_first_parameter_in_value_parameter_list(
        &self,
        ast: &mut Ast,
        node: NodeId,
        emit: &mut Emit<'_>,
        multiline: bool,
        dry_run: bool,
    ) -> i32 {
        let mut white_space_correction = 0;

        let value_parameter_list = value_parameter_list(ast, node);
        let first_parameter_in_list =
            *value_parameters(ast, value_parameter_list).first().expect("NoSuchElementException: No element matching predicate");

        let first_parameter = ast.first_child_node(first_parameter_in_list);
        let white_space_before_identifier = first_parameter.and_then(|it| ast.prev_leaf(it)).filter(|&it| ast.is_white_space(it));
        if multiline {
            let expected_parameter_indent = self.indent_config.child_indent_of(ast, node);
            if white_space_before_identifier.is_none_or(|it| !ast.text_matches(it, &expected_parameter_indent)) {
                if !dry_run {
                    emit(ast, ast.start_offset(first_parameter_in_list), "Newline expected after opening parenthesis", true)
                        .if_autocorrect_allowed(|| {
                            let lpar = ast.first_child_node(value_parameter_list).expect("NullPointerException: firstChildNode");
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

        let value_parameter_list = value_parameter_list(ast, node);
        let value_parameters = value_parameters(ast, value_parameter_list);
        let first_parameter_in_list = *value_parameters.first().expect("NoSuchElementException: No element matching predicate");

        for value_parameter in value_parameters.into_iter().filter(|&it| it != first_parameter_in_list) {
            let first_child_node_in_value_parameter = ast.first_child_node(value_parameter);
            let white_space_before_identifier =
                first_child_node_in_value_parameter.and_then(|it| ast.prev_leaf(it)).filter(|&it| ast.is_white_space(it));
            if multiline {
                let expected_parameter_indent = self.indent_config.child_indent_of(ast, node);
                if white_space_before_identifier.is_none_or(|it| !ast.text_matches(it, &expected_parameter_indent)) {
                    if !dry_run && ast.has_no_max_line_length_suppression(value_parameter_list) {
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

    let newline_and_indent = ast.indent(node);
    let value_parameter_list = value_parameter_list(ast, node);

    let closing_parenthesis = ast.find_child_by_type(value_parameter_list, RPAR);
    let white_space_before_closing_parenthesis =
        closing_parenthesis.and_then(|it| ast.prev_sibling(it)).filter(|&it| ast.is_white_space(it));
    if multiline {
        if white_space_before_closing_parenthesis.is_none_or(|it| !ast.text_matches(it, &newline_and_indent)) {
            if !dry_run {
                let closing_parenthesis = closing_parenthesis.expect("NullPointerException: closingParenthesis!!");
                emit(ast, ast.start_offset(closing_parenthesis), "Newline expected before closing parenthesis", true)
                    .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(closing_parenthesis, &newline_and_indent));
            } else {
                white_space_correction += utf16_len(&newline_and_indent) - text_length(ast, white_space_before_closing_parenthesis);
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
