//! Port of ktlint-ruleset-standard `FunctionExpressionBodyRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{self, BLOCK, COLON, EQ, FUN, LBRACE, RBRACE, RETURN, RETURN_KEYWORD, THROW, TYPE_REFERENCE, WHITE_SPACE};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::{CODE_STYLE_PROPERTY, CodeStyleValue, EditorConfig, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, Emit, RuleId, RuleV2, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;

/// Prefer using an expression body for functions with the body consisting of a single expression
/// (<https://kotlinlang.org/docs/coding-conventions.html#functions>).
pub struct FunctionExpressionBodyRule {
    #[allow(dead_code)]
    code_style: CodeStyleValue,
    indent_config: IndentConfig,
    traversal_state: TraversalState,
}

impl FunctionExpressionBodyRule {
    pub fn new() -> FunctionExpressionBodyRule {
        FunctionExpressionBodyRule {
            code_style: CODE_STYLE_PROPERTY.default_value,
            indent_config: IndentConfig::default_indent_config(),
            traversal_state: TraversalState::default(),
        }
    }
}

impl Default for FunctionExpressionBodyRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for FunctionExpressionBodyRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:function-expression-body")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*CODE_STYLE_PROPERTY),
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
        ]
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal_state)
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.code_style = editor_config.get(&CODE_STYLE_PROPERTY);
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        if self.indent_config.disabled() {
            self.traversal_state.stop_traversal_of_ast();
        }
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == BLOCK && ast.parent(node).map(|it| ast.element_type(it)) == Some(FUN) {
            visit_function_body(ast, node, emit);
        }
    }
}

fn visit_function_body(ast: &mut Ast, block: NodeId, emit: &mut Emit<'_>) {
    assert!(ast.element_type(block) == BLOCK, "IllegalArgumentException: Failed requirement.");
    if let Some(code_sibling) = Some(block)
        .filter(|&it| containing_only(ast, it, RETURN))
        .filter(|&it| count_return_keywords(ast, it) <= 1)
        .and_then(|it| ast.find_child_by_type(it, RETURN))
        .and_then(|it| ast.find_child_by_type(it, RETURN_KEYWORD))
        .and_then(|it| ast.next_sibling_matching(it, |s| !ast.is_white_space(s)))
    {
        emit(ast, ast.start_offset(block), "Function body should be replaced with body expression", true).if_autocorrect_allowed(|| {
            let parent = ast.parent(block).expect("NullPointerException: block.parent!!");
            // Insert the code sibling before the block
            let eq = ast.new_leaf(EQ, "=");
            ast.add_child(parent, eq, Some(block));
            let white_space = ast.new_leaf(WHITE_SPACE, " ");
            ast.add_child(parent, white_space, Some(block));
            ast.add_child(parent, code_sibling, Some(block));
            ast.remove(block);
        });
    }
    if let Some(throw_node) = Some(block).filter(|&it| containing_only(ast, it, THROW)).and_then(|it| ast.find_child_by_type(it, THROW)) {
        emit(ast, ast.start_offset(block), "Function body should be replaced with body expression", true).if_autocorrect_allowed(|| {
            let parent = ast.parent(block).expect("NullPointerException: block.parent!!");
            // Remove whitespace before block
            if let Some(it) = ast.prev_sibling(block).filter(|&it| ast.is_white_space(it)) {
                ast.remove(it);
            }
            if ast.find_child_by_type(parent, TYPE_REFERENCE).is_none() {
                // Insert Unit as return type as otherwise a compilation error results
                let colon = ast.new_leaf(COLON, ":");
                ast.add_child(parent, colon, Some(block));
                let white_space = ast.new_leaf(WHITE_SPACE, " ");
                ast.add_child(parent, white_space, Some(block));
                let unit_type_reference = create_unit_type_reference(ast);
                ast.add_child(parent, unit_type_reference, Some(block));
            }
            for (kind, text) in [(WHITE_SPACE, " "), (EQ, "="), (WHITE_SPACE, " ")] {
                let leaf = ast.new_leaf(kind, text);
                ast.add_child(parent, leaf, Some(block));
            }
            ast.add_child(parent, throw_node, Some(block));
            ast.remove(block);
        });
    }
}

fn containing_only(ast: &Ast, node: NodeId, i_element_type: SyntaxKind) -> bool {
    let mut others = ast.children(node).filter(|&it| !matches!(ast.element_type(it), LBRACE | RBRACE) && !ast.is_white_space(it));
    let single = match (others.next(), others.next()) {
        (Some(only), None) => Some(ast.element_type(only)),
        _ => None,
    };
    Some(i_element_type) == single
}

fn count_return_keywords(ast: &Ast, node: NodeId) -> usize {
    ast.leaves_in_closed_range(ast.first_child_leaf_or_self(node), ast.last_child_leaf_or_self(node))
        .filter(|&it| ast.element_type(it) == RETURN_KEYWORD)
        .count()
}

fn create_unit_type_reference(ast: &mut Ast) -> NodeId {
    ast.create_ast_node_from_text("fun foo(): Unit {}")
        .and_then(|it| ast.find_child_by_type(it, FUN))
        .and_then(|it| ast.find_child_by_type(it, TYPE_REFERENCE))
        .expect("IllegalStateException: Can not create function with unit type reference")
}
