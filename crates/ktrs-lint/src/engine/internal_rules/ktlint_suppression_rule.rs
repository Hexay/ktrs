//! Port of ktlint-rule-engine `internal/rules/KtlintSuppressionRule.kt` (the rule half; directives are in
//! `ktlint_directive.rs`).
//!
//! Disallows the old `ktlint-disable`/`ktlint-enable` directives: a disable becomes a `@Suppress` on the
//! closest declaration or expression (`@file:Suppress` at the top level; block comments only when top level
//! or matched by an enable in the same parent), an enable is removed. Suppression ids in annotations must
//! be fully qualified and name a loaded rule.

use std::collections::BTreeSet;

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATION, ANNOTATION_ENTRY, BLOCK_COMMENT, CALL_EXPRESSION, EOL_COMMENT,
    LITERAL_STRING_TEMPLATE_ENTRY, STRING_TEMPLATE, VALUE_ARGUMENT, VALUE_ARGUMENT_LIST,
};

use super::ktlint_directive::{
    KtLintDirective, KtlintDirectiveType, SuppressionIdChange, ktlint_directive_or_null,
    should_be_converted_to_file_annotation,
};
use super::{INTERNAL_RULE_ABOUT, KTLINT_SUPPRESSION_RULE_ID};
use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::engine::ast_helpers::{find_child_by_type_recursively, replace_with};
use crate::engine::ktlint_suppression::{
    insert_ktlint_rule_suppression, is_ktlint_suppression_id, qualified_rule_id_string,
    to_fully_qualified_ktlint_suppression_id,
};
use crate::engine::suppression_ids::remove_surrounding;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};

const VISITED_TYPES: TokenSet = TokenSet::create(&[STRING_TEMPLATE, EOL_COMMENT, BLOCK_COMMENT]);

pub struct KtlintSuppressionRule {
    allowed_rule_ids: Vec<RuleId>,
}

impl KtlintSuppressionRule {
    pub fn new(allowed_rule_ids: Vec<RuleId>) -> KtlintSuppressionRule {
        KtlintSuppressionRule { allowed_rule_ids }
    }

    fn rule_id_validator(&self, rule_id: &str) -> bool {
        self.allowed_rule_ids.iter().any(|it| it.value() == rule_id)
    }
}

impl RuleV2 for KtlintSuppressionRule {
    fn rule_id(&self) -> RuleId {
        KTLINT_SUPPRESSION_RULE_ID
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        INTERNAL_RULE_ABOUT
    }

    /// The suppression locator no longer knows the old directives, so this rule may not be suppressed.
    fn ignores_ktlint_suppressions(&self) -> bool {
        true
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if is_ktlint_rule_suppression_in_annotation(ast, node) {
            self.visit_ktlint_suppression_in_annotation(ast, node, emit);
        }
        let validator = |id: &str| self.rule_id_validator(id);
        if let Some(directive) = ktlint_directive_or_null(ast, node, &validator) {
            visit_ktlint_directive(ast, &directive, emit);
        }
    }
}

fn is_ktlint_rule_suppression_in_annotation(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == STRING_TEMPLATE
        && is_ktlint_suppression_id(&ast.text(node))
        && ast
            .find_parent_by_type(node, VALUE_ARGUMENT)
            .is_some_and(|it| is_part_of_annotation(ast, it))
}

fn is_part_of_annotation(ast: &Ast, node: NodeId) -> bool {
    ast.parent_matching(node, |it| {
        matches!(ast.element_type(it), ANNOTATION | ANNOTATION_ENTRY)
    })
    .is_some()
}

impl KtlintSuppressionRule {
    fn visit_ktlint_suppression_in_annotation(
        &self,
        ast: &mut Ast,
        node: NodeId,
        emit: &mut Emit<'_>,
    ) {
        let Some(literal_string_template_entry) =
            find_child_by_type_recursively(ast, node, LITERAL_STRING_TEMPLATE_ENTRY)
        else {
            return;
        };
        let entry_text = ast.text(literal_string_template_entry);
        let prefixed_suppression =
            remove_surrounding(&to_fully_qualified_ktlint_suppression_id(&entry_text), "\"")
                .to_owned();
        let offset = ast.start_offset(literal_string_template_entry);
        if self.is_unknown_ktlint_suppression(&prefixed_suppression) {
            emit(
                ast,
                offset,
                &format!("Ktlint rule with id '{prefixed_suppression}' is unknown or not loaded"),
                false,
            );
        } else if prefixed_suppression != entry_text {
            emit(
                ast,
                offset,
                "Identifier to suppress ktlint rule must be fully qualified with the rule set id",
                true,
            )
            .if_autocorrect_allowed(|| {
                if let Some(it) = create_literal_string_template_entry(ast, &prefixed_suppression) {
                    replace_with(ast, literal_string_template_entry, it);
                }
            });
        }
    }

    fn is_unknown_ktlint_suppression(&self, s: &str) -> bool {
        let rule_id = qualified_rule_id_string(s);
        !self.allowed_rule_ids.iter().any(|it| it.value() == rule_id)
    }
}

