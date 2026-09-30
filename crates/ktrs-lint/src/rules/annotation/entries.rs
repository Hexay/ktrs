//! `AnnotationRule.kt` from `visitAnnotationEntry` to the end.

use ktrs_ast::psi::AnnotationUseSiteTarget;
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATION, ANNOTATION_ENTRY, ANNOTATION_TARGET, CLASS, COLON, CONSTRUCTOR_CALLEE, IDENTIFIER, REFERENCE_EXPRESSION, RPAR,
    TYPE_REFERENCE, USER_TYPE, VALUE_ARGUMENT_LIST,
};

use super::{AnnotationRule, FAILED_REQUIREMENT};
use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::Emit;

/// `AnnotationUseSiteTarget.entries`, for `USE_SITE_TARGETS` (by `renderName`).
const USE_SITE_TARGETS: [AnnotationUseSiteTarget; 10] = [
    AnnotationUseSiteTarget::All,
    AnnotationUseSiteTarget::Field,
    AnnotationUseSiteTarget::File,
    AnnotationUseSiteTarget::Property,
    AnnotationUseSiteTarget::PropertyGetter,
    AnnotationUseSiteTarget::PropertySetter,
    AnnotationUseSiteTarget::Receiver,
    AnnotationUseSiteTarget::ConstructorParameter,
    AnnotationUseSiteTarget::SetterParameter,
    AnnotationUseSiteTarget::PropertyDelegateField,
];

pub(super) fn visit_annotation_entry(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    assert!(ast.element_type(node) == ANNOTATION_ENTRY, "{FAILED_REQUIREMENT}");
    if is_preceded_by_other_annotation_entry_on_the_same_line(ast, node) && is_preceded_by_annotation_on_another_line(ast, node) {
        // Code below is disallowed
        //   @Foo1
        //   @Foo2 @Foo3
        //   fun foo() {}
        emit(ast, ast.start_offset(node), "All annotations should either be on a single line or all annotations should be on a separate line", true)
            .if_autocorrect_allowed(|| {
                let first_child_leaf = ast.first_child_leaf_or_self(node);
                let text = get_newline_with_indent(ast, ast.parent(node).expect("NullPointerException: parent!!"));
                ast.upsert_whitespace_before_me(first_child_leaf, &text);
            });
    }
}

fn is_preceded_by_annotation_on_another_line(ast: &Ast, n: NodeId) -> bool {
    let first_annotation = ast.parent(n).and_then(|it| ast.find_child_by_type(it, ANNOTATION_ENTRY));
    ast.siblings(n, false).take_while(|&it| Some(it) != first_annotation).any(|it| ast.is_white_space_with_newline(it))
}

pub(super) fn is_not_receiver_target_annotation(ast: &Ast, n: NodeId) -> bool {
    get_annotation_use_site_target(ast, n) != Some(AnnotationUseSiteTarget::Receiver)
}

fn get_annotation_use_site_target(ast: &Ast, n: NodeId) -> Option<AnnotationUseSiteTarget> {
    Some(n)
        .filter(|&it| ast.element_type(it) == ANNOTATION_ENTRY)
        .and_then(|it| ast.find_child_by_type(it, ANNOTATION_TARGET))
        .and_then(|it| {
            let text = ast.text(it);
            USE_SITE_TARGETS.into_iter().find(|target| target.render_name() == text)
        })
}

impl AnnotationRule {
    pub(super) fn is_annotation_entry_with_value_argument_list_that_should_be_wrapped(&self, ast: &Ast, n: NodeId) -> bool {
        if self.handle_all_annotations_with_parameters_same_as_annotations_without_parameters() {
            // Do never distinct between annotation with and without parameters
            false
        } else if self.annotations_with_parameters_no_to_be_wrapped.is_empty() {
            // All annotations with parameters should be wrapped as the whitelist is empty
            true
        } else if ast.element_type(n) == ANNOTATION_ENTRY && ast.find_child_by_type(n, VALUE_ARGUMENT_LIST).is_some() {
            // Only wrap annotation with parameters when it is not on the whitelist
            let identifier = get_annotation_identifier(ast, n);
            !self.annotations_with_parameters_no_to_be_wrapped.iter().any(|it| Some(it) == identifier.as_ref())
        } else {
            // Annotation without parameter
            false
        }
    }
}

fn get_annotation_identifier(ast: &Ast, n: NodeId) -> Option<String> {
    ast.find_child_by_type(n, CONSTRUCTOR_CALLEE)
        .and_then(|it| ast.find_child_by_type(it, TYPE_REFERENCE))
        .and_then(|it| ast.find_child_by_type(it, USER_TYPE))
        .and_then(|it| ast.find_child_by_type(it, REFERENCE_EXPRESSION))
        .and_then(|it| ast.find_child_by_type(it, IDENTIFIER))
        .map(|it| ast.text(it))
}

