//! Port of ktlint-ruleset-standard `ParameterWrappingRule.kt` (id `parameter-wrapping`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CALL_EXPRESSION, COLON, COMMA, EQ, IDENTIFIER, TYPE_REFERENCE, VALUE_PARAMETER};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, KtlintVersion, MAX_LINE_LENGTH_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;
use crate::rules::property_wrapping_rule::sum_of_text_length_until;

const VISITED_TYPES: TokenSet = TokenSet::create(&[VALUE_PARAMETER]);

/// Inserts missing newlines inside a value parameter, indented relative to the parent (a best effort for when the
/// indentation rule does not run). Like `PropertyWrappingRule`, with subtle differences.
pub struct ParameterWrappingRule {
    indent_config: IndentConfig,
    max_line_length: i32,
    ktlint_version: KtlintVersion,
}

impl ParameterWrappingRule {
    pub fn new() -> ParameterWrappingRule {
        ParameterWrappingRule {
            indent_config: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
            ktlint_version: KtlintVersion::default(),
        }
    }
}

impl Default for ParameterWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for ParameterWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:parameter-wrapping")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
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
        self.ktlint_version = KtlintVersion::of(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == VALUE_PARAMETER {
            self.rearrange_value_parameter(ast, node, emit);
        }
    }
}

impl ParameterWrappingRule {
    fn rearrange_value_parameter(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(node) == VALUE_PARAMETER, "IllegalArgumentException: Failed requirement.");

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
            && ast.has_no_max_line_length_suppression_in(colon, self.ktlint_version)
            && base_indent_length + sum_of_text_length_until(ast, from_node, colon) > max_line_length
        {
            self.require_newline_after_leaf(ast, colon, emit);
            return;
        }

        if let Some(type_reference) = ast.find_child_by_type(node, TYPE_REFERENCE)
            && ast.has_no_max_line_length_suppression_in(type_reference, self.ktlint_version)
            && base_indent_length + sum_of_text_length_until(ast, from_node, or_trailing_comma(ast, type_reference)) > max_line_length
        {
            require_newline_before_leaf(ast, type_reference, emit);
            return;
        }

        if let Some(equal) = ast.find_child_by_type(node, EQ)
            && ast.has_no_max_line_length_suppression_in(equal, self.ktlint_version)
            && base_indent_length + sum_of_text_length_until(ast, from_node, or_trailing_comma(ast, equal)) > max_line_length
        {
            self.require_newline_after_leaf(ast, equal, emit);
            return;
        }

        if let Some(call_expression) = ast.find_child_by_type(node, CALL_EXPRESSION)
            && ast.has_no_max_line_length_suppression_in(call_expression, self.ktlint_version)
            && base_indent_length + sum_of_text_length_until(ast, from_node, or_trailing_comma(ast, call_expression)) > max_line_length
        {
            require_newline_before_leaf(ast, call_expression, emit);
        }
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

fn or_trailing_comma(ast: &Ast, node: NodeId) -> NodeId {
    ast.next_code_leaf(ast.last_child_leaf_or_self(node)).filter(|&it| ast.element_type(it) == COMMA).unwrap_or(node)
}

fn require_newline_before_leaf(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let message = format!("Missing newline before \"{}\"", ast.text(node));
    emit(ast, ast.start_offset(node) - 1, &message, true).if_autocorrect_allowed(|| {
        let indent = ast.indent(node);
        ast.upsert_whitespace_before_me(node, &indent);
    });
}
