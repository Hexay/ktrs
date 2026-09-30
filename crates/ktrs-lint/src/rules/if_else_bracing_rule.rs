//! Port of ktlint-ruleset-standard `IfElseBracingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{BLOCK, ELSE, ELSE_KEYWORD, IF, LBRACE, RBRACE, RPAR, THEN, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// All branches of the if-statement should be wrapped between braces if at least one branch is wrapped between braces.
pub struct IfElseBracingRule {
    indent_config: IndentConfig,
}

impl IfElseBracingRule {
    pub fn new() -> IfElseBracingRule {
        IfElseBracingRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for IfElseBracingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for IfElseBracingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:if-else-bracing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*INDENT_SIZE_PROPERTY), PropertyRef::from(&*INDENT_STYLE_PROPERTY)]
    }

    fn is_official_code_style(&self) -> bool {
        true
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == IF {
            self.visit_if_statement(ast, node, emit);
        }
    }
}

impl IfElseBracingRule {
    fn visit_if_statement(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let then_node = ast.find_child_by_type(node, THEN).expect("IllegalArgumentException: Can not find THEN branch in IF");
        let Some(else_node) = ast.find_child_by_type(node, ELSE) else { return };
        let parent_if_bracing = ast
            .parent(node)
            .filter(|&it| ast.element_type(it) == ELSE)
            .and_then(|it| ast.parent(it))
            .is_some_and(|it| has_bracing(ast, Some(it)));
        let then_bracing = has_bracing(ast, Some(then_node));
        let else_bracing = has_bracing(ast, Some(else_node));
        if parent_if_bracing || then_bracing || else_bracing {
            if !then_bracing {
                self.visit_branch_without_braces(ast, then_node, emit);
            }
            if !else_bracing {
                if ast.first_child_node(else_node).map(|it| ast.element_type(it)) != Some(IF) {
                    self.visit_branch_without_braces(ast, else_node, emit);
                } else {
                    // Postpone changing the else-if until that node is being processed
                }
            }
        }
    }

    fn visit_branch_without_braces(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) -> bool {
        let offset = ast.first_child_node(node).map_or_else(|| ast.start_offset(node), |it| ast.start_offset(it));
        emit(
            ast,
            offset,
            "All branches of the if statement should be wrapped between braces if at least one branch is wrapped between braces",
            true,
        )
        .if_autocorrect_allowed(|| self.autocorrect(ast, node));
        true
    }

    fn autocorrect(&self, ast: &mut Ast, node: NodeId) {
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
        let previous_child = ast.first_child_node(node);
        match previous_child {
            None => ast.add_child(node, block, None),
            Some(previous_child) => ast.replace_child(node, previous_child, block),
        }
        let lbrace = ast.new_leaf(LBRACE, "{");
        ast.add_child(block, lbrace, None);
        if previous_child.is_some() {
            let child_indent = self.indent_config.child_indent_of(ast, node);
            let white_space = ast.new_leaf(WHITE_SPACE, &child_indent);
            ast.add_child(block, white_space, None);
        }
        let skip = prev_leaves.iter().take_while(|&&it| ast.is_white_space(it)).count();
        for &leaf in &prev_leaves[skip..] {
            ast.add_child(block, leaf, None);
        }
        if let Some(previous_child) = previous_child {
            ast.add_child(block, previous_child, None);
        }
        for &leaf in &next_leaves {
            ast.add_child(block, leaf, None);
        }
        if previous_child.is_some() {
            let indent = ast.indent(node);
            let white_space = ast.new_leaf(WHITE_SPACE, &indent);
            ast.add_child(block, white_space, None);
        }
        let rbrace = ast.new_leaf(RBRACE, "}");
        ast.add_child(block, rbrace, None);

        // Make sure else starts on same line as newly inserted right brace
        if ast.element_type(node) == THEN
            && let Some(next) = ast.next_sibling_matching(node, |it| !ast.is_part_of_comment(it))
        {
            ast.upsert_whitespace_before_me(next, " ");
        }
    }
}

fn has_bracing(ast: &Ast, n: Option<NodeId>) -> bool {
    let Some(n) = n else { return false };
    match ast.element_type(n) {
        BLOCK => true,
        IF => has_bracing(ast, ast.find_child_by_type(n, THEN)) || has_bracing(ast, ast.find_child_by_type(n, ELSE)),
        _ => has_bracing(ast, ast.first_child_node(n)),
    }
}