pub(super) fn is_last_annotation_entry(ast: &Ast, n: NodeId) -> bool {
    ast.parent(n).and_then(|p| ast.children(p).filter(|&it| ast.element_type(it) == ANNOTATION_ENTRY).last()) == Some(n)
}

impl AnnotationRule {
    pub(super) fn is_preceded_by_other_annotation_entry_without_parameters_on_the_same_line(&self, ast: &Ast, n: NodeId) -> bool {
        ast.siblings(n, false)
            .take_while(|&it| {
                !ast.is_white_space_with_newline(it) && !self.is_annotation_entry_with_value_argument_list_that_should_be_wrapped(ast, it)
            })
            .any(|it| {
                ast.element_type(it) == ANNOTATION_ENTRY && !self.is_annotation_entry_with_value_argument_list_that_should_be_wrapped(ast, it)
            })
    }
}

pub(super) fn is_preceded_by_other_annotation_entry_on_the_same_line(ast: &Ast, n: NodeId) -> bool {
    ast.siblings(n, false).take_while(|&it| !ast.is_white_space_with_newline(it)).any(|it| ast.element_type(it) == ANNOTATION_ENTRY)
}

fn is_preceded_by_other_annotation_entry(ast: &Ast, n: NodeId) -> bool {
    ast.siblings(n, false).any(|it| ast.element_type(it) == ANNOTATION_ENTRY)
}

fn is_on_same_line_as_previous_annotation_entry(ast: &Ast, n: NodeId) -> bool {
    !ast.siblings(n, false).take_while(|&it| ast.element_type(it) != ANNOTATION_ENTRY).any(|it| ast.is_white_space_with_newline(it))
}

fn is_followed_by_other_annotation_entry(ast: &Ast, n: NodeId) -> bool {
    ast.siblings(n, true).any(|it| ast.element_type(it) == ANNOTATION_ENTRY)
}

/// Allow:
///     class Foo(
///         bar: Bar,
///     ) : @Suppress("DEPRECATION")
///         FooBar()
pub(super) fn annotation_on_same_line_as_closing_parenthesis_of_class_parameter_list(ast: &Ast, n: Option<NodeId>) -> bool {
    n.filter(|&it| ast.parent(it).map(|p| ast.element_type(p)) == Some(CLASS))
        .and_then(|it| ast.prev_code_sibling(it))
        .filter(|&it| ast.element_type(it) == COLON)
        .and_then(|it| ast.prev_code_leaf(it))
        .filter(|&it| ast.element_type(it) == RPAR)
        .and_then(|it| ast.prev_leaf(it))
        .is_some_and(|it| ast.is_white_space_with_newline(it))
}

fn is_on_same_line_as_next_annotation_entry(ast: &Ast, n: NodeId) -> bool {
    !ast.siblings(n, true).take_while(|&it| ast.element_type(it) != ANNOTATION_ENTRY).any(|it| ast.is_white_space_with_newline(it))
}

pub(super) fn visit_file_annotation_list(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let Some(code_leaf) = ast.next_code_leaf(ast.last_child_leaf_or_self(node)) else { return };
    let whitespace_before = ast.prev_leaf_matching(code_leaf, |it| ast.is_white_space(it));

    if whitespace_before.is_none_or(|it| ast.leaf_text(it) != "\n\n") {
        emit(ast, ast.start_offset(code_leaf), "File annotations should be separated from file contents with a blank line", true)
            .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(code_leaf, "\n\n"));
    }
}

pub(super) fn visit_annotation(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    assert!(ast.element_type(node) == ANNOTATION, "{FAILED_REQUIREMENT}");

    if (is_followed_by_other_annotation_entry(ast, node) && is_on_same_line_as_next_annotation_entry(ast, node))
        || (is_preceded_by_other_annotation_entry(ast, node) && is_on_same_line_as_previous_annotation_entry(ast, node))
    {
        emit(ast, ast.start_offset(node), "@[...] style annotations should be on a separate line from other annotations.", true)
            .if_autocorrect_allowed(|| {
                if is_followed_by_other_annotation_entry(ast, node) {
                    let text = get_newline_with_indent(ast, ast.parent(node).expect("NullPointerException: parent!!"));
                    ast.upsert_whitespace_after_me(node, &text);
                } else if is_preceded_by_other_annotation_entry(ast, node) {
                    let text = get_newline_with_indent(ast, ast.parent(node).expect("NullPointerException: parent!!"));
                    ast.upsert_whitespace_before_me(node, &text);
                }
            });
    }
}

fn get_newline_with_indent(ast: &Ast, modifier_list_root: NodeId) -> String {
    let node_before_annotations = ast.parent(modifier_list_root).and_then(|it| ast.prev_sibling(it));
    // Make sure we only insert a single newline
    let text = node_before_annotations.map(|it| ast.text(it)).unwrap_or_default();
    let indent_without_newline = text.rfind('\n').map_or(text.as_str(), |i| &text[i + 1..]);
    format!("\n{indent_without_newline}")
}
