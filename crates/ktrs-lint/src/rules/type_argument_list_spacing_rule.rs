//! Port of ktlint-ruleset-standard `TypeArgumentListSpacingRule.kt` (id `type-argument-list-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    CALL_EXPRESSION, GT, LAMBDA_ARGUMENT, LT, SUPER_EXPRESSION, SUPER_TYPE_LIST, TYPE_ARGUMENT_LIST, TYPE_REFERENCE, WHITE_SPACE,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[TYPE_ARGUMENT_LIST, SUPER_TYPE_LIST, SUPER_EXPRESSION]);

/// Lints and formats the spacing before and after the angle brackets of a type argument list.
pub struct TypeArgumentListSpacingRule {
    indent_config: IndentConfig,
}

impl TypeArgumentListSpacingRule {
    pub fn new() -> TypeArgumentListSpacingRule {
        TypeArgumentListSpacingRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for TypeArgumentListSpacingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for TypeArgumentListSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:type-argument-list-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
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
        match ast.element_type(node) {
            TYPE_ARGUMENT_LIST => {
                visit_function_declaration(ast, node, emit);
                self.visit_inside_type_argument_list(ast, node, emit);
            }
            SUPER_TYPE_LIST | SUPER_EXPRESSION => self.visit_inside_type_argument_list(ast, node, emit),
            _ => {}
        }
    }
}

fn visit_function_declaration(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    // No whitespace expected before type argument list of function call
    //    val list = listOf <String>()
    if let Some(it) = ast.prev_leaf(node).filter(|&it| ast.element_type(it) == WHITE_SPACE) {
        no_whitespace_expected(ast, it, emit);
    }

    // No whitespace expected after type argument list of function call
    //    val list = listOf<String> ()
    // unless it is part of a type reference (`fun foo(): List<Foo> { ... }`, `var bar: List<Bar> = emptyList()`)
    // or of a call expression followed by lambda (`bar<Foo> { ... }`)
    if let Some(it) = Some(node)
        .filter(|&it| !ast.is_part_of(it, TYPE_REFERENCE))
        .filter(|&it| !is_part_of_call_expression_followed_by_lambda(ast, it))
        .and_then(|it| ast.last_child_node(it))
        .and_then(|it| ast.next_leaf(it))
        .filter(|&it| ast.element_type(it) == WHITE_SPACE)
    {
        no_whitespace_expected(ast, it, emit);
    }
}

impl TypeArgumentListSpacingRule {
    fn visit_inside_type_argument_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let multiline = ast.text_contains(node, '\n');
        let expected_indent = if multiline {
            self.indent_config.child_indent_of(ast, node)
        } else {
            self.indent_config.sibling_indent_of(ast, node)
        };

        if let Some(next_sibling) = ast.find_child_by_type(node, LT).and_then(|it| ast.next_sibling(it)) {
            if multiline {
                // Otherwise, let Indentation rule fix the indentation
                if !ast.text_matches(next_sibling, &expected_indent) && ast.is_white_space_without_newline(next_sibling) {
                    emit(ast, ast.start_offset(next_sibling), "Expected newline", true)
                        .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(next_sibling, &expected_indent));
                }
            } else if ast.is_white_space(next_sibling) {
                // Disallow
                //    val list = listOf< String>()
                no_whitespace_expected(ast, next_sibling, emit);
            }
        }

        if let Some(prev_sibling) = ast.find_child_by_type(node, GT).and_then(|it| ast.prev_sibling(it)) {
            if multiline {
                // Otherwise, let Indentation rule fix the indentation
                if !ast.text_matches(prev_sibling, &expected_indent) && ast.is_white_space_without_newline(prev_sibling) {
                    emit(ast, ast.start_offset(prev_sibling), "Expected newline", true)
                        .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(prev_sibling, &expected_indent));
                }
            } else if ast.is_white_space(prev_sibling) {
                // Disallow
                //    val list = listOf<String >()
                no_whitespace_expected(ast, prev_sibling, emit);
            }
        }
    }
}

fn no_whitespace_expected(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    if !ast.text_matches(node, "") {
        emit(ast, ast.start_offset(node), "No whitespace expected at this position", true).if_autocorrect_allowed(|| ast.remove(node));
    }
}

fn is_part_of_call_expression_followed_by_lambda(ast: &Ast, node: NodeId) -> bool {
    ast.find_parent_by_type(node, CALL_EXPRESSION)
        .filter(|&it| ast.element_type(it) == CALL_EXPRESSION)
        .and_then(|it| ast.find_child_by_type(it, LAMBDA_ARGUMENT))
        .is_some()
}
