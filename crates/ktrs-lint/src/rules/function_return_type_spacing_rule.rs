//! Port of ktlint-ruleset-standard `FunctionReturnTypeSpacingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{COLON, FUN};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{EditorConfig, MAX_LINE_LENGTH_PROPERTY, PropertyRef};
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;

pub struct FunctionReturnTypeSpacingRule {
    max_line_length: i32,
}

impl FunctionReturnTypeSpacingRule {
    pub fn new() -> FunctionReturnTypeSpacingRule {
        FunctionReturnTypeSpacingRule { max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value }
    }
}

impl Default for FunctionReturnTypeSpacingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for FunctionReturnTypeSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:function-return-type-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.max_line_length = max_line_length(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if let Some(colon_node) = Some(node).filter(|&it| ast.element_type(it) == FUN).and_then(|it| ast.find_child_by_type(it, COLON)) {
            remove_white_space_between_closing_parenthesis_and_colon(ast, colon_node, emit);
            self.fix_white_space_between_colon_and_return_type(ast, colon_node, emit);
        }
    }
}

fn remove_white_space_between_closing_parenthesis_and_colon(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    assert!(ast.element_type(node) == COLON, "IllegalArgumentException: Failed requirement.");
    if let Some(whitespace_before_colon_node) = ast.prev_leaf(node).filter(|&it| ast.is_white_space(it)) {
        emit(ast, ast.start_offset(whitespace_before_colon_node), "Unexpected whitespace", true)
            .if_autocorrect_allowed(|| ast.remove(whitespace_before_colon_node));
    }
}

impl FunctionReturnTypeSpacingRule {
    fn fix_white_space_between_colon_and_return_type(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(node) == COLON, "IllegalArgumentException: Failed requirement.");
        let white_space_after_colon = ast.next_leaf(node).filter(|&it| ast.is_white_space(it));
        if white_space_after_colon.is_none_or(|it| !ast.text_matches(it, " ")) {
            // In case the whitespace contains a newline than replacing it with a single space results in merging the lines to a
            // single line. This rule allows this only when the merged lines entirely fit on a single line. Suppose that code below
            // does not fit on a single line:
            //    fun foo():
            //        String = "some-looooooooooooooooong-string"
            // This rule does *not* attempt to reformat the code as follows:
            //    fun foo(): String =
            //        "some-looooooooooooooooong-string"
            // See FunctionSignatureRule for such reformatting.
            let new_line_length = length_until_newline(ast, Some(node), false) // Length of line before but excluding the colon
                + ast.text_length_utf16(node) // Length of the colon itself
                + 1 // Length of the fixed whitespace
                + length_until_newline(ast, white_space_after_colon, true); // Length of the line after but excluding the whitespace
            if new_line_length as i64 <= self.max_line_length as i64 {
                emit(ast, ast.start_offset(node), "Single space expected between colon and return type", true)
                    .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(node, " "));
            }
        }
    }
}

fn length_until_newline(ast: &Ast, node: Option<NodeId>, forward: bool) -> usize {
    match node {
        None => 0,
        Some(node) => ast
            .leaves(node, forward)
            .take_while(|&it| !ast.is_white_space_with_newline(it))
            .map(|it| ast.text_length_utf16(it))
            .sum(),
    }
}
