//! Port of ktlint-ruleset-standard `PropertyWrappingRule.kt` (id `property-wrapping`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CALL_EXPRESSION, COLON, EQ, IDENTIFIER, PROPERTY, TYPE_REFERENCE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, MAX_LINE_LENGTH_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;

/// Inserts missing newlines inside a property, indented relative to the parent (a best effort for when the
/// indentation rule does not run). Like `ParameterWrappingRule`, with subtle differences.
pub struct PropertyWrappingRule {
    indent_config: IndentConfig,
    max_line_length: i32,
}

impl PropertyWrappingRule {
    pub fn new() -> PropertyWrappingRule {
        PropertyWrappingRule { indent_config: IndentConfig::default_indent_config(), max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value }
    }
}

impl Default for PropertyWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for PropertyWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:property-wrapping")
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

    // Upstream's `line` counter only feeds trace logging, which is not ported.
    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        self.max_line_length = max_line_length(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == PROPERTY {
            self.rearrange_property(ast, node, emit);
        }
    }
}

impl PropertyWrappingRule {
    fn rearrange_property(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(node) == PROPERTY, "IllegalArgumentException: Failed requirement.");

        let base_indent_length = ast.indent_without_newline_prefix(node).encode_utf16().count() as i64;

        // Find the first node after the indenting whitespace on the same line as the identifier
        let node_first_child_leaf_or_self = ast.first_child_leaf_or_self(node);
        let from_node = ast
            .find_child_by_type(node, IDENTIFIER)
            .and_then(|it| {
                ast.leaves_backwards_including_self(it)
                    .find(|&it| ast.is_white_space_with_newline(ast.prev_leaf(it)) || it == node_first_child_leaf_or_self)
            })
            .unwrap_or(node);
        let max_line_length = self.max_line_length as i64;

        if let Some(colon) = ast.find_child_by_type(node, COLON)
            && ast.has_no_max_line_length_suppression(colon)
            && base_indent_length + sum_of_text_length_until(ast, from_node, colon) > max_line_length
        {
            self.require_newline_after_leaf(ast, colon, emit);
            return;
        }

        if let Some(type_reference) = ast.find_child_by_type(node, TYPE_REFERENCE)
            && ast.has_no_max_line_length_suppression(type_reference)
            && base_indent_length + sum_of_text_length_until(ast, from_node, type_reference) > max_line_length
        {
            self.require_newline_before_leaf(ast, type_reference, emit);
            return;
        }

        if let Some(equal) = ast.find_child_by_type(node, EQ)
            && ast.has_no_max_line_length_suppression(equal)
            && base_indent_length + sum_of_text_length_until(ast, from_node, equal) > max_line_length
        {
            self.require_newline_after_leaf(ast, equal, emit);
            return;
        }

        if let Some(call_expression) = ast.find_child_by_type(node, CALL_EXPRESSION)
            && ast.has_no_max_line_length_suppression(call_expression)
            && base_indent_length + sum_of_text_length_until(ast, from_node, call_expression) > max_line_length
        {
            self.require_newline_before_leaf(ast, call_expression, emit);
        }
    }

    fn require_newline_before_leaf(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let message = format!("Missing newline before \"{}\"", ast.text(node));
        emit(ast, ast.start_offset(node) - 1, &message, true).if_autocorrect_allowed(|| {
            let indent = self.indent_config.child_indent_of(ast, node);
            ast.upsert_whitespace_before_me(node, &indent);
        });
    }

    fn require_newline_after_leaf(&self, ast: &mut Ast, node_after_which_newline_is_required: NodeId, emit: &mut Emit<'_>) {
        let node_to_fix = node_after_which_newline_is_required;
        let message = format!("Missing newline after \"{}\"", ast.text(node_after_which_newline_is_required));
        emit(ast, ast.start_offset(node_after_which_newline_is_required) + 1, &message, true).if_autocorrect_allowed(|| {
            let indent = self.indent_config.child_indent_of(ast, node_to_fix);
            ast.upsert_whitespace_after_me(node_to_fix, &indent);
        });
    }
}

/// `ASTNode.sumOfTextLengthUntil`, in UTF-16 units.
pub(super) fn sum_of_text_length_until(ast: &Ast, from: NodeId, ast_node: NodeId) -> i64 {
    let stop_at_leaf = ast.last_child_leaf_or_self(ast_node);
    ast.leaves_forwards_including_self(from)
        .take_while(|&it| !ast.is_white_space_with_newline(it) && ast.prev_leaf(it) != Some(stop_at_leaf))
        .map(|it| ast.text_length_utf16(it) as i64)
        .sum()
}
