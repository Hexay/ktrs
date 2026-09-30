//! Port of ktlint-ruleset-standard `NoSemicolonsRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATION_ENTRY, BODY, CLASS, CLASS_BODY, DOC_COMMENT as KDOC, ENUM_ENTRY, ENUM_KEYWORD, FOR, IF, LBRACE,
    OBJECT_KEYWORD, SEMICOLON, THEN, WHILE,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{Emit, RuleId, RuleV2};

pub struct NoSemicolonsRule;

impl RuleV2 for NoSemicolonsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:no-semi")
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != SEMICOLON {
            return;
        }
        let next_leaf = ast.next_leaf(node);
        if does_not_require_pre_semi(ast, next_leaf) && is_no_semicolon_required_after(ast, node) {
            emit(ast, ast.start_offset(node), "Unnecessary semicolon", true).if_autocorrect_allowed(|| {
                let prev_leaf = ast.prev_leaf(node);
                ast.remove(node);
                if let Some(prev_leaf) = prev_leaf.filter(|&p| ast.is_white_space(p))
                    && (next_leaf.is_none() || ast.is_white_space(next_leaf))
                {
                    ast.remove(prev_leaf);
                }
            });
        } else if !ast.is_white_space(next_leaf) {
            if ast.is_white_space_with_newline(ast.prev_leaf(node)) {
                return;
            }
            // todo: move to a separate rule
            emit(ast, ast.start_offset(node) + 1, "Missing spacing after \";\"", true)
                .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(node, " "));
        }
    }
}

fn does_not_require_pre_semi(ast: &Ast, this: Option<NodeId>) -> bool {
    match this {
        None => true,
        Some(this) if ast.is_white_space(this) => {
            let next_leaf = ast.next_leaf_matching(this, |it| {
                ast.is_code(it) && ast.find_parent_by_type(it, KDOC).is_none() && ast.find_parent_by_type(it, ANNOTATION_ENTRY).is_none()
            });
            // \s+ and then eof
            next_leaf.is_none_or(|next_leaf| ast.text_contains(this, '\n') && ast.element_type(next_leaf) != LBRACE)
        }
        Some(_) => false,
    }
}

fn is_no_semicolon_required_after(ast: &Ast, node: NodeId) -> bool {
    if let Some(prev_code_leaf) = ast.prev_code_leaf(node) {
        if ast.element_type(prev_code_leaf) == OBJECT_KEYWORD {
            // https://github.com/ktlint/ktlint/issues/281
            return false;
        }
        if let Some(parent) = ast.parent(prev_code_leaf) {
            if is_loop_without_body(ast, parent) {
                // https://github.com/ktlint/ktlint/issues/955
                return false;
            }
            if is_if_expression_without_then(ast, parent) {
                return false;
            }
        }
    }
    // In case of an enum entry the semicolon (e.g. the node) is a direct child node of enum entry
    if ast.parent(node).map(|p| ast.element_type(p)) == Some(ENUM_ENTRY) {
        return is_last_code_leaf_before_closing_of_class_body(ast, Some(node));
    }
    if is_enum_class_without_values(ast, Some(node)) {
        return false;
    }
    true
}

fn is_loop_without_body(ast: &Ast, this: NodeId) -> bool {
    matches!(ast.element_type(this), WHILE | FOR) && ast.find_child_by_type(this, BODY).and_then(|b| ast.first_child_node(b)).is_none()
}

fn is_if_expression_without_then(ast: &Ast, this: NodeId) -> bool {
    ast.element_type(this) == IF && ast.find_child_by_type(this, THEN).and_then(|t| ast.first_child_node(t)).is_none()
}

fn is_last_code_leaf_before_closing_of_class_body(ast: &Ast, this: Option<NodeId>) -> bool {
    get_last_code_leaf_before_closing_of_class_body(ast, this) == this
}

fn get_last_code_leaf_before_closing_of_class_body(ast: &Ast, this: Option<NodeId>) -> Option<NodeId> {
    this.and_then(|n| ast.find_parent_by_type(n, CLASS_BODY))
        .map(|body| ast.last_child_leaf_or_self(body))
        .and_then(|leaf| ast.prev_code_leaf(leaf))
}

fn is_enum_class_without_values(ast: &Ast, this: Option<NodeId>) -> bool {
    this.filter(|_| !is_last_code_leaf_before_closing_of_class_body(ast, this))
        .and_then(|n| ast.find_parent_by_type(n, CLASS_BODY))
        .filter(|&body| this == ast.next_code_sibling(ast.first_child_node(body).expect("NullPointerException: firstChildNode")))
        .and_then(|body| ast.find_parent_by_type(body, CLASS))
        .is_some_and(|class| ast.has_modifier(class, ENUM_KEYWORD))
}
