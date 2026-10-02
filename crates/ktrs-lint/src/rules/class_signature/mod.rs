//! Port of ktlint-ruleset-standard `ClassSignatureRule.kt`, in upstream order over `mod.rs` (up to `isAnnotated`),
//! `parameter_list.rs`, `super_types.rs` (up to `getPrimaryConstructorParameterListOrNull`) and `properties.rs` (the
//! companion object). `collectLeavesRecursively`, `childrenBetween` and `joinTextToString` are shared with
//! `FunctionSignatureRule`.

mod parameter_list;
mod properties;
mod super_types;

pub use properties::FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY;

use ktrs_ast::{Ast, NodeId};

use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{
    CODE_STYLE_PROPERTY, CodeStyleValue, EditorConfig, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, KtlintVersion, MAX_LINE_LENGTH_PROPERTY,
    PropertyRef,
};
use crate::element_type::{
    ANNOTATION, ANNOTATION_ENTRY, CLASS, CLASS_BODY, COLON, COMMA, EOL_COMMENT, MODIFIER_LIST, SUPER_TYPE_LIST, VALUE_PARAMETER,
    WHITE_SPACE,
};
use crate::indent_config::IndentConfig;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::function_signature::{children_between, collect_leaves_recursively, join_text_length};
use crate::rules::max_line_length_rule::max_line_length;

use super_types::{fix_class_body, get_primary_constructor_parameter_list_or_null};

/// Formats the class signature according to <https://kotlinlang.org/docs/coding-conventions.html#class-headers>.
///
/// In code style `ktlint_official` class headers containing 2 or more parameters are formatted as multiline signature. As the
/// Kotlin Coding conventions do not specify what is meant with a "few parameters", no default is set for other code styles.
pub struct ClassSignatureRule {
    code_style: CodeStyleValue,
    indent_config: IndentConfig,
    max_line_length: i32,
    class_signature_wrapping_minimum_parameters: i32,
    ktlint_version: KtlintVersion,
}

impl ClassSignatureRule {
    pub fn new() -> ClassSignatureRule {
        ClassSignatureRule {
            code_style: CODE_STYLE_PROPERTY.default_value,
            indent_config: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
            class_signature_wrapping_minimum_parameters: FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY.default_value,
            ktlint_version: KtlintVersion::default(),
        }
    }
}

impl Default for ClassSignatureRule {
    fn default() -> Self {
        Self::new()
    }
}

const VISITED_TYPES: TokenSet = TokenSet::create(&[CLASS]);

impl RuleV2 for ClassSignatureRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:class-signature")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY),
            PropertyRef::from(&*FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY),
        ]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.code_style = editor_config.get(&CODE_STYLE_PROPERTY);
        self.class_signature_wrapping_minimum_parameters =
            editor_config.get(&FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY);
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        self.max_line_length = max_line_length(editor_config);
        self.ktlint_version = KtlintVersion::of(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == CLASS {
            self.visit_class(ast, node, emit);
        }
    }
}

fn class_signature_nodes(ast: &Ast, node: NodeId, exclude_super_types: bool) -> Vec<NodeId> {
    // Find the nodes that are to be placed on the same line if no max line length is set
    //     internal class Foo(bar: String) { // Class without super type
    // or
    //     public class Foo(bar: String) : Bar { // Class with exactly one super type
    // or
    //     private class Foo(bar: String) : // Class with multiple super types which have to be wrapped to next lines
    let first_code_child = get_first_child_in_signature(ast, node);
    let last_node_in_primary_class_signature_line = Some(node)
        .filter(|_| exclude_super_types)
        // When the class extends multiple super types or if the super type list contains a newline, all super types are wrapped
        // on separate line
        .and_then(|it| ast.find_child_by_type(it, COLON))
        .or_else(|| ast.find_child_by_type(node, CLASS_BODY).and_then(|it| ast.first_child_node(it)));
    children_between(
        collect_leaves_recursively(ast, node),
        |it| Some(it) == first_code_child,
        |it| Some(it) == last_node_in_primary_class_signature_line,
    )
}

/// The lazy `Sequence` upstream returns: seeded with the list's first child when created, then re-walked over the live
/// `nextSibling` chain at every use, so after the reorder in `fixWhitespacesInSuperTypeList` it misses the moved call entry.
struct SuperTypes(Option<NodeId>);

