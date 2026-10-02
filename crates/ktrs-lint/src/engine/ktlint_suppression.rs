//! Port of ktlint-rule-engine `internal/KtlintSuppression.kt`, first half: where a `@Suppress` goes and
//! how suppression ids are written. The annotation building is in `ktlint_suppression_annotation.rs`.

use std::collections::BTreeSet;

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    self, BINARY_EXPRESSION, BLOCK, CLASS_INITIALIZER, COMMA, FILE, FUNCTION_LITERAL,
    LAMBDA_EXPRESSION, PRIMARY_CONSTRUCTOR, STRING_TEMPLATE, TYPE_ARGUMENT_LIST, TYPE_PARAMETER,
    TYPE_PARAMETER_LIST, TYPE_PROJECTION, VALUE_ARGUMENT, VALUE_ARGUMENT_LIST, VALUE_PARAMETER,
    VALUE_PARAMETER_LIST,
};

use crate::ast_node_extension::AstNodeExtension;
use crate::engine::ast_helpers::{
    psi_is_kt_declaration, psi_is_kt_expression, psi_kind, psi_parent,
};
use crate::engine::ktlint_suppression_annotation::{
    SuppressAnnotationType, create_suppress_annotation, find_suppression_annotations, merge_into,
};
use crate::engine::suppression_ids::remove_surrounding;

const KTLINT_PREFIX: &str = "ktlint";
const RULE_ID_SEPARATOR: &str = ":";
const STANDARD_RULE_SET_PREFIX: &str = "standard";
const EXPERIMENTAL_RULE_SET_PREFIX: &str = "experimental";
pub(crate) const KTLINT_SUPPRESSION_ID_ALL_RULES: &str = "\"ktlint\"";
const KTLINT_SUPPRESSION_ID_PREFIX: &str = "ktlint:";
const DOUBLE_QUOTE: &str = "\"";

/// `insertKtlintRuleSuppression(suppressionIds, forceFileAnnotation)`: adds the ids to the `@Suppress`
/// (else `@SuppressWarnings`) of the nearest declaration or expression that can carry one, creating a
/// `@Suppress` when there is none; `@file:Suppress` at the top level.
pub(crate) fn insert_ktlint_rule_suppression(
    ast: &mut Ast,
    node: NodeId,
    suppression_ids: &BTreeSet<String>,
    force_file_annotation: bool,
) {
    if suppression_ids.is_empty() {
        return;
    }
    let fully_qualified_suppression_ids: BTreeSet<String> = suppression_ids
        .iter()
        .map(|id| to_fully_qualified_ktlint_suppression_id(id))
        .collect();
    let target_ast_node = find_parent_declaration_or_expression(ast, node, force_file_annotation);
    let suppression_annotations = find_suppression_annotations(ast, target_ast_node);
    let find = |t: SuppressAnnotationType| {
        suppression_annotations
            .iter()
            .find(|(k, _)| *k == t)
            .map(|(_, v)| *v)
    };
    if let Some(annotation) = find(SuppressAnnotationType::Suppress) {
        merge_into(
            ast,
            &fully_qualified_suppression_ids,
            annotation,
            SuppressAnnotationType::Suppress,
        );
    } else if let Some(annotation) = find(SuppressAnnotationType::SuppressWarnings) {
        merge_into(
            ast,
            &fully_qualified_suppression_ids,
            annotation,
            SuppressAnnotationType::SuppressWarnings,
        );
    } else {
        create_suppress_annotation(
            ast,
            target_ast_node,
            SuppressAnnotationType::Suppress,
            &fully_qualified_suppression_ids,
        );
    }
}

