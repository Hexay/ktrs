//! Port of ktlint-ruleset-standard `MultiLineIfElseRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    BINARY_EXPRESSION, BLOCK, DOT_QUALIFIED_EXPRESSION, ELSE, ELSE_KEYWORD, IF, LBRACE, RBRACE, RPAR, THEN, WHITE_SPACE,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::indent_config::IndentConfig;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// https://kotlinlang.org/docs/reference/coding-conventions.html#formatting-control-flow-statements
pub struct MultiLineIfElseRule {
    indent_config: IndentConfig,
}

impl MultiLineIfElseRule {
    pub fn new() -> MultiLineIfElseRule {
        MultiLineIfElseRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for MultiLineIfElseRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for MultiLineIfElseRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:multiline-if-else")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*INDENT_SIZE_PROPERTY), PropertyRef::from(&*INDENT_STYLE_PROPERTY)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let element_type = ast.element_type(node);
        if element_type != THEN && element_type != ELSE {
            return;
        }
        let first_child_type = ast.first_child_node(node).map(|c| ast.element_type(c));
        let first_grandchild_type = || ast.first_child_node(node).and_then(|c| ast.first_child_node(c)).map(|c| ast.element_type(c));

        // Ignore when already wrapped in a block
        if first_child_type == Some(BLOCK) {
            return;
        }
        if element_type == ELSE && first_child_type == Some(BINARY_EXPRESSION) && first_grandchild_type() == Some(IF) {
            // Allow `} else if (bar2) { … } else { … } ?: "something-else"`
            return;
        }
        if element_type == ELSE && first_child_type == Some(DOT_QUALIFIED_EXPRESSION) && first_grandchild_type() == Some(IF) {
            // Allow `} else if (bar2) { … } else { … }.plus("foo")`
            return;
        }

        if !ast.text_contains(ast.prev_sibling(node).expect("NullPointerException: prevSibling!!"), '\n') {
            let Some(first_child_type) = first_child_type else {
                // if block with an empty 'then' block
                return;
            };
            if first_child_type == IF {
                // Allow single line for: else if (...)
                return;
            }
            if ast.parent(node).map(|p| ast.text_contains(p, '\n')) == Some(false) {
                // Allow single line if statements as long as they are really simple (e.g. do not contain newlines)
                //    if (...) <statement> // no else statement
                //    if (...) <statement> else <statement>
                if ast.parent(node).and_then(|p| ast.parent(p)).map(|pp| ast.element_type(pp)) == Some(ELSE) {
                    // Except in case nested if-else-if on single line
                    //    if (...) <statement> else if (..) <statement>
                } else {
                    return;
                }
            }
        }

        let first_child = ast.first_child_node(node).expect("NullPointerException: firstChildNode.startOffset");
        let indent_config = &self.indent_config;
        emit(ast, ast.start_offset(first_child), "Missing { ... }", true).if_autocorrect_allowed(|| autocorrect(ast, indent_config, node));
    }
}

fn autocorrect(ast: &mut Ast, indent_config: &IndentConfig, node: NodeId) {
    let mut prev_leaves: Vec<NodeId> =
        ast.leaves(node, false).take_while(|&it| !matches!(ast.element_type(it), RPAR | ELSE_KEYWORD)).collect();
    prev_leaves.reverse();
    let mut next_leaves: Vec<NodeId> = ast
        .leaves(node, true)
        .take_while(|&it| ast.is_white_space_without_newline(it) || ast.is_part_of_comment(it))
        .collect();
    while next_leaves.last().is_some_and(|&it| ast.is_white_space_without_newline(it)) {
        next_leaves.pop();
    }

    if let Some(first) = prev_leaves.first().copied().filter(|&it| ast.is_white_space(it)) {
        ast.replace_text_with(first, " ");
    }
    let block = ast.new_composite(BLOCK);
    let previous_child = ast.first_child_node(node).unwrap();
    ast.replace_child(node, previous_child, block);
    let lbrace = ast.new_leaf(LBRACE, "{");
    ast.add_child(block, lbrace, None);
    let child_indent = indent_config.child_indent_of(ast, node);
    let white_space = ast.new_leaf(WHITE_SPACE, &child_indent);
    ast.add_child(block, white_space, None);
    let skip = prev_leaves.iter().take_while(|&&it| ast.is_white_space(it)).count();
    for &leaf in &prev_leaves[skip..] {
        ast.add_child(block, leaf, None);
    }
    ast.add_child(block, previous_child, None);
    for &leaf in &next_leaves {
        ast.add_child(block, leaf, None);
    }
    let indent = ast.indent(node);
    let white_space = ast.new_leaf(WHITE_SPACE, &indent);
    ast.add_child(block, white_space, None);
    let rbrace = ast.new_leaf(RBRACE, "}");
    ast.add_child(block, rbrace, None);

    // Make sure else starts on same line as newly inserted right brace
    if ast.element_type(node) == THEN
        && let Some(next) = ast.next_sibling_matching(node, |it| !ast.is_part_of_comment(it))
    {
        ast.upsert_whitespace_before_me(next, " ");
    }
}
