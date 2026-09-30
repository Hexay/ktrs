//! Port of ktlint-ruleset-standard `ParameterListWrappingRule.kt` (id `parameter-list-wrapping`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATION_ENTRY, CALL_EXPRESSION, FUN, FUNCTION_LITERAL, FUNCTION_TYPE, LPAR, MODIFIER_LIST, NULLABLE_TYPE, RPAR,
    TYPE_PARAMETER_LIST, VALUE_ARGUMENT_LIST, VALUE_PARAMETER, VALUE_PARAMETER_LIST, WHITE_SPACE,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{
    CODE_STYLE_PROPERTY, CodeStyleValue, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, MAX_LINE_LENGTH_PROPERTY, PropertyRef,
};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;

pub struct ParameterListWrappingRule {
    code_style: CodeStyleValue,
    indent_config: IndentConfig,
    max_line_length: i32,
    traversal: TraversalState,
}

impl ParameterListWrappingRule {
    pub fn new() -> ParameterListWrappingRule {
        ParameterListWrappingRule {
            code_style: CODE_STYLE_PROPERTY.default_value,
            indent_config: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
            traversal: TraversalState::default(),
        }
    }
}

impl Default for ParameterListWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for ParameterListWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:parameter-list-wrapping")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*CODE_STYLE_PROPERTY),
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY),
        ]
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal)
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.code_style = editor_config.get(&CODE_STYLE_PROPERTY);
        self.max_line_length = max_line_length(editor_config);
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        if self.indent_config.disabled() {
            self.traversal.stop_traversal_of_ast();
        }
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        match ast.element_type(node) {
            NULLABLE_TYPE => self.visit_nullable_type(ast, node, emit),
            VALUE_PARAMETER_LIST => self.visit_parameter_list(ast, node, emit),
            _ => {}
        }
    }
}

impl ParameterListWrappingRule {
    fn visit_nullable_type(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        require(ast.element_type(node) == NULLABLE_TYPE);
        let Some(nullable_type) = Some(node)
            .filter(|&it| ast.has_no_max_line_length_suppression(it))
            // skip when max line length is not exceeded
            .filter(|_| (ast.column(node) as i64 - 1 + ast.text_length_utf16(node) as i64) > self.max_line_length as i64)
            .filter(|&it| !ast.is_white_space_with_newline(it))
            .filter(|&it| is_function_type_with_non_empty_value_parameter_list(ast, it))
        else {
            return;
        };
        if let Some(lpar) =
            ast.find_child_by_type(nullable_type, LPAR).filter(|&it| !ast.next_leaf(it).is_some_and(|it| ast.is_white_space_with_newline(it)))
        {
            emit(ast, ast.start_offset(lpar) + 1, "Expected new line before function type as it does not fit on a single line", true)
                .if_autocorrect_allowed(|| {
                    let indent = self.indent_config.child_indent_of(ast, node);
                    ast.upsert_whitespace_after_me(lpar, &indent);
                });
        }
        if let Some(rpar) =
            ast.find_child_by_type(nullable_type, RPAR).filter(|&it| !ast.prev_leaf(it).is_some_and(|it| ast.is_white_space_with_newline(it)))
        {
            emit(ast, ast.start_offset(rpar), "Expected new line after function type as it does not fit on a single line", true)
                .if_autocorrect_allowed(|| {
                    let indent = self.indent_config.parent_indent_of(ast, node);
                    ast.upsert_whitespace_before_me(rpar, &indent);
                });
        }
    }

    fn need_to_wrap_parameter_list(&self, ast: &Ast, node: NodeId) -> bool {
        let ktlint_official = self.code_style == CodeStyleValue::KtlintOfficial;
        if has_no_parameters(ast, node) {
            false
        } else if !ktlint_official && is_part_of_function_literal_in_non_ktlint_official_code_style(ast, node) {
            false
        } else if ktlint_official && contains_annotated_parameter(ast, node) {
            true
        } else if ktlint_official
            && is_part_of_function_literal_starting_on_same_line_as_closing_parenthesis_of_preceding_reference_expression(ast, node)
        {
            false
        } else if ast.text_contains(node, '\n') {
            true
        } else {
            self.is_on_line_exceeding_max_line_length(ast, node)
        }
    }

