//! Port of ktlint-ruleset-standard `SpacingBetweenDeclarationsWithAnnotationsRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{ANNOTATION_ENTRY, MODIFIER_LIST, PROPERTY_ACCESSOR};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeQueries};
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// See <https://youtrack.jetbrains.com/issue/KT-35106>.
pub struct SpacingBetweenDeclarationsWithAnnotationsRule;

impl RuleV2 for SpacingBetweenDeclarationsWithAnnotationsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:spacing-between-declarations-with-annotations")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if is_declaration_or_property_accessor(ast, node) && is_annotated(ast, node) {
            visit_declaration(ast, node, emit);
        }
    }
}

fn is_declaration_or_property_accessor(ast: &Ast, node: NodeId) -> bool {
    ast.is_declaration(node) || ast.element_type(node) == PROPERTY_ACCESSOR
}

fn visit_declaration(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    if ast
        .prev_code_sibling(node)
        .filter(|&it| is_declaration_or_property_accessor(ast, it))
        .is_some_and(|prev_declaration| has_no_blank_line_between_declarations(ast, node, prev_declaration))
    {
        let prev_leaf = ast
            .prev_code_leaf(node)
            .and_then(|it| ast.next_leaf_matching(it, |it| ast.is_white_space(it)))
            .expect("NullPointerException: prevLeaf");
        emit(ast, ast.start_offset(prev_leaf) + 1, "Declarations and declarations with annotations should have an empty space between.", true)
            .if_autocorrect_allowed(|| {
                let indent = "\n".to_owned() + &ast.indent(node);
                ast.upsert_whitespace_before_me(prev_leaf, &indent);
            });
    }
}

fn is_annotated(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, MODIFIER_LIST)
        .is_some_and(|it| ast.children(it).any(|it| ast.element_type(it) == ANNOTATION_ENTRY))
}

fn has_no_blank_line_between_declarations(ast: &Ast, node: NodeId, prev_declaration: NodeId) -> bool {
    !ast.leaves(node, false)
        .take_while(|&it| !ast.is_code(it))
        .take_while(|&it| it != prev_declaration)
        .any(|it| is_blank_line(ast, it))
}

fn is_blank_line(ast: &Ast, node: NodeId) -> bool {
    ast.is_white_space(node) && ast.leaf_text(node).matches('\n').count() > 1
}
