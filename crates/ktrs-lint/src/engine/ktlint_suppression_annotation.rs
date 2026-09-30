//! Port of ktlint-rule-engine `internal/KtlintSuppression.kt`, second half: reading and (re)building the
//! `@Suppress`/`@SuppressWarnings` annotations, from parsed snippets as upstream does.

use std::collections::BTreeSet;

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATED_EXPRESSION, ANNOTATION, ANNOTATION_ENTRY, CLASS, COLLECTION_LITERAL_EXPRESSION,
    CONSTRUCTOR_CALLEE, ENUM_ENTRY, FILE, FILE_ANNOTATION_LIST, FUN, FUNCTION_LITERAL, IDENTIFIER,
    MODIFIER_LIST, PACKAGE_DIRECTIVE, PRIMARY_CONSTRUCTOR, PROPERTY, PROPERTY_ACCESSOR,
    REFERENCE_EXPRESSION, SECONDARY_CONSTRUCTOR, STRING_TEMPLATE, TYPE_REFERENCE, USER_TYPE,
    VALUE_ARGUMENT, VALUE_ARGUMENT_LIST, VALUE_PARAMETER, WHITE_SPACE,
};

use crate::ast_node_extension::AstNodeExtension;
use crate::engine::ast_helpers::{
    create_psi_file_first_child_from_text, find_child_by_type_recursively,
    indent_without_newline_prefix, is_root, psi_is_kt_expression, psi_kind, replace_with,
};
use crate::engine::kotlin_text::{trim_indent, trim_margin};
use crate::engine::ktlint_suppression::{
    KTLINT_SUPPRESSION_ID_ALL_RULES, is_ktlint_suppression_id,
};

/// The indentation of the raw-string snippets in the Kotlin source; `trimIndent`/`trimMargin` see it.
const SOURCE_INDENT: &str = "                    ";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SuppressAnnotationType {
    Suppress,
    SuppressWarnings,
}

impl SuppressAnnotationType {
    fn annotation_name(self) -> &'static str {
        match self {
            SuppressAnnotationType::Suppress => "Suppress",
            SuppressAnnotationType::SuppressWarnings => "SuppressWarnings",
        }
    }

    fn find_by_id_or_null(id: &str) -> Option<SuppressAnnotationType> {
        [
            SuppressAnnotationType::Suppress,
            SuppressAnnotationType::SuppressWarnings,
        ]
        .into_iter()
        .find(|t| t.annotation_name() == id)
    }
}

/// Adds `suppressions` to the existing ones of `annotation_node`; a suppression of all ktlint rules drops
/// the rule-specific ones.
pub(crate) fn merge_into(
    ast: &mut Ast,
    suppressions: &BTreeSet<String>,
    annotation_node: NodeId,
    suppress_type: SuppressAnnotationType,
) {
    let mut merged = existing_suppressions(ast, annotation_node);
    merged.extend(suppressions.iter().cloned());
    if merged.contains(KTLINT_SUPPRESSION_ID_ALL_RULES) {
        merged.retain(|it| !is_ktlint_suppression_id(it));
    }
    create_suppress_annotation(ast, annotation_node, suppress_type, &merged);
}

fn existing_suppressions(ast: &Ast, n: NodeId) -> BTreeSet<String> {
    existing_suppressions_from_named_argument_or_null(ast, n)
        .unwrap_or_else(|| get_value_arguments(ast, n))
}

fn existing_suppressions_from_named_argument_or_null(
    ast: &Ast,
    n: NodeId,
) -> Option<BTreeSet<String>> {
    let collection = find_child_by_type_recursively(ast, n, COLLECTION_LITERAL_EXPRESSION)?;
    Some(
        ast.children(collection)
            .filter(|&it| ast.element_type(it) == STRING_TEMPLATE)
            .map(|it| ast.text(it))
            .collect(),
    )
}