    fn visit_parameter_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if is_preceded_by_comment(ast, node) {
            emit(ast, ast.start_offset(node), "Parameter list should not be preceded by a comment", false);
        } else if self.need_to_wrap_parameter_list(ast, node) {
            // A lazy walk, like upstream's `children().forEach`: the next sibling is read after the edit.
            let mut child = ast.first_child_node(node);
            while let Some(c) = child {
                self.wrap_parameter_in_list(ast, c, emit);
                child = ast.next_sibling(c);
            }
        }
    }

    fn intended_indent(&self, ast: &Ast, child: NodeId) -> String {
        let parent = ast.parent(child).expect("NullPointerException: parent!!");
        // IDEA quirk: the parameters of `fun <\n T,\n R> test(` get the indent of that line instead of one more.
        let mut indent_level_fix = if is_fun_with_type_parameter_list_in_front(ast, parent) { -1 } else { 0 };
        if ast.element_type(child) == VALUE_PARAMETER {
            indent_level_fix += 1;
        }
        let indent_level = self.indent_config.indent_level_from(&ast.indent_without_newline_prefix(parent)) + indent_level_fix;
        let count = usize::try_from(indent_level)
            .unwrap_or_else(|_| panic!("IllegalArgumentException: Count 'n' must be non-negative, but was {indent_level}."));
        format!("\n{}", self.indent_config.indent.repeat(count))
    }

    fn wrap_parameter_in_list(&self, ast: &mut Ast, child: NodeId, emit: &mut Emit<'_>) {
        match ast.element_type(child) {
            LPAR => {
                if let Some(whitespace) = ast
                    .parent(child)
                    .filter(|&it| !is_value_parameter_list_in_function_type(ast, it))
                    .and_then(|it| ast.prev_leaf(it))
                    .filter(|&it| ast.is_white_space_with_newline(it))
                {
                    emit(ast, ast.start_offset(child), error_message(ast, child), true).if_autocorrect_allowed(|| ast.remove(whitespace));
                }
            }

            VALUE_PARAMETER | RPAR => {
                // aiming for
                // ... LPAR
                // <line indent + indentSize> VALUE_PARAMETER...
                // <line indent> RPAR
                let intended_indent = self.intended_indent(ast, child);
                let prev_leaf = ast.prev_leaf(child);
                if ast.is_white_space_with_newline(prev_leaf) {
                    // Already wrapped; fixing the size of the indent is the responsibility of the IndentationRule.
                } else if let Some(prev_leaf) = prev_leaf.filter(|&it| ast.is_white_space(it)) {
                    // The current child needs to be wrapped to a newline. The indent is purely based on the previous leaf;
                    // the indent rule, if enabled, runs after this rule and determines the final indentation.
                    emit(ast, ast.start_offset(child), error_message(ast, child), true)
                        .if_autocorrect_allowed(|| ast.replace_text_with(prev_leaf, &intended_indent));
                } else {
                    // Insert a new whitespace element in order to wrap the current child to a new line.
                    emit(ast, ast.start_offset(child), error_message(ast, child), true).if_autocorrect_allowed(|| {
                        if let Some(parent) = ast.parent(child) {
                            let white_space = ast.new_leaf(WHITE_SPACE, &intended_indent);
                            ast.add_child(parent, white_space, Some(child));
                        }
                    });
                }
                // Indentation of child nodes need to be fixed by the IndentationRule.
            }

            _ => {}
        }
    }

    fn is_on_line_exceeding_max_line_length(&self, ast: &Ast, node: NodeId) -> bool {
        if !ast.has_no_max_line_length_suppression(node) {
            return false;
        }
        let stop_leaf = ast.next_leaf_matching(node, |it| ast.is_white_space_with_newline(it)).and_then(|it| ast.next_leaf(it));
        let line_content: String = ast
            .drop_trailing_eol_comment(ast.leaves_on_line(node))
            .take_while(|&it| ast.prev_leaf(it) != stop_leaf)
            .map(|it| ast.text(it))
            .collect();
        let after = line_content.split_once('\n').map_or(line_content.as_str(), |(_, after)| after);
        let line = after.split_once('\n').map_or(after, |(before, _)| before);
        line.encode_utf16().count() as i64 > self.max_line_length as i64
    }
}