/// The node that may carry the `@Suppress`: a declaration or an expression, with exceptions.
fn find_parent_declaration_or_expression(
    ast: &Ast,
    node: NodeId,
    force_file_annotation: bool,
) -> NodeId {
    if !force_file_annotation && is_top_level(ast, node) {
        return node;
    }
    let mut target_psi_element = Some(node);
    let mut is_annotation_for_binary_expression = false;
    loop {
        let target = target_psi_element.expect("NullPointerException: targetPsiElement.parent");
        let kind = psi_kind(ast, target);
        let is = |k: SyntaxKind| kind == Some(k);
        let parent = psi_parent(ast, target);
        let parent_kind = parent.and_then(|p| psi_kind(ast, p));
        let keeps_climbing = force_file_annotation
            || is(CLASS_INITIALIZER)
            || is(BINARY_EXPRESSION)
            || is(BLOCK)
            || is(PRIMARY_CONSTRUCTOR)
            || is(FUNCTION_LITERAL)
            || is(LAMBDA_EXPRESSION)
            || parent_kind == Some(STRING_TEMPLATE)
            || (psi_is_kt_expression(ast, target)
                && parent.is_some_and(|p| psi_is_kt_expression(ast, p))
                && parent_kind != Some(BLOCK)
                && !parent.is_some_and(|p| psi_is_kt_declaration(ast, p)))
            || (!psi_is_kt_declaration(ast, target) && !psi_is_kt_expression(ast, target));
        if !keeps_climbing {
            return target;
        }
        if is(BINARY_EXPRESSION) {
            is_annotation_for_binary_expression = true;
        }
        let next = if parent.is_none() {
            // Already at the root.
            return target;
        } else if is_list_element_type(ast.element_type(target))
            && !is_annotation_for_binary_expression
        {
            // Inside a list element, the annotation goes on that element (on the root of a binary expression
            // it would only cover its left-hand side).
            return target;
        } else if is_ignorable_list_element(ast, target) {
            // Between list elements, it moves to the next one (or to the parent of the list).
            ast.next_code_sibling(target)
                .map(|it| ast.first_child_leaf_or_self(it))
        } else {
            Some(target)
        };
        target_psi_element = next.and_then(|it| psi_parent(ast, it));
    }
}

fn is_list_type(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        TYPE_ARGUMENT_LIST | TYPE_PARAMETER_LIST | VALUE_ARGUMENT_LIST | VALUE_PARAMETER_LIST
    )
}

fn is_list_element_type(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        TYPE_PROJECTION | TYPE_PARAMETER | VALUE_ARGUMENT | VALUE_PARAMETER
    )
}

fn is_ignorable_list_element(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .is_some_and(|p| is_list_type(ast.element_type(p)))
        && (ast.element_type(node) == COMMA
            || ast.is_white_space(node)
            || ast.is_part_of_comment(node))
}

/// `isTopLevel()`: the file or a child of it.
pub(crate) fn is_top_level(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == FILE
        || ast
            .parent(node)
            .is_some_and(|p| ast.element_type(p) == FILE)
}

pub(crate) fn is_ktlint_suppression_id(s: &str) -> bool {
    s.strip_prefix(DOUBLE_QUOTE)
        .unwrap_or(s)
        .starts_with(KTLINT_SUPPRESSION_ID_PREFIX)
}

/// `"ktlint"` stays; anything else becomes `"ktlint:<set>:<rule>"` (quoted).
pub(crate) fn to_fully_qualified_ktlint_suppression_id(s: &str) -> String {
    match s {
        KTLINT_SUPPRESSION_ID_ALL_RULES => s.to_owned(),
        KTLINT_PREFIX => surround_with(s, DOUBLE_QUOTE),
        _ => {
            let qualified_rule_id = qualified_rule_id_string(remove_surrounding(s, DOUBLE_QUOTE));
            surround_with(
                &format!("{KTLINT_PREFIX}{RULE_ID_SEPARATOR}{qualified_rule_id}"),
                DOUBLE_QUOTE,
            )
        }
    }
}

/// `<set>:<rule>` without the `ktlint:` prefix; no set means `standard`, and `experimental` (the old
/// home of the experimental rules) is `standard` too.
pub(crate) fn qualified_rule_id_string(s: &str) -> String {
    let without_prefix = s.strip_prefix(KTLINT_SUPPRESSION_ID_PREFIX).unwrap_or(s);
    let rule_set_id = match without_prefix.split_once(RULE_ID_SEPARATOR) {
        Some((set, _)) if set != EXPERIMENTAL_RULE_SET_PREFIX => set,
        _ => STANDARD_RULE_SET_PREFIX,
    };
    let rule_id = without_prefix
        .split_once(RULE_ID_SEPARATOR)
        .map_or(without_prefix, |(_, id)| id);
    format!("{rule_set_id}{RULE_ID_SEPARATOR}{rule_id}")
}

fn surround_with(s: &str, string: &str) -> String {
    let inner = remove_surrounding(s, string);
    let prefixed = if inner.starts_with(string) {
        inner.to_owned()
    } else {
        format!("{string}{inner}")
    };
    prefixed + string
}