fn remove_preceding_whitespace(ast: &mut Ast, node: NodeId) {
    if let Some(prev) = ast.prev_leaf(node).filter(|&it| ast.is_white_space(it)) {
        ast.remove(prev);
    }
}

fn create_literal_string_template_entry(
    ast: &mut Ast,
    prefixed_suppression: &str,
) -> Option<NodeId> {
    let mut node = ast.create_ast_node_from_text(&format!("listOf(\"{prefixed_suppression}\")"))?;
    for kind in [
        CALL_EXPRESSION,
        VALUE_ARGUMENT_LIST,
        VALUE_ARGUMENT,
        STRING_TEMPLATE,
        LITERAL_STRING_TEMPLATE_ENTRY,
    ] {
        node = ast.find_child_by_type(node, kind)?;
    }
    Some(node)
}

fn visit_ktlint_directive(ast: &mut Ast, directive: &KtLintDirective<'_>, emit: &mut Emit<'_>) {
    match directive.ktlint_directive_type {
        KtlintDirectiveType::KtlintDisable => {
            let prev_leaf = ast.prev_leaf(directive.node);
            if ast.element_type(directive.node) == EOL_COMMENT
                && ast.is_white_space_with_newline(prev_leaf)
            {
                remove_dangling_eol_comment_with_ktlint_disable_directive(ast, directive, emit);
            } else {
                visit_ktlint_disable_directive(ast, directive, emit);
            }
        }
        KtlintDirectiveType::KtlintEnable => remove_ktlint_enable_directive(ast, directive, emit),
    }
}

fn remove_dangling_eol_comment_with_ktlint_disable_directive(
    ast: &mut Ast,
    directive: &KtLintDirective<'_>,
    emit: &mut Emit<'_>,
) {
    emit(ast, directive.offset, "Directive 'ktlint-disable' in EOL comment is ignored as it is not preceded by a code element", true)
        .if_autocorrect_allowed(|| {
            remove_preceding_whitespace(ast, directive.node);
            ast.remove(directive.node);
        });
}

fn visit_ktlint_disable_directive(
    ast: &mut Ast,
    directive: &KtLintDirective<'_>,
    emit: &mut Emit<'_>,
) {
    let node = directive.node;
    if ast.element_type(node) == BLOCK_COMMENT
        && directive.has_no_matching_ktlint_enable_directive(ast)
    {
        emit(
            ast,
            directive.offset,
            "Directive 'ktlint-disable' is deprecated. The matching 'ktlint-enable' directive is not found in same scope. Replace with \
             @Suppress annotation",
            false,
        );
        return;
    }
    let autocorrect_decision = emit(
        ast,
        directive.offset,
        "Directive 'ktlint-disable' is deprecated. Replace with @Suppress annotation",
        true,
    );
    for change in &directive.suppression_id_changes {
        if let SuppressionIdChange::InvalidSuppressionId {
            original_rule_id,
            offset_original_rule_id,
        } = change
        {
            let offset = directive.offset as isize
                + directive.ktlint_directive_type.id().len() as isize
                + offset_original_rule_id;
            emit(
                ast,
                offset as usize,
                &format!("Ktlint rule with id '{original_rule_id}' is unknown or not loaded"),
                false,
            );
        }
    }
    autocorrect_decision.if_autocorrect_allowed(|| {
        let suppression_ids: BTreeSet<String> = directive
            .suppression_id_changes
            .iter()
            .filter_map(|change| match change {
                SuppressionIdChange::ValidSuppressionId { suppression_id } => {
                    Some(suppression_id.clone())
                }
                SuppressionIdChange::InvalidSuppressionId { .. } => None,
            })
            .collect();
        let target = if ast.element_type(node) == BLOCK_COMMENT
            && directive.should_be_promoted_to_parent_declaration(ast)
        {
            ast.parent(node).and_then(|p| ast.parent(p))
        } else {
            Some(node)
        };
        let force_file_annotation = should_be_converted_to_file_annotation(ast, node);
        if let Some(target) = target {
            insert_ktlint_rule_suppression(ast, target, &suppression_ids, force_file_annotation);
        }
        if ast.element_type(node) == EOL_COMMENT {
            remove_preceding_whitespace(ast, node);
        } else {
            let next_leaf = ast.next_leaf(node);
            if ast.is_white_space_with_newline(next_leaf) {
                if let Some(next_leaf) = ast.next_leaf(node) {
                    ast.remove(next_leaf);
                }
            } else {
                remove_preceding_whitespace(ast, node);
            }
        }
        ast.remove(node);
    });
}

fn remove_ktlint_enable_directive(
    ast: &mut Ast,
    directive: &KtLintDirective<'_>,
    emit: &mut Emit<'_>,
) {
    emit(
        ast,
        directive.offset,
        "Directive 'ktlint-enable' is obsolete after migrating to suppress annotations",
        true,
    )
    .if_autocorrect_allowed(|| {
        remove_preceding_whitespace(ast, directive.node);
        ast.remove(directive.node);
    });
}
