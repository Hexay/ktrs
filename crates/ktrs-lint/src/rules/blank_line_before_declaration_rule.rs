//! Port of ktlint-ruleset-standard `BlankLineBeforeDeclarationRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    BLOCK, CLASS, CLASS_BODY, CLASS_INITIALIZER, EQ, FUN, FUNCTION_LITERAL, LBRACE, OBJECT_DECLARATION, OBJECT_LITERAL, PROPERTY,
    PROPERTY_ACCESSOR, RETURN_KEYWORD, VALUE_ARGUMENT, WHEN,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeQueries};
use crate::editorconfig::{CODE_STYLE_PROPERTY, CodeStyleValue, EditorConfig};
use crate::rule::{About, Emit, RuleId, RuleV2, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;

/// Insert a blank line before declarations. No blank line is inserted before between a class or method signature and the
/// first declaration in the class or method respectively. Also, no blank lines are inserted between consecutive properties.
#[derive(Default)]
pub struct BlankLineBeforeDeclarationRule {
    traversal_state: TraversalState,
}

impl RuleV2 for BlankLineBeforeDeclarationRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:blank-line-before-declaration")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal_state)
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        if editor_config.get(&CODE_STYLE_PROPERTY) == CodeStyleValue::IntellijIdea {
            self.traversal_state.stop_traversal_of_ast();
        }
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if let CLASS | CLASS_INITIALIZER | FUN | OBJECT_DECLARATION | PROPERTY | PROPERTY_ACCESSOR = ast.element_type(node) {
            visit_declaration(ast, node, emit);
        }
    }
}

fn visit_declaration(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let element_type = ast.element_type(node);
    let parent_type = ast.parent(node).map(|it| ast.element_type(it));
    let prev_code_sibling_type = ast.prev_code_sibling(node).map(|it| ast.element_type(it));
    if is_first_code_sibling_in_class_body(ast, node)
        || is_first_code_sibling_in_block(ast, node)
        || is_first_code_sibling_in_body_of_function_literal(ast, node)
        || is_consecutive_property(ast, node)
        || is_local_property(ast, node)
        // `when (val foo = foo()) {`
        || (element_type == PROPERTY && parent_type == Some(WHEN))
        // `val foo =\n    fun(): String { ... }`
        || (element_type == FUN && (prev_code_sibling_type == Some(EQ) || prev_code_sibling_type == Some(RETURN_KEYWORD)))
        // `val foo1 = foo2(fun() = 42)`
        || (element_type == FUN && parent_type == Some(VALUE_ARGUMENT))
        // `fun foo() =\n    object : Foo() { ... }`
        || (element_type == OBJECT_DECLARATION && parent_type == Some(OBJECT_LITERAL))
    {
        return;
    }
    if let Some(insert_before_node) = Some(node).filter(|&it| ast.is_declaration(it)).filter(|&it| !is_blank_line(ast, ast.prev_leaf(it))) {
        emit(ast, ast.start_offset(insert_before_node), "Expected a blank line for this declaration", true).if_autocorrect_allowed(|| {
            let indent = "\n".to_owned() + &ast.indent(node);
            ast.upsert_whitespace_before_me(insert_before_node, &indent);
        });
    }
}

fn is_blank_line(ast: &Ast, node: Option<NodeId>) -> bool {
    // `prevLeaf` never yields a composite: empty ones are skipped.
    node.is_none_or(|it| ast.leaf_text(it).starts_with("\n\n"))
}

fn is_first_code_sibling_in_class_body(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == CLASS_BODY)
        .and_then(|it| ast.find_child_by_type(it, LBRACE))
        .and_then(|it| ast.next_code_sibling(it))
        == Some(node)
}

fn is_first_code_sibling_in_block(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == BLOCK)
        .and_then(|it| ast.find_child_by_type(it, LBRACE))
        .and_then(|it| ast.next_code_sibling(it))
        == Some(node)
}

fn is_first_code_sibling_in_body_of_function_literal(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == BLOCK && ast.parent(it).map(|it| ast.element_type(it)) == Some(FUNCTION_LITERAL))
        .and_then(|it| ast.parent(it))
        .filter(|&it| ast.element_type(it) == FUNCTION_LITERAL)
        .and_then(|it| ast.find_child_by_type(it, BLOCK))
        // Upstream's `firstOrNull { isCode }` tests the receiver (`node`), not each child.
        .and_then(|it| ast.children(it).find(|_| ast.is_code(node)))
        == Some(node)
}

fn is_consecutive_property(ast: &Ast, node: NodeId) -> bool {
    Some(node)
        .filter(|&it| property_related(ast, it))
        .and_then(|it| ast.prev_code_sibling(it))
        .is_some_and(|it| property_related(ast, it) || property_related(ast, ast.parent(it).expect("NullPointerException: parent")))
}

fn is_local_property(ast: &Ast, node: NodeId) -> bool {
    Some(node)
        .filter(|&it| property_related(ast, it))
        .and_then(|it| ast.parent(it))
        .is_some_and(|it| ast.element_type(it) == BLOCK)
}

fn property_related(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == PROPERTY || ast.element_type(node) == PROPERTY_ACCESSOR
}
