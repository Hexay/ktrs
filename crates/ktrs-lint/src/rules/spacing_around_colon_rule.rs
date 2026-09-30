//! Port of ktlint-ruleset-standard `SpacingAroundColonRule.kt` (id `colon-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATION, ANNOTATION_ENTRY, BLOCK, CLASS, COLON, EQ, FUN, OBJECT_DECLARATION, PROPERTY, SECONDARY_CONSTRUCTOR,
    TYPE_CONSTRAINT, TYPE_PARAMETER_LIST,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct SpacingAroundColonRule;

impl RuleV2 for SpacingAroundColonRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:colon-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == COLON {
            remove_unexpected_newline_before(ast, node, emit);
            remove_unexpected_spacing_around(ast, node, emit);
            add_missing_spacing_around(ast, node, emit);
        }
    }
}

fn remove_unexpected_newline_before(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let Some(prev_leaf) = ast.prev_leaf(node).filter(|&it| ast.is_white_space_with_newline(it)) else { return };
    emit(ast, ast.start_offset(prev_leaf), "Unexpected newline before \":\"", true).if_autocorrect_allowed(|| {
        let parent_type = ast.parent(node).map(|p| ast.element_type(p));
        let mut prev_non_code_elements: Vec<NodeId> = ast.siblings(node, false).take_while(|&it| !ast.is_code(it)).collect();
        prev_non_code_elements.reverse();
        if parent_type == Some(PROPERTY) || parent_type == Some(FUN) {
            let next_sibling = ast.siblings(node, true).find(|&it| ast.element_type(it) == EQ).and_then(|eq| ast.next_sibling(eq));
            if let Some(next_sibling) = next_sibling {
                for &it in &prev_non_code_elements {
                    if let Some(parent) = ast.parent(node) {
                        ast.add_child(parent, it, Some(next_sibling));
                    }
                }
                if ast.is_white_space(next_sibling) {
                    ast.remove(next_sibling);
                }
            }
            let block_element = ast.siblings(node, true).find(|&it| ast.element_type(it) == BLOCK);
            if let Some(block_element) = block_element {
                let before = ast.first_child_node(block_element).and_then(|it| ast.next_sibling(it));
                let first = *prev_non_code_elements.first().expect("NoSuchElementException: List is empty.");
                // Upstream discards `drop(1)`: the removed first whitespace is re-added to the block below.
                if ast.is_white_space(first) {
                    ast.remove(first);
                }
                let last = *prev_non_code_elements.last().expect("NoSuchElementException: List is empty.");
                let elements = if ast.is_white_space_with_newline(last) {
                    ast.remove(last);
                    &prev_non_code_elements[..prev_non_code_elements.len() - 1]
                } else {
                    &prev_non_code_elements[..]
                };
                for &it in elements {
                    ast.add_child(block_element, it, before);
                }
            }
        } else if ast.prev_leaf(prev_leaf).is_some_and(|it| ast.is_part_of_comment(it)) {
            let next_leaf = ast.next_leaf(node);
            for &it in &prev_non_code_elements {
                if let Some(parent) = ast.parent(node) {
                    ast.add_child(parent, it, next_leaf);
                }
            }
            if let Some(next_leaf) = next_leaf.filter(|&it| ast.is_white_space(it)) {
                ast.remove(next_leaf);
            }
        } else {
            let text = ast.text(prev_leaf);
            if spacing_before(ast, node) {
                ast.replace_text_with(prev_leaf, " ");
            } else {
                ast.remove(prev_leaf);
            }
            ast.upsert_whitespace_after_me(node, &text);
        }
    });
}

fn remove_unexpected_spacing_around(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    if ast.is_white_space_without_newline(ast.prev_sibling(node)) && no_spacing_before(ast, node) {
        emit(ast, ast.start_offset(node), "Unexpected spacing before \":\"", true).if_autocorrect_allowed(|| {
            if let Some(prev_sibling) = ast.prev_sibling(node) {
                ast.remove(prev_sibling);
            }
        });
    }
    if ast.is_white_space_without_newline(ast.next_sibling(node)) && spacing_after(ast, node) {
        emit(ast, ast.start_offset(node), "Unexpected spacing after \":\"", true).if_autocorrect_allowed(|| {
            if let Some(next_sibling) = ast.next_sibling(node) {
                ast.remove(next_sibling);
            }
        });
    }
}

fn add_missing_spacing_around(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let missing_spacing_before = !ast.is_white_space(ast.prev_sibling(node)) && spacing_before(ast, node);
    let missing_spacing_after = !ast.is_white_space(ast.next_sibling(node)) && no_spacing_after(ast, node);
    if missing_spacing_before && missing_spacing_after {
        emit(ast, ast.start_offset(node), "Missing spacing around \":\"", true).if_autocorrect_allowed(|| {
            ast.upsert_whitespace_before_me(node, " ");
            ast.upsert_whitespace_after_me(node, " ");
        });
    } else if missing_spacing_before {
        emit(ast, ast.start_offset(node), "Missing spacing before \":\"", true)
            .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(node, " "));
    } else if missing_spacing_after {
        emit(ast, ast.start_offset(node) + 1, "Missing spacing after \":\"", true)
            .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(node, " "));
    }
}

fn spacing_before(ast: &Ast, node: NodeId) -> bool {
    match ast.parent(node).map(|p| ast.element_type(p)) {
        Some(CLASS | OBJECT_DECLARATION) => true,
        // constructor : this/super
        Some(SECONDARY_CONSTRUCTOR) => true,
        // where T : S
        Some(TYPE_CONSTRAINT) => true,
        _ => ast.parent(node).and_then(|p| ast.parent(p)).map(|pp| ast.element_type(pp)) == Some(TYPE_PARAMETER_LIST),
    }
}

fn no_spacing_before(ast: &Ast, node: NodeId) -> bool {
    !spacing_before(ast, node)
}

fn spacing_after(ast: &Ast, node: NodeId) -> bool {
    matches!(ast.parent(node).map(|p| ast.element_type(p)), Some(ANNOTATION | ANNOTATION_ENTRY))
}

fn no_spacing_after(ast: &Ast, node: NodeId) -> bool {
    !spacing_after(ast, node)
}
