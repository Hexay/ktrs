//! Port of ktlint-ruleset-standard `TrailingCommaOnDeclarationSiteRule.kt`, split in upstream order: this file (the rule
//! up to `findNodeAfterLastEnumEntry`) and `report.rs` (`reportAndCorrectTrailingCommaNodeBefore` to the end).

mod report;

use std::sync::LazyLock;

use ktrs_ast::psi::{KtWhenEntry, KtWhenExpression};
use ktrs_ast::{Ast, NodeId};
use ktrs_editorconfig::PropertyType;
use ktrs_editorconfig::property_type::boolean_value_parser;
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{
    ARROW, CLASS, CLASS_BODY, DESTRUCTURING_DECLARATION, ENUM_ENTRY, ENUM_KEYWORD, FUNCTION_LITERAL, FUNCTION_TYPE, GT, RBRACE,
    RPAR, SEMICOLON, TYPE_PARAMETER_LIST, VALUE_PARAMETER_LIST, WHEN_ENTRY,
};

use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::{EditorConfigProperty, PropertyRef};
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

use report::{closing_element_destructuring_declaration_entries, report_and_correct_trailing_comma_node_before};

static TRAILING_COMMA_ON_DECLARATION_SITE_PROPERTY_TYPE: PropertyType<bool> = PropertyType {
    name: "ij_kotlin_allow_trailing_comma",
    description: "Defines whether a trailing comma (or no trailing comma) should be enforced on the defining \
                  side, e.g. parameter-list, type-argument-list, lambda-value-parameters, enum-entries, etc.\
                  When set, IntelliJ IDEA uses this property to allow usage of a trailing comma by discretion \
                  of the developer. KtLint however uses this setting to enforce consistent usage of the \
                  trailing comma when set.",
    parser: boolean_value_parser,
    possible_values: &["true", "false"],
    lower_casing: true,
};

pub static TRAILING_COMMA_ON_DECLARATION_SITE_PROPERTY: LazyLock<EditorConfigProperty<bool>> = LazyLock::new(|| EditorConfigProperty {
    android_studio_code_style_default_value: false,
    ..EditorConfigProperty::new(&TRAILING_COMMA_ON_DECLARATION_SITE_PROPERTY_TYPE, true)
});

const TYPES_ON_DECLARATION_SITE: TokenSet = TokenSet::create(&[
    CLASS,
    DESTRUCTURING_DECLARATION,
    FUNCTION_LITERAL,
    FUNCTION_TYPE,
    TYPE_PARAMETER_LIST,
    VALUE_PARAMETER_LIST,
    WHEN_ENTRY,
]);

const NO_MATCHING_ELEMENT: &str = "NoSuchElementException: Sequence contains no element matching the predicate.";

/// Linting trailing comma for declaration site (https://kotlinlang.org/docs/coding-conventions.html#trailing-commas).
pub struct TrailingCommaOnDeclarationSiteRule {
    allow_trailing_comma: bool,
}

impl TrailingCommaOnDeclarationSiteRule {
    pub fn new() -> TrailingCommaOnDeclarationSiteRule {
        TrailingCommaOnDeclarationSiteRule { allow_trailing_comma: TRAILING_COMMA_ON_DECLARATION_SITE_PROPERTY.default_value }
    }
}

impl Default for TrailingCommaOnDeclarationSiteRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for TrailingCommaOnDeclarationSiteRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:trailing-comma-on-declaration-site")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*TRAILING_COMMA_ON_DECLARATION_SITE_PROPERTY)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.allow_trailing_comma = editor_config.get(&TRAILING_COMMA_ON_DECLARATION_SITE_PROPERTY);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        // Keep processing of element types in sync with Intellij Kotlin formatting settings.
        // https://github.com/JetBrains/intellij-kotlin/blob/master/formatter/src/org/jetbrains/kotlin/idea/formatter/trailingComma/util.kt
        match ast.element_type(node) {
            CLASS => self.visit_class(ast, node, emit),
            DESTRUCTURING_DECLARATION => self.visit_destructuring_declaration(ast, node, emit),
            FUNCTION_LITERAL => self.visit_function_literal(ast, node, emit),
            TYPE_PARAMETER_LIST => self.visit_type_list(ast, node, emit),
            VALUE_PARAMETER_LIST => self.visit_value_list(ast, node, emit),
            WHEN_ENTRY => self.visit_when_entry(ast, node, emit),
            _ => {}
        }
    }
}

