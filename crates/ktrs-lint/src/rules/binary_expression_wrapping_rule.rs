//! Port of ktlint-ruleset-standard `BinaryExpressionWrappingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    BINARY_EXPRESSION, CALL_EXPRESSION, CONDITION, ELVIS, EQ, FUN, FUNCTION_LITERAL, LAMBDA_ARGUMENT, LAMBDA_EXPRESSION, LBRACE,
    LONG_STRING_TEMPLATE_ENTRY, OPERATION_REFERENCE, PROPERTY, RBRACE, VALUE_ARGUMENT,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, MAX_LINE_LENGTH_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;

/// Wraps a binary expression that does not fit on the line, in preference to wrapping the arguments of calls inside it.
pub struct BinaryExpressionWrappingRule {
    indent_config: IndentConfig,
    max_line_length: i32,
}

impl BinaryExpressionWrappingRule {
    pub fn new() -> BinaryExpressionWrappingRule {
        BinaryExpressionWrappingRule {
            indent_config: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
        }
    }
}

impl Default for BinaryExpressionWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for BinaryExpressionWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:binary-expression-wrapping")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY),
        ]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        self.max_line_length = max_line_length(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == BINARY_EXPRESSION {
            self.visit_binary_expression(ast, node, emit);
        }
    }
}

impl BinaryExpressionWrappingRule {
    fn visit_binary_expression(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(node) == BINARY_EXPRESSION, "IllegalArgumentException: Failed requirement.");
        let indent_config = &self.indent_config;

        // First check whether the entire expression has to be pushed to the next line after and assignment in a property or function
        let parent_type = ast.element_type(ast.parent(node).expect("NullPointerException: parent!!"));
        if matches!(parent_type, PROPERTY | FUN)
            && ast
                .prev_sibling_matching(node, |it| ast.element_type(it) == EQ)
                .is_some_and(|eq| ast.no_new_line_in_closed_range(eq, ast.first_child_leaf_or_self(node)))
            && self.is_on_line_exceeding_max_line_length(ast, node)
        {
            emit(ast, ast.start_offset(node), "Line is exceeding max line length. Break line between assignment and expression", true)
                .if_autocorrect_allowed(|| {
                    let indent = indent_config.child_indent_of(ast, node);
                    ast.upsert_whitespace_before_me(node, &indent);
                });
        }

        // Prefer wrapping the entire binary expression to a newline over wrapping it at the operation reference:
        // `fooBar(\n "foooooo" + "bar",\n)` instead of `fooBar("foooooo" +\n "bar")`
        if ast.parent(node).map(|p| ast.element_type(p)) == Some(VALUE_ARGUMENT)
            // Allow `fooBar(\n "tooLongToFitOnSingleLine" +\n "bar",\n)`
            && !ast.is_white_space_with_newline(ast.prev_leaf(node))
            && self.causes_max_line_length_to_be_exceeded(ast, node)
        {
            emit(ast, ast.start_offset(node), "Line is exceeding max line length. Break line before expression", true)
                .if_autocorrect_allowed(|| {
                    let indent = indent_config.child_indent_of(ast, node);
                    ast.upsert_whitespace_before_me(node, &indent);
                });
        }

        // When left hand side is a call expression which causes the max line length to be exceeded then first wrap that expression
        let call_expression = ast
            .children(node)
            .find(|&it| !ast.is_leaf(it) || !ast.is_code(it))
            .filter(|&it| ast.element_type(it) == CALL_EXPRESSION)
            .filter(|&it| self.causes_max_line_length_to_be_exceeded(ast, it));
        if let Some(call_expression) = call_expression {
            self.visit_call_expression(ast, call_expression, emit);
        }