/// The annotations by type (the last one of a type wins, as `toMap` does).
pub(crate) fn find_suppression_annotations(
    ast: &Ast,
    n: NodeId,
) -> Vec<(SuppressAnnotationType, NodeId)> {
    let list = if is_root(ast, n) {
        ast.find_child_by_type(n, FILE_ANNOTATION_LIST)
    } else if ast.element_type(n) == ANNOTATED_EXPRESSION {
        Some(n)
    } else {
        ast.find_child_by_type(n, MODIFIER_LIST)
    };
    let mut map: Vec<(SuppressAnnotationType, NodeId)> = Vec::new();
    for modifier in list.into_iter().flat_map(|l| ast.children(l)) {
        if let Some(t) = suppression_annotation_type_or_null(ast, modifier) {
            map.retain(|(k, _)| *k != t);
            map.push((t, modifier));
        }
    }
    map
}

fn suppression_annotation_type_or_null(ast: &Ast, n: NodeId) -> Option<SuppressAnnotationType> {
    if !matches!(ast.element_type(n), ANNOTATION | ANNOTATION_ENTRY) {
        return None;
    }
    let mut node = n;
    for kind in [
        CONSTRUCTOR_CALLEE,
        TYPE_REFERENCE,
        USER_TYPE,
        REFERENCE_EXPRESSION,
        IDENTIFIER,
    ] {
        node = ast.find_child_by_type(node, kind)?;
    }
    SuppressAnnotationType::find_by_id_or_null(&ast.text(node))
}

fn get_value_arguments(ast: &Ast, n: NodeId) -> BTreeSet<String> {
    let Some(list) = ast.find_child_by_type(n, VALUE_ARGUMENT_LIST) else {
        return BTreeSet::new();
    };
    ast.children(list)
        .filter(|&it| ast.element_type(it) == VALUE_ARGUMENT)
        .map(|it| ast.text(it))
        .collect()
}

/// Replaces (or creates) the annotation on `n` with one listing `suppressions`, sorted.
pub(crate) fn create_suppress_annotation(
    ast: &mut Ast,
    n: NodeId,
    suppress_type: SuppressAnnotationType,
    suppressions: &BTreeSet<String>,
) {
    let target_node = if ast.element_type(n) == ANNOTATION_ENTRY {
        ast.parent(n).expect("NullPointerException: parent!!")
    } else {
        n
    };
    let psi = psi_kind(ast, n);
    if psi == Some(FILE) {
        let file_annotation = create_file_annotation(ast, suppress_type, suppressions);
        create_file_annotation_list(ast, n, file_annotation);
    } else if psi == Some(ANNOTATION_ENTRY) {
        if ast
            .parent(n)
            .is_some_and(|p| ast.element_type(p) == FILE_ANNOTATION_LIST)
        {
            let file_annotation = create_file_annotation(ast, suppress_type, suppressions);
            let first = ast
                .first_child_node(file_annotation)
                .expect("NullPointerException: firstChildNode");
            replace_with(ast, n, first);
        } else {
            let modifier_list =
                create_modifier_list_with_annotation_entry(ast, suppress_type, suppressions);
            let entry = ast
                .find_child_by_type(modifier_list, ANNOTATION_ENTRY)
                .expect("NullPointerException: ANNOTATION_ENTRY!!");
            replace_with(ast, n, entry);
        }
    } else if matches!(
        psi,
        Some(
            CLASS
                | ENUM_ENTRY
                | FUN
                | FUNCTION_LITERAL
                | PRIMARY_CONSTRUCTOR
                | SECONDARY_CONSTRUCTOR
                | PROPERTY
                | PROPERTY_ACCESSOR
        )
    ) {
        let indent = ast.indent(n);
        let white_space = ast.new_leaf(WHITE_SPACE, &indent);
        let first = ast.first_child_node(n);
        ast.add_child(n, white_space, first);
        let modifier_list =
            create_modifier_list_with_annotation_entry(ast, suppress_type, suppressions);
        let first = ast.first_child_node(n);
        ast.add_child(n, modifier_list, first);
    } else if psi_is_kt_expression(ast, target_node)
        && ast.element_type(target_node) != ANNOTATED_EXPRESSION
        && ast.element_type(n) != VALUE_PARAMETER
    {
        let annotated_expression =
            create_annotated_expression(ast, target_node, suppress_type, suppressions);
        let parent = ast.parent(n).expect("NullPointerException: parent!!");
        ast.replace_child(parent, target_node, annotated_expression);
    } else {
        let modifier_list =
            create_modifier_list_with_annotation_entry(ast, suppress_type, suppressions);
        let entry = ast
            .find_child_by_type(modifier_list, ANNOTATION_ENTRY)
            .expect("NullPointerException: ANNOTATION_ENTRY!!");
        let parent = ast.parent(n).expect("NullPointerException: parent!!");
        ast.add_child(parent, entry, Some(n));
        let indent = ast.indent(n);
        let white_space = ast.new_leaf(WHITE_SPACE, &indent);
        ast.add_child(parent, white_space, Some(n));
    }
}

