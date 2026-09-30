//! Port of ktlint-ruleset-standard `IfElseWrappingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{self, BLOCK, ELSE, ELSE_KEYWORD, IF, IF_KEYWORD, LBRACE, RBRACE, RPAR, THEN};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeQueries};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

const IF_THEN_ELSE_ELEMENT_TYPES: [SyntaxKind; 3] = [IF, THEN, ELSE];

/// Enforce that single line if statements are kept simple. A single line if statement is allowed only when it has at most one else
/// branch. Also, the branches of such an if statement may not be wrapped in a block.
pub struct IfElseWrappingRule {
    indent_config: IndentConfig,
}

impl IfElseWrappingRule {
    pub fn new() -> IfElseWrappingRule {
        IfElseWrappingRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for IfElseWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for IfElseWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:if-else-wrapping")
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
            self.visit_if(ast, node, emit);
        } else if ast.is_part_of_comment(node) && ast.parent(node).map(|it| ast.element_type(it)) == Some(IF) {
            visit_comment(ast, node, emit);
        }
    }
}

impl IfElseWrappingRule {
    fn visit_if(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let outer_if = outer_if(ast, node);
        let multiline_if = ast.text_contains(outer_if, '\n');
        let nested_if = is_nested_if(ast, outer_if);
        for element_type in [THEN, ELSE_KEYWORD, ELSE] {
            if let Some(it) = ast.find_child_by_type(node, element_type) {
                self.visit_element(ast, it, emit, multiline_if, nested_if);
            }
        }
    }

    fn visit_element(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>, multiline_if: bool, nested_if: bool) {
        if !multiline_if {
            visit_branch_single_line_if(ast, node, emit);
        }
        if multiline_if || nested_if {
            self.visit_branch(ast, node, emit, multiline_if);
        }
    }

    fn visit_branch(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>, multiline_if: bool) {
        let next_code_leaf_type = |ast: &Ast| ast.next_code_leaf(node).map(|it| ast.element_type(it));
        if multiline_if {
            if is_else_if(ast, node) {
                // Allow "else if" on single line
                return;
            }
            if ast.element_type(node) == ELSE_KEYWORD && next_code_leaf_type(ast) == Some(LBRACE) {
                // Allow "else {" on single line
                return;
            }
        } else {
            // Outer if statement is a single line statement
            if ast.element_type(node) == ELSE && next_code_leaf_type(ast) == Some(IF_KEYWORD) {
                // Ignore "else if" as it is reported via another message
                return;
            }
        }

        let this = find_first_node_in_block_to_be_indented(ast, node).unwrap_or(node);
        let expected_indent = if ast.next_sibling(this).map(|it| ast.element_type(it)) == Some(RBRACE) {
            ast.indent(node)
        } else {
            self.indent_config.sibling_indent_of(ast, node)
        };
        let target = if matches!(ast.element_type(this), THEN | ELSE | ELSE_KEYWORD) {
            ast.prev_leaf(this).expect("NullPointerException: prevLeaf!!")
        } else {
            this
        };
        if !ast.is_white_space_with_newline(target) {
            // Expected a newline with indent. Leave it up to the IndentationRule to determine exact indent
            emit(ast, ast.start_offset(this), "Expected a newline", true)
                .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(this, &expected_indent));
        }
    }
}

fn visit_branch_single_line_if(ast: &Ast, node: NodeId, emit: &mut Emit<'_>) {
    if ast.find_child_by_type(node, BLOCK).is_some() {
        // The then or else block is part of single line if-statement. To enforce that such statements are readable and
        // simple, blocks are forbidden entirely in single line if-statements.
        let message = format!(
            "A single line if-statement should be kept simple. The '{}' may not be wrapped in a block.",
            if ast.element_type(node) == THEN { "THEN" } else { "ELSE" },
        );
        emit(ast, ast.start_offset(node), &message, false);
    }
}

fn is_else_if(ast: &Ast, n: NodeId) -> bool {
    match ast.element_type(n) {
        IF => ast.prev_code_leaf(n).map(|it| ast.element_type(it)) == Some(ELSE_KEYWORD),
        ELSE | ELSE_KEYWORD => ast.next_code_leaf(n).map(|it| ast.element_type(it)) == Some(IF_KEYWORD),
        _ => false,
    }
}

fn find_first_node_in_block_to_be_indented(ast: &Ast, n: NodeId) -> Option<NodeId> {
    ast.find_child_by_type(n, BLOCK).map(|block| {
        ast.children(block)
            .find(|&it| ast.element_type(it) != LBRACE && !is_whitespace_before_comment(ast, it) && !ast.is_part_of_comment(it))
            .expect("NoSuchElementException: Sequence contains no element matching the predicate.")
    })
}

fn is_whitespace_before_comment(ast: &Ast, n: NodeId) -> bool {
    ast.is_white_space_without_newline(n) && ast.next_leaf(n).is_some_and(|it| ast.is_part_of_comment(it))
}

fn outer_if(ast: &Ast, n: NodeId) -> NodeId {
    assert!(ast.element_type(n) == IF, "IllegalArgumentException: Failed requirement.");
    ast.parents(n).take_while(|&it| IF_THEN_ELSE_ELEMENT_TYPES.contains(&ast.element_type(it))).last().unwrap_or(n)
}

fn is_nested_if(ast: &Ast, n: NodeId) -> bool {
    assert!(ast.element_type(n) == IF, "IllegalArgumentException: Failed requirement.");
    let first_child_type = |t| ast.find_child_by_type(n, t).and_then(|it| ast.first_child_node(it)).map(|it| ast.element_type(it));
    first_child_type(THEN) == Some(IF) || first_child_type(ELSE) == Some(IF)
}

fn visit_comment(ast: &Ast, comment: NodeId, emit: &mut Emit<'_>) {
    assert!(ast.is_part_of_comment(comment), "IllegalArgumentException: Failed requirement.");
    if ast.between_code_siblings(comment, RPAR, THEN)
        || ast.between_code_siblings(comment, THEN, ELSE_KEYWORD)
        || ast.between_code_siblings(comment, ELSE_KEYWORD, ELSE)
    {
        emit(ast, ast.start_offset(comment), "No comment expected at this location", false);
    }
}
