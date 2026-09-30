//! Port of ktlint-ruleset-standard `FunctionNamingRule.kt` (https://kotlinlang.org/docs/coding-conventions.html#function-names).

use std::sync::LazyLock;

use ktrs_ast::psi::{KtFunction, KtImportDirective};
use ktrs_ast::{Ast, NodeId};
use ktrs_editorconfig::PropertyType;
use ktrs_syntax::SyntaxKind::{
    ANNOTATION, ANNOTATION_ENTRY, CALL_EXPRESSION, CONSTRUCTOR_CALLEE, FUN, FUN_KEYWORD, IDENTIFIER, IMPORT_DIRECTIVE,
    MODIFIER_LIST, OVERRIDE_KEYWORD, REFERENCE_EXPRESSION, TYPE_REFERENCE, USER_TYPE, VALUE_PARAMETER_LIST,
};

use crate::ast_node_extension::AstNodeExtension;
use crate::editorconfig::{EditorConfigProperty, PropertyRef, comma_separated_list_value_parser};
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::kotlin_string::remove_surrounding;
use crate::rules::internal::{KotlinRegex, is_keyword, reg_ex_ignoring_diacritics_and_strokes_on_letters};

pub struct FunctionNamingRule {
    is_test_class: bool,
    ignore_when_annotated_with: Vec<String>,
}

impl FunctionNamingRule {
    pub fn new() -> FunctionNamingRule {
        FunctionNamingRule { is_test_class: false, ignore_when_annotated_with: IGNORE_WHEN_ANNOTATED_WITH_PROPERTY.default_value.clone() }
    }
}

impl Default for FunctionNamingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for FunctionNamingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:function-naming")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*IGNORE_WHEN_ANNOTATED_WITH_PROPERTY)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.ignore_when_annotated_with = editor_config.get(&IGNORE_WHEN_ANNOTATED_WITH_PROPERTY);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if !self.is_test_class
            && ast.element_type(node) == IMPORT_DIRECTIVE
            && KtImportDirective::of(ast, node)
                .import_path(ast)
                .is_some_and(|path| TEST_LIBRARIES_SET.iter().any(|it| path.path_str().starts_with(it)))
        {
            self.is_test_class = true;
        }

        if ast.element_type(node) != FUN {
            return;
        }
        if is_factory_method(ast, node)
            || self.is_method_in_test_class(ast, node)
            || has_valid_function_name(ast, node)
            || is_anonymous_function(ast, node)
            || is_override_function(ast, node)
            || is_annotated_with_any_of(ast, node, &self.ignore_when_annotated_with)
        {
            return;
        }
        if let Some(identifier) = ast.find_child_by_type(node, IDENTIFIER).filter(|&it| !is_token_keyword_between_backticks(ast, it)) {
            emit(
                ast,
                ast.start_offset(identifier),
                "Function name should start with a lowercase letter (except factory methods) and use camel case",
                false,
            );
        }
    }
}

impl FunctionNamingRule {
    fn is_method_in_test_class(&self, ast: &Ast, node: NodeId) -> bool {
        self.is_test_class && has_valid_test_function_name(ast, node)
    }
}

fn is_factory_method(ast: &Ast, node: NodeId) -> bool {
    let kt_function = KtFunction::of(ast, node);
    if kt_function.has_declared_return_type(ast) {
        // Allow:
        //     fun Foo(): Foo = ..
        //     fun <T> Foo(action: () -> T): Foo<T> = ..
        name_equals_text_of(ast, kt_function, type_reference_name_without_generics(ast, kt_function))
    } else {
        // Allow factory methods to overload another factory method or class constructor without specifying the type like:
        //     fun Foo(value: Bar) = Foo(value.baz())
        name_equals_text_of(ast, kt_function, call_expression_reference_identifier(ast, kt_function))
    }
}

/// `ktFunction.name == other?.text`, without building either string (this runs on every function).
fn name_equals_text_of(ast: &Ast, kt_function: KtFunction, other: Option<NodeId>) -> bool {
    let name = kt_function.name_identifier(ast).map(|it| {
        let quoted = ast.leaf_text(it);
        if quoted.len() >= 2 && quoted.starts_with('`') && quoted.ends_with('`') { &quoted[1..quoted.len() - 1] } else { quoted }
    });
    match (name, other) {
        (None, None) => true,
        (Some(name), Some(other)) => ast.text_matches(other, name),
        _ => false,
    }
}

/// The node whose text upstream returns.
fn type_reference_name_without_generics(ast: &Ast, kt_function: KtFunction) -> Option<NodeId> {
    kt_function
        .type_reference(ast)
        .and_then(|it| ast.find_child_by_type(it.node(), USER_TYPE))
        .and_then(|it| ast.find_child_by_type(it, REFERENCE_EXPRESSION))
}