        // The remainder (operation reference plus right hand side) might still cause the max line length to be exceeded
        let last_child_node = ast.last_child_node(node).expect("NullPointerException: lastChildNode");
        if (self.causes_max_line_length_to_be_exceeded(ast, last_child_node) || self.is_part_of_condition_exceeding_max_line_length(ast, node))
            && let Some(operation_reference) = ast.find_child_by_type(node, OPERATION_REFERENCE)
        {
            self.visit_operation_reference(ast, operation_reference, emit);
        }
    }

    /// The binary expression itself fits, but the closing parenthesis or opening brace of the `if (...) {` does not.
    fn is_part_of_condition_exceeding_max_line_length(&self, ast: &Ast, node: NodeId) -> bool {
        ast.parent(node)
            .filter(|&it| ast.element_type(it) == CONDITION)
            .map(|it| ast.last_child_leaf_or_self(it))
            .and_then(|it| ast.next_leaf_matching(it, |leaf| ast.is_white_space_with_newline(leaf)))
            .and_then(|it| ast.prev_leaf(it))
            .is_some_and(|it| self.causes_max_line_length_to_be_exceeded(ast, it))
    }

    fn visit_call_expression(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != CALL_EXPRESSION || ast.parent(node).map(|p| ast.element_type(p)) != Some(BINARY_EXPRESSION) {
            return;
        }
        // Breaking the lambda expression has priority over breaking value arguments
        let Some(function_literal) = ast
            .find_child_by_type(node, LAMBDA_ARGUMENT)
            .and_then(|it| ast.find_child_by_type(it, LAMBDA_EXPRESSION))
            .and_then(|it| ast.find_child_by_type(it, FUNCTION_LITERAL))
        else {
            return;
        };
        let indent_config = &self.indent_config;
        if let Some(lbrace) = ast.find_child_by_type(function_literal, LBRACE) {
            emit(ast, ast.start_offset(lbrace) + 1, "Newline expected after '{'", true).if_autocorrect_allowed(|| {
                let indent = indent_config.child_indent_of(ast, ast.parent(lbrace).expect("NullPointerException: parent!!"));
                ast.upsert_whitespace_after_me(lbrace, &indent);
            });
        }
        if let Some(rbrace) = ast.find_child_by_type(function_literal, RBRACE) {
            emit(ast, ast.start_offset(rbrace), "Newline expected before '}'", true).if_autocorrect_allowed(|| {
                let indent = indent_config.sibling_indent_of(ast, ast.parent(node).expect("NullPointerException: parent!!"));
                ast.upsert_whitespace_before_me(rbrace, &indent);
            });
        }
    }

    fn visit_operation_reference(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != OPERATION_REFERENCE {
            return;
        }
        // Allow `val foo = "string too long to fit on the line" +\n "more text"`
        if ast.is_white_space_with_newline(ast.next_sibling(node)) {
            return;
        }
        if ast.parent(node).map(|p| ast.element_type(p)) != Some(BINARY_EXPRESSION) {
            return;
        }
        // Raw string literals may exceed max-line-length; wrapping binary expressions inside them creates more chaos than it resolves
        if ast.parent_matching(node, |it| ast.element_type(it) == LONG_STRING_TEMPLATE_ENTRY).is_some() {
            return;
        }
        let operation_reference = node;
        let indent_config = &self.indent_config;
        let first_child = ast.first_child_node(operation_reference).expect("NullPointerException: firstChildNode");
        if ast.element_type(first_child) == ELVIS {
            let prev_white_space = ast.prev_leaf_matching(operation_reference, |it| ast.is_white_space(it));
            if prev_white_space.is_some() && !ast.is_white_space_with_newline(prev_white_space) {
                // Wrapping after the elvis operator violates 'chain-wrapping', so it is wrapped itself
                emit(ast, ast.start_offset(operation_reference), "Line is exceeding max line length. Break line before '?:'", true)
                    .if_autocorrect_allowed(|| {
                        let indent = indent_config.child_indent_of(ast, operation_reference);
                        ast.upsert_whitespace_before_me(operation_reference, &indent);
                    });
            }
        } else if let Some(next_sibling) = ast.next_sibling(operation_reference) {
            let message =
                format!("Line is exceeding max line length. Break line after '{}' in binary expression", ast.text(operation_reference));
            emit(ast, ast.start_offset(next_sibling), &message, true).if_autocorrect_allowed(|| {
                let indent = indent_config.child_indent_of(ast, operation_reference);
                ast.upsert_whitespace_before_me(next_sibling, &indent);
            });
        }
    }

    fn is_on_line_exceeding_max_line_length(&self, ast: &Ast, node: NodeId) -> bool {
        ast.has_no_max_line_length_suppression(node)
            && (self.max_line_length as i64) < ast.line_length(ast.drop_trailing_eol_comment(ast.leaves_on_line(node))) as i64
    }

    fn causes_max_line_length_to_be_exceeded(&self, ast: &Ast, node: NodeId) -> bool {
        if !ast.has_no_max_line_length_suppression(node) {
            return false;
        }
        let last_child_leaf = ast.last_child_leaf_or_self(node);
        let leaves = ast.drop_trailing_eol_comment(ast.leaves_on_line(node)).take_while(|&it| ast.prev_leaf(it) != Some(last_child_leaf));
        ast.line_length(leaves) as i64 > self.max_line_length as i64
    }
}