fn annotation_text(
    prefix: &str,
    suppress_type: SuppressAnnotationType,
    suppressions: &BTreeSet<String>,
) -> String {
    let sorted_suppressions: Vec<&str> = suppressions.iter().map(String::as_str).collect();
    format!(
        "{prefix}{}({})",
        suppress_type.annotation_name(),
        sorted_suppressions.join(", ")
    )
}

fn create_file_annotation(
    ast: &mut Ast,
    suppress_type: SuppressAnnotationType,
    suppressions: &BTreeSet<String>,
) -> NodeId {
    let annotation = annotation_text("@file:", suppress_type, suppressions);
    create_psi_file_first_child_from_text(ast, &annotation).unwrap_or_else(|| {
        panic!("IllegalStateException: Can not create annotation '{annotation}'")
    })
}

/// Always into the root, before its package directive.
fn create_file_annotation_list(ast: &mut Ast, n: NodeId, annotation: NodeId) {
    assert!(
        is_root(ast, n),
        "IllegalArgumentException: File annotation list can only be created for root node"
    );
    if let Some(package_directive) = ast.find_child_by_type(n, PACKAGE_DIRECTIVE) {
        if let Some(parent) = ast.parent(package_directive) {
            ast.add_child(parent, annotation, Some(package_directive));
        }
        if let Some(parent) = ast.parent(package_directive) {
            let white_space = ast.new_leaf(WHITE_SPACE, &format!("\n{}", ast.indent(n)));
            ast.add_child(parent, white_space, Some(package_directive));
        }
    }
}

/// The `MODIFIER_LIST` of `fun foo() {}` annotated with the suppression (the snippet must be valid code).
fn create_modifier_list_with_annotation_entry(
    ast: &mut Ast,
    suppress_type: SuppressAnnotationType,
    suppressions: &BTreeSet<String>,
) -> NodeId {
    let annotation = annotation_text("@", suppress_type, suppressions);
    let text = trim_indent(&format!(
        "\n{SOURCE_INDENT}{annotation}\n{SOURCE_INDENT}fun foo() {{}}\n{SOURCE_INDENT}"
    ));
    ast.create_ast_node_from_text(&text)
        .and_then(|it| ast.find_child_by_type(it, FUN))
        .and_then(|it| ast.find_child_by_type(it, MODIFIER_LIST))
        .unwrap_or_else(|| {
            panic!("IllegalStateException: Can not create annotation '{annotation}'")
        })
}

fn create_annotated_expression(
    ast: &mut Ast,
    n: NodeId,
    suppress_type: SuppressAnnotationType,
    suppressions: &BTreeSet<String>,
) -> NodeId {
    let annotation = annotation_text("@", suppress_type, suppressions);
    let indent = indent_without_newline_prefix(ast, n);
    let text = trim_margin(&format!(
        "\n{SOURCE_INDENT}|{indent}{annotation}\n{SOURCE_INDENT}|{indent}{}\n{SOURCE_INDENT}",
        ast.text(n)
    ));
    ast.create_ast_node_from_text(&text)
        .and_then(|it| ast.find_child_by_type(it, ANNOTATED_EXPRESSION))
        .unwrap_or_else(|| {
            panic!("IllegalStateException: Can not create annotation '{annotation}'")
        })
}