/// The node whose text upstream returns.
fn call_expression_reference_identifier(ast: &Ast, kt_function: KtFunction) -> Option<NodeId> {
    kt_function
        .body_expression(ast)
        .filter(|&it| ast.element_type(it) == CALL_EXPRESSION)
        .and_then(|it| ast.find_child_by_type(it, REFERENCE_EXPRESSION))
        .and_then(|it| ast.find_child_by_type(it, IDENTIFIER))
}

fn identifier_text(ast: &Ast, node: NodeId) -> &str {
    ast.find_child_by_type(node, IDENTIFIER).map_or("", |it| ast.leaf_text(it))
}

fn has_valid_test_function_name(ast: &Ast, node: NodeId) -> bool {
    VALID_TEST_FUNCTION_NAME_REGEXP.matches(identifier_text(ast, node))
}

fn has_valid_function_name(ast: &Ast, node: NodeId) -> bool {
    VALID_FUNCTION_NAME_REGEXP.matches(identifier_text(ast, node))
}

fn is_anonymous_function(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, FUN_KEYWORD).and_then(|it| ast.next_code_sibling(it)).map(|it| ast.element_type(it))
        == Some(VALUE_PARAMETER_LIST)
}

/// An override is not reported: its name may be defined out of the project's scope (the definition itself is
/// reported where it is in scope).
fn is_override_function(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, MODIFIER_LIST)
        .is_some_and(|list| AstNodeExtension::children(ast, list).any(|it| ast.element_type(it) == OVERRIDE_KEYWORD))
}

fn is_annotated_with_any_of(ast: &Ast, node: NodeId, exclude_when_annotated_with: &[String]) -> bool {
    contains_annotation_entry_with_identifier_in(ast, ast.find_child_by_type(node, MODIFIER_LIST), exclude_when_annotated_with)
}

fn contains_annotation_entry_with_identifier_in(ast: &Ast, this: Option<NodeId>, exclude_when_annotated_with: &[String]) -> bool {
    this.is_some_and(|this| {
        AstNodeExtension::children(ast, this).any(|it| match ast.element_type(it) {
            ANNOTATION => contains_annotation_entry_with_identifier_in(ast, Some(it), exclude_when_annotated_with),
            ANNOTATION_ENTRY => annotation_entry_name(ast, it).is_some_and(|name| exclude_when_annotated_with.contains(&name)),
            _ => false,
        })
    })
}

fn annotation_entry_name(ast: &Ast, node: NodeId) -> Option<String> {
    ast.find_child_by_type(node, CONSTRUCTOR_CALLEE)
        .and_then(|it| ast.find_child_by_type(it, TYPE_REFERENCE))
        .and_then(|it| ast.find_child_by_type(it, USER_TYPE))
        .and_then(|it| ast.find_child_by_type(it, REFERENCE_EXPRESSION))
        .and_then(|it| ast.find_child_by_type(it, IDENTIFIER))
        .map(|it| ast.text(it))
}

fn is_token_keyword_between_backticks(ast: &Ast, node: NodeId) -> bool {
    let text = if ast.element_type(node) == IDENTIFIER { ast.leaf_text(node) } else { "" };
    is_keyword(remove_surrounding(text, "`", "`"))
}

static IGNORE_WHEN_ANNOTATED_WITH_PROPERTY_TYPE: PropertyType<Vec<String>> = PropertyType {
    name: "ktlint_function_naming_ignore_when_annotated_with",
    description: "Ignore functions that are annotated with. Value is a comma separated list of name without the '@' prefix.",
    parser: comma_separated_list_value_parser,
    possible_values: &[],
    lower_casing: true,
};

pub static IGNORE_WHEN_ANNOTATED_WITH_PROPERTY: LazyLock<EditorConfigProperty<Vec<String>>> =
    LazyLock::new(|| EditorConfigProperty::new(&IGNORE_WHEN_ANNOTATED_WITH_PROPERTY_TYPE, vec!["unset".to_owned()]));

static VALID_FUNCTION_NAME_REGEXP: LazyLock<KotlinRegex> =
    LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("[a-z][A-Za-z\\d]*"));
static VALID_TEST_FUNCTION_NAME_REGEXP: LazyLock<KotlinRegex> =
    LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("(`.*`)|([a-z][A-Za-z\\d_]*)"));
const TEST_LIBRARIES_SET: &[&str] = &["io.kotest", "junit.framework", "kotlin.test", "org.junit", "org.testng"];
