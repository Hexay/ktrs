//! Port of ktlint-ruleset-standard `MultilineLoopRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{BLOCK, BODY, DO_KEYWORD, LBRACE, RBRACE, RPAR, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// https://developer.android.com/kotlin/style-guide#braces
pub struct MultilineLoopRule {
    indent_config: IndentConfig,
}

impl MultilineLoopRule {
    pub fn new() -> MultilineLoopRule {
        MultilineLoopRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for MultilineLoopRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for MultilineLoopRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:multiline-loop")
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
        if ast.element_type(node) != BODY {
            return;
        }
        // Ignore loop with empty body
        let Some(first_child) = ast.first_child_node(node) else { return };
        if ast.element_type(first_child) == BLOCK {
            return;
        }
        // Allow single line loop statements as long as they are really simple (e.g. do not contain newlines)
        //    for (...) <statement>
        //    while (...) <statement>
        //    do <statement> while (...)
        if !ast.parent(node).is_some_and(|it| ast.text_contains(it, '\n')) {
            return;
        }
        let indent_config = &self.indent_config;
        emit(ast, ast.start_offset(first_child), "Missing { ... }", true).if_autocorrect_allowed(|| autocorrect(ast, indent_config, node));
    }
}

fn autocorrect(ast: &mut Ast, indent_config: &IndentConfig, node: NodeId) {
    let mut prev_leaves: Vec<NodeId> =
        ast.leaves(node, false).take_while(|&it| !matches!(ast.element_type(it), RPAR | DO_KEYWORD)).collect();
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

    if let Some(next) = ast.next_sibling_matching(node, |it| !ast.is_part_of_comment(it)) {
        ast.upsert_whitespace_before_me(next, " ");
    }
}