impl SuperTypes {
    fn list(&self, ast: &Ast) -> Vec<NodeId> {
        std::iter::successors(self.0, |&it| ast.next_sibling(it)).filter(|&it| ast.is_code(it) && ast.element_type(it) != COMMA).collect()
    }
}

fn super_types(ast: &Ast, node: NodeId) -> Option<SuperTypes> {
    ast.find_child_by_type(node, SUPER_TYPE_LIST).map(|list| SuperTypes(ast.first_child_node(list)))
}

fn has_multiline_super_type_list(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, SUPER_TYPE_LIST).is_some_and(|it| ast.text_contains(it, '\n'))
}

fn get_first_child_in_signature(ast: &Ast, node: NodeId) -> Option<NodeId> {
    if let Some(modifier_list) = ast.find_child_by_type(node, MODIFIER_LIST) {
        for current_node in ast.children(modifier_list) {
            if !matches!(ast.element_type(current_node), ANNOTATION | ANNOTATION_ENTRY | WHITE_SPACE | EOL_COMMENT) {
                return Some(current_node);
            }
        }
        return ast.next_code_sibling(modifier_list);
    }
    ast.next_code_leaf(node)
}

impl ClassSignatureRule {
    fn visit_class(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(node) == CLASS, "IllegalArgumentException: Failed requirement.");

        let wrap_primary_constructor_parameters = self.has_too_many_parameters(ast, node)
            || contains_multiline_parameter(ast, node)
            || (self.code_style == CodeStyleValue::KtlintOfficial && contains_annotated_parameter(ast, node))
            || (self.is_max_line_length_set()
                && ast.has_no_max_line_length_suppression_in(node, self.ktlint_version)
                && self.class_signature_excluding_super_types_exceeds_max_line_length(ast, node, emit))
            || (!self.is_max_line_length_set() && class_signature_excluding_super_types_is_multiline(ast, node))
            || contains_eol_comment(ast, node);
        self.fix_white_spaces_in_value_parameter_list(ast, node, emit, wrap_primary_constructor_parameters, false);
        self.fix_whitespaces_in_super_type_list(ast, node, emit, wrap_primary_constructor_parameters);
        fix_class_body(ast, node, emit);
    }
}

fn contains_eol_comment(ast: &Ast, node: NodeId) -> bool {
    get_primary_constructor_parameter_list_or_null(ast, node).is_some_and(|list| ast.children(list).any(|it| ast.element_type(it) == EOL_COMMENT))
}

impl ClassSignatureRule {
    fn class_signature_excluding_super_types_exceeds_max_line_length(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) -> bool {
        let actual_class_signature_length = get_class_signature_length(ast, node, true);
        // Calculate the length of the class signature in case it, excluding the super types, would be rewritten as single
        // line (and without a maximum line length). The white space correction will be calculated via a dry run of the
        // actual fix.
        let length = actual_class_signature_length
            // Calculate the white space correction in case the signature would be rewritten to a single line
            + self.fix_white_spaces_in_value_parameter_list(ast, node, emit, false, true);
        ast.has_no_max_line_length_suppression_in(node, self.ktlint_version) && length > self.max_line_length
    }
}

fn class_signature_excluding_super_types_is_multiline(ast: &Ast, node: NodeId) -> bool {
    class_signature_nodes(ast, node, true).iter().any(|&it| ast.is_white_space_with_newline(it))
}

fn get_class_signature_length(ast: &Ast, node: NodeId, exclude_super_types: bool) -> i32 {
    ast.indent_without_newline_prefix(node).encode_utf16().count() as i32 + get_class_signature_nodes_length(ast, node, exclude_super_types)
}

fn get_class_signature_nodes_length(ast: &Ast, node: NodeId, exclude_super_types: bool) -> i32 {
    join_text_length(ast, &class_signature_nodes(ast, node, exclude_super_types))
}

fn contains_multiline_parameter(ast: &Ast, node: NodeId) -> bool {
    get_primary_constructor_parameter_list_or_null(ast, node)
        .is_some_and(|list| ast.children(list).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).any(|it| ast.text_contains(it, '\n')))
}

fn contains_annotated_parameter(ast: &Ast, node: NodeId) -> bool {
    get_primary_constructor_parameter_list_or_null(ast, node)
        .is_some_and(|list| ast.children(list).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).any(|it| is_annotated(ast, it)))
}

fn is_annotated(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, MODIFIER_LIST).is_some_and(|list| ast.children(list).any(|it| ast.element_type(it) == ANNOTATION_ENTRY))
}
