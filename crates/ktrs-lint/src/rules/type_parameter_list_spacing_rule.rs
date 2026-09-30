//! Port of ktlint-ruleset-standard `TypeParameterListSpacingRule.kt` (id `type-parameter-list-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    CLASS, CLASS_BODY, CONSTRUCTOR_KEYWORD, EQ, FUN, GT, LT, PRIMARY_CONSTRUCTOR, TYPEALIAS, TYPE_PARAMETER_LIST,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

/// Lints and formats the spacing before and after the angle brackets of a type parameter list.
pub struct TypeParameterListSpacingRule {
    indent_config: IndentConfig,
}

impl TypeParameterListSpacingRule {
    pub fn new() -> TypeParameterListSpacingRule {
        TypeParameterListSpacingRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for TypeParameterListSpacingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for TypeParameterListSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:type-parameter-list-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*INDENT_SIZE_PROPERTY), PropertyRef::from(&*INDENT_STYLE_PROPERTY)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != TYPE_PARAMETER_LIST {
            return;
        }
        match ast.parent(node).map(|p| ast.element_type(p)) {
            Some(CLASS) => visit_class_declaration(ast, node, emit),
            Some(TYPEALIAS) => visit_type_alias_declaration(ast, node, emit),
            Some(FUN) => visit_function_declaration(ast, node, emit),
            _ => {}
        }
        self.visit_inside_type_parameter_list(ast, node, emit);
    }
}

fn next_code_sibling_type(ast: &Ast, node: NodeId) -> Option<ktrs_syntax::SyntaxKind> {
    ast.next_code_sibling(node).map(|it| ast.element_type(it))
}

fn visit_class_declaration(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    // No white space expected between class name and parameter list
    //     class Bar <T>
    if let Some(it) = ast.prev_sibling(node).filter(|&it| ast.is_white_space(it)) {
        visit_whitespace(ast, it, emit, "");
    }

    // No white space expected between parameter type list and the constructor except when followed by compound
    // constructor
    //     class Bar<T> (...)
    if let Some(white_space) = ast
        .next_sibling(node)
        .filter(|&it| ast.is_white_space(it) && next_code_sibling_type(ast, it) == Some(PRIMARY_CONSTRUCTOR))
    {
        if ast.next_code_sibling(white_space).and_then(|it| ast.find_child_by_type(it, CONSTRUCTOR_KEYWORD)).is_some() {
            // A newline is acceptable before (the modifier list of) the constructor; on the same line, a single space:
            //    class Bar<T> constructor(...)
            //    class Bar<T> actual constructor(...)
            //    class Bar<T> @SomeAnnotation constructor(...)
            if !ast.is_white_space_with_newline(white_space) && ast.leaf_text(white_space) != " " {
                emit(ast, ast.start_offset(white_space), "Expected a single space", true)
                    // If line is to be wrapped this should have been done by other rules before running this rule
                    .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(white_space, " "));
            }
        } else {
            visit_whitespace(ast, white_space, emit, "");
        }
    }

    // No white space expected between parameter type list and class body when constructor is missing
    //    class Bar<T> {
    if let Some(it) =
        ast.next_sibling(node).filter(|&it| ast.is_white_space(it) && next_code_sibling_type(ast, it) == Some(CLASS_BODY))
    {
        single_space_expected(ast, it, emit);
    }
}

fn visit_type_alias_declaration(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    // No white space expected between typealias keyword name and parameter list
    //     typealias Bar <T>
    if let Some(it) = ast.prev_sibling(node).filter(|&it| ast.is_white_space(it)) {
        visit_whitespace(ast, it, emit, "");
    }

    // No white space expected between parameter type list and equals sign
    //    typealias Bar<T> = ...
    if let Some(it) = ast.next_sibling(node).filter(|&it| ast.is_white_space(it) && next_code_sibling_type(ast, it) == Some(EQ)) {
        single_space_expected(ast, it, emit);
    }
}

fn visit_function_declaration(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    // Single space expected before type parameter list of function
    //    fun<T> foo(...)
    if let Some(prev_leaf) = ast.prev_leaf(node) {
        if ast.is_white_space(prev_leaf) {
            single_space_expected(ast, prev_leaf, emit);
        } else {
            let first_child_node = ast.first_child_node(node).expect("NullPointerException: firstChildNode");
            single_space_expected(ast, first_child_node, emit);
        }
    }

    // Single space expected after type parameter list of function
    //   fun <T>foo(...)
    //   fun <T>List<T>foo(...)
    let last_child_node = ast.last_child_node(node).expect("NullPointerException: lastChildNode");
    if let Some(next_sibling) = ast.next_leaf(last_child_node) {
        single_space_expected(ast, next_sibling, emit);
    }
}

impl TypeParameterListSpacingRule {
    fn visit_inside_type_parameter_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if let Some(it) = ast.find_child_by_type(node, LT).and_then(|lt| ast.next_sibling(lt)).filter(|&it| ast.is_white_space(it)) {
            let expected_whitespace = self.expected_whitespace(ast, node);
            visit_whitespace(ast, it, emit, &expected_whitespace);
        }

        if let Some(it) = ast.find_child_by_type(node, GT).and_then(|gt| ast.prev_sibling(gt)).filter(|&it| ast.is_white_space(it)) {
            let expected_whitespace = self.expected_whitespace(ast, node);
            visit_whitespace(ast, it, emit, &expected_whitespace);
        }
    }

    fn expected_whitespace(&self, ast: &Ast, node: NodeId) -> String {
        if ast.text_contains(node, '\n') { self.indent_config.child_indent_of(ast, node) } else { String::new() }
    }
}

fn visit_whitespace(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>, expected_whitespace: &str) {
    if ast.text_matches(node, expected_whitespace) {
        return;
    }
    if expected_whitespace.is_empty() {
        emit(ast, ast.start_offset(node), "No whitespace expected", true).if_autocorrect_allowed(|| ast.remove(node));
    } else if ast.is_white_space_without_newline(node) && expected_whitespace.starts_with('\n') {
        emit(ast, ast.start_offset(node), "Expected a newline", true)
            .if_autocorrect_allowed(|| ast.replace_text_with(node, expected_whitespace));
    } else if expected_whitespace == " " {
        emit(ast, ast.start_offset(node), "Expected a single space", true)
            .if_autocorrect_allowed(|| ast.replace_text_with(node, expected_whitespace));
    }
}

fn single_space_expected(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    if ast.text_matches(node, " ") {
        // Nothing to do
    } else if ast.is_white_space_with_newline(node) {
        emit(ast, ast.start_offset(node), "Expected a single space instead of newline", true)
            .if_autocorrect_allowed(|| ast.replace_text_with(node, " "));
    } else {
        emit(ast, ast.start_offset(node), "Expected a single space", true)
            .if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(node, " "));
    }
}