fn require(value: bool) {
    assert!(value, "IllegalArgumentException: Failed requirement.");
}

fn is_function_type_with_non_empty_value_parameter_list(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, FUNCTION_TYPE)
        .and_then(|it| ast.find_child_by_type(it, VALUE_PARAMETER_LIST))
        .and_then(|it| ast.find_child_by_type(it, VALUE_PARAMETER))
        .is_some()
}

fn has_no_parameters(ast: &Ast, node: NodeId) -> bool {
    require(ast.element_type(node) == VALUE_PARAMETER_LIST);
    ast.first_child_node(node).and_then(|it| ast.next_sibling(it)).map(|it| ast.element_type(it)) == Some(RPAR)
}

fn is_part_of_function_literal_in_non_ktlint_official_code_style(ast: &Ast, node: NodeId) -> bool {
    require(ast.element_type(node) == VALUE_PARAMETER_LIST);
    ast.parent(node).map(|it| ast.element_type(it)) == Some(FUNCTION_LITERAL)
}

fn is_part_of_function_literal_starting_on_same_line_as_closing_parenthesis_of_preceding_reference_expression(
    ast: &Ast,
    node: NodeId,
) -> bool {
    require(ast.element_type(node) == VALUE_PARAMETER_LIST);
    let start_of_function_literal = ast.first_child_leaf_or_self(node);
    let parent_type = |n: NodeId| ast.parent(n).map(|it| ast.element_type(it));
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == FUNCTION_LITERAL)
        .and_then(|it| ast.prev_code_leaf(it))
        .filter(|&it| parent_type(it) == Some(VALUE_ARGUMENT_LIST))
        .filter(|&it| ast.parent(it).and_then(parent_type) == Some(CALL_EXPRESSION))
        .is_some_and(|it| {
            !ast.leaves(it, true).take_while(|&it| it != start_of_function_literal).any(|it| ast.is_white_space_with_newline(it))
        })
}

fn contains_annotated_parameter(ast: &Ast, node: NodeId) -> bool {
    require(ast.element_type(node) == VALUE_PARAMETER_LIST);
    ast.children(node).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).any(|it| is_annotated(ast, it))
}

fn is_annotated(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, MODIFIER_LIST).is_some_and(|it| ast.children(it).any(|it| ast.element_type(it) == ANNOTATION_ENTRY))
}

fn is_preceded_by_comment(ast: &Ast, node: NodeId) -> bool {
    ast.prev_leaf_matching(node, |it| !ast.is_white_space(it))
        .and_then(|it| ast.prev_leaf(it))
        .is_some_and(|it| ast.is_part_of_comment(it))
}

fn error_message(ast: &Ast, node: NodeId) -> &'static str {
    match ast.element_type(node) {
        LPAR => "Unnecessary newline before \"(\"",
        VALUE_PARAMETER => "Parameter should start on a newline",
        RPAR => "Missing newline before \")\"",
        _ => panic!("UnsupportedOperationException"),
    }
}

fn is_value_parameter_list_in_function_type(ast: &Ast, node: NodeId) -> bool {
    Some(node)
        .filter(|&it| ast.element_type(it) == VALUE_PARAMETER_LIST)
        .and_then(|it| ast.parent(it))
        .is_some_and(|it| ast.element_type(it) == FUNCTION_TYPE)
}

/// Upstream's `takeIf { elementType == FUN }` tests the receiver (the parameter list), not the parent, so this is
/// always false; kept for parity.
fn is_fun_with_type_parameter_list_in_front(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .filter(|_| ast.element_type(node) == FUN)
        .and_then(|it| ast.find_child_by_type(it, TYPE_PARAMETER_LIST))
        .is_some_and(|it| ast.children(it).any(|it| ast.is_white_space_with_newline(it)))
}
