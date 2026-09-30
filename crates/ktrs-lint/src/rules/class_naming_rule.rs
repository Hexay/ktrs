//! Port of ktlint-ruleset-standard `ClassNamingRule.kt` (https://kotlinlang.org/docs/coding-conventions.html#naming-rules).
//! Backticked class names are allowed in test files, consistent with test function names.

use std::sync::LazyLock;

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CLASS, DOT_QUALIFIED_EXPRESSION, IDENTIFIER, IMPORT_DIRECTIVE, OBJECT_DECLARATION};

use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::kotlin_string::remove_surrounding;
use crate::rules::internal::{KotlinRegex, is_keyword, reg_ex_ignoring_diacritics_and_strokes_on_letters};

#[derive(Default)]
pub struct ClassNamingRule {
    allow_backticked_class_name: bool,
}

impl RuleV2 for ClassNamingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:class-naming")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if !self.allow_backticked_class_name
            && ast.element_type(node) == IMPORT_DIRECTIVE
            && ast
                .find_child_by_type(node, DOT_QUALIFIED_EXPRESSION)
                .is_some_and(|it| ast.text(it).starts_with("org.junit.jupiter.api"))
        {
            // Assume that each file that imports a Junit Jupiter Api class is a test class
            self.allow_backticked_class_name = true;
        }

        if !matches!(ast.element_type(node), CLASS | OBJECT_DECLARATION) {
            return;
        }
        if let Some(it) = ast.find_child_by_type(node, IDENTIFIER).filter(|&it| {
            !(is_valid_function_name(ast, it) || self.is_test_class(ast, it) || is_token_keyword_between_backticks(ast, it))
        }) {
            emit(ast, ast.start_offset(it), "Class or object name should start with an uppercase letter and use camel case", false);
        }
    }
}

impl ClassNamingRule {
    fn is_test_class(&self, ast: &Ast, node: NodeId) -> bool {
        self.allow_backticked_class_name && has_back_ticked_identifier(ast, node)
    }
}

fn is_valid_function_name(ast: &Ast, node: NodeId) -> bool {
    VALID_CLASS_NAME_REGEXP.matches(ast.leaf_text(node))
}

fn has_back_ticked_identifier(ast: &Ast, node: NodeId) -> bool {
    BACK_TICKED_FUNCTION_NAME_REGEXP.matches(ast.leaf_text(node))
}

fn is_token_keyword_between_backticks(ast: &Ast, node: NodeId) -> bool {
    let text = if ast.element_type(node) == IDENTIFIER { ast.leaf_text(node) } else { "" };
    is_keyword(remove_surrounding(text, "`", "`"))
}

static VALID_CLASS_NAME_REGEXP: LazyLock<KotlinRegex> =
    LazyLock::new(|| reg_ex_ignoring_diacritics_and_strokes_on_letters("[A-Z][A-Za-z\\d]*"));
static BACK_TICKED_FUNCTION_NAME_REGEXP: LazyLock<KotlinRegex> = LazyLock::new(|| KotlinRegex::new("`.*`"));