impl TrailingCommaOnDeclarationSiteRule {
    fn visit_destructuring_declaration(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let inspect_node = closing_element_destructuring_declaration_entries(ast, node);
        report_and_correct_trailing_comma_node_before(ast, node, inspect_node, self.is_trailing_comma_allowed(ast, node), emit);
    }

    fn is_trailing_comma_allowed(&self, ast: &Ast, n: NodeId) -> bool {
        TYPES_ON_DECLARATION_SITE.contains(ast.element_type(n)) && self.allow_trailing_comma
    }

    fn visit_function_literal(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        // lambda w/o an arrow -> no arguments -> no commas
        let Some(inspect_node) = ast.children(node).filter(|&it| ast.element_type(it) == ARROW).last() else { return };
        report_and_correct_trailing_comma_node_before(ast, node, inspect_node, self.is_trailing_comma_allowed(ast, node), emit);
    }

    fn visit_value_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.parent(node).map(|it| ast.element_type(it)) != Some(FUNCTION_LITERAL)
            && let Some(inspect_node) = ast.children(node).filter(|&it| ast.element_type(it) == RPAR).last()
        {
            report_and_correct_trailing_comma_node_before(ast, node, inspect_node, self.is_trailing_comma_allowed(ast, node), emit);
        }
    }

    fn visit_type_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let inspect_node = ast.children(node).find(|&it| ast.element_type(it) == GT).expect(NO_MATCHING_ELEMENT);
        report_and_correct_trailing_comma_node_before(ast, node, inspect_node, self.is_trailing_comma_allowed(ast, node), emit);
    }

    fn visit_when_entry(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let psi = KtWhenEntry::cast(ast, node).expect("IllegalArgumentException: Failed requirement.");
        let when_expression = KtWhenExpression::of(ast, ast.parent(node).expect("NullPointerException: psi.parent"));
        if psi.is_else(ast) || when_expression.left_parenthesis(ast).is_none() {
            // no commas for "else" or when there are no opening parenthesis for the when-expression
            return;
        }

        let inspect_node = ast.children(node).find(|&it| ast.element_type(it) == ARROW).expect(NO_MATCHING_ELEMENT);
        report_and_correct_trailing_comma_node_before(ast, node, inspect_node, self.is_trailing_comma_allowed(ast, node), emit);
    }

    fn visit_class(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(node) == CLASS, "IllegalArgumentException: Failed requirement.");

        if !ast.has_modifier(node, ENUM_KEYWORD) {
            return;
        }
        let Some(class_body) = ast.find_child_by_type(node, CLASS_BODY).filter(|&it| !no_enum_entries(ast, it)) else { return };
        let Some(node_after_last_enum_entry) = find_node_after_last_enum_entry(ast, class_body) else { return };
        if !self.is_trailing_comma_allowed(ast, node) && ast.element_type(node_after_last_enum_entry) == RBRACE {
            report_and_correct_trailing_comma_node_before(ast, node, node_after_last_enum_entry, false, emit);
        } else if !last_two_enum_entries_are_on_same_line(ast, class_body) {
            report_and_correct_trailing_comma_node_before(
                ast,
                node,
                node_after_last_enum_entry,
                self.is_trailing_comma_allowed(ast, node),
                emit,
            );
        }
    }
}

fn no_enum_entries(ast: &Ast, n: NodeId) -> bool {
    !ast.children(n).any(|it| ast.element_type(it) == ENUM_ENTRY)
}

fn last_two_enum_entries_are_on_same_line(ast: &Ast, n: NodeId) -> bool {
    let enum_entries: Vec<NodeId> = ast.children(n).filter(|&it| ast.element_type(it) == ENUM_ENTRY).collect();
    let last_two_enum_entries = &enum_entries[enum_entries.len().saturating_sub(2)..];

    last_two_enum_entries.len() == 2 && ast.no_new_line_in_closed_range(last_two_enum_entries[0], last_two_enum_entries[1])
}

/// Determines the node before which the trailing comma is allowed: the semicolon terminating the list of enumeration entries
/// if any, otherwise the last element of the class.
fn find_node_after_last_enum_entry(ast: &Ast, n: NodeId) -> Option<NodeId> {
    let semicolon = ast.children(n).filter(|&it| ast.element_type(it) == ENUM_ENTRY).last().and_then(|entry| {
        let mut semicolons = ast.children(entry).filter(|&it| ast.element_type(it) == SEMICOLON);
        match (semicolons.next(), semicolons.next()) {
            (Some(single), None) => Some(single),
            _ => None,
        }
    });
    semicolon.or_else(|| ast.last_child_node(n))
}
