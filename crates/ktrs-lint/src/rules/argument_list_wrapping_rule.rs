//! Port of ktlint-ruleset-standard `ArgumentListWrappingRule.kt` (id `argument-list-wrapping`).

use std::sync::LazyLock;

use ktrs_ast::{Ast, NodeId, psi};
use ktrs_editorconfig::PropertyType;
use ktrs_editorconfig::property_type::positive_int_value_parser;
use ktrs_syntax::SyntaxKind::{
    BINARY_EXPRESSION, COLLECTION_LITERAL_EXPRESSION, DOT_QUALIFIED_EXPRESSION, EQ, FUNCTION_LITERAL, LPAR, OPERATION_REFERENCE, RPAR,
    TYPE_ARGUMENT_LIST, VALUE_ARGUMENT, VALUE_ARGUMENT_LIST, WHITE_SPACE,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{EditorConfigProperty, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, KtlintVersion, MAX_LINE_LENGTH_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;
use crate::token_sets::CONTROL_FLOW_KEYWORDS;

const VISITED_TYPES: TokenSet = TokenSet::create(&[VALUE_ARGUMENT_LIST]);

/// https://kotlinlang.org/docs/reference/coding-conventions.html#method-call-formatting
///
/// More aggressive than the styleguide: each argument goes on a separate line if at least one of the arguments is, or
/// if the max line length is exceeded (and wrapping would help); "(" and ")" then go on separate lines too.
pub struct ArgumentListWrappingRule {
    editor_config_indent: IndentConfig,
    max_line_length: i32,
    ignore_when_parameter_count_greater_or_equal_than_property: i32,
    ktlint_version: KtlintVersion,
}

impl ArgumentListWrappingRule {
    pub fn new() -> ArgumentListWrappingRule {
        ArgumentListWrappingRule {
            editor_config_indent: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
            ignore_when_parameter_count_greater_or_equal_than_property: UNSET_IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY,
            ktlint_version: KtlintVersion::default(),
        }
    }
}

impl Default for ArgumentListWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for ArgumentListWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:argument-list-wrapping")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY),
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY),
        ]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.editor_config_indent =
            IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        self.max_line_length = max_line_length(editor_config);
        self.ignore_when_parameter_count_greater_or_equal_than_property =
            editor_config.get(&IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY);
        self.ktlint_version = KtlintVersion::of(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if self.editor_config_indent.disabled() {
            return;
        }

        if ast.element_type(node) == VALUE_ARGUMENT_LIST && self.need_to_wrap_argument_list(ast, node) {
            // A lazy walk, like upstream's `children().forEach`: the next sibling is read after the edit.
            let mut child = ast.first_child_node(node);
            while let Some(c) = child {
                self.wrap_argument_in_list(ast, c, emit);
                child = ast.next_sibling(c);
            }
        }
    }
}

impl ArgumentListWrappingRule {
    fn need_to_wrap_argument_list(&self, ast: &Ast, node: NodeId) -> bool {
        // skip when there are no arguments
        if ast.first_child_node(node).and_then(|it| ast.next_sibling(it)).map(|it| ast.element_type(it)) != Some(RPAR)
            // skip lambda arguments
            && ast.parent(node).map(|it| ast.element_type(it)) != Some(FUNCTION_LITERAL)
            // skip if number of arguments is big
            && ast.children(node).filter(|&it| ast.element_type(it) == VALUE_ARGUMENT).count() as i64
                <= self.ignore_when_parameter_count_greater_or_equal_than_property as i64
        {
            // each argument should be on a separate line if at least one of the arguments is, or if maxLineLength is
            // exceeded (and separating arguments with \n would actually help)
            text_contains_ignoring_lambda(ast, node, '\n') || self.exceeds_max_line_length(ast, node)
        } else {
            false
        }
    }

    /// 1.8 measures the whole line, not just up to the list's `)` (#3252).
    fn exceeds_max_line_length(&self, ast: &Ast, node: NodeId) -> bool {
        if self.ktlint_version.is_1_8() {
            return ast.line_length(ast.drop_trailing_eol_comment(ast.leaves_on_line(node))) as i64 > self.max_line_length as i64
                && !ast.text_contains(node, '\n');
        }
        if ast.text_contains(node, '\n') {
            return false;
        }
        assert!(ast.element_type(node) == VALUE_ARGUMENT_LIST, "IllegalArgumentException: Failed requirement.");
        let stop_at_leaf = ast.find_child_by_type(node, RPAR).map(|it| ast.first_child_leaf_or_self(it));
        ast.has_no_max_line_length_suppression(node)
            && (self.max_line_length as i64)
                < ast.line_length(ast.leaves_on_line(node).take_while(|&it| ast.prev_leaf(it) != stop_at_leaf)) as i64
    }

    fn intended_indent(&self, ast: &Ast, child: NodeId) -> String {
        let parent = ast.parent(child).expect("NullPointerException: parent!!");
        // IDEA quirks: the arguments get the indent of the line of `generic<\n T,\n R>(` or of `foo\n .bar = Baz(`,
        // instead of one more.
        let mut it = if has_type_argument_list_in_front(ast, parent) {
            -1
        } else if is_part_of_dot_qualified_assignment_expression(ast, parent) {
            -1
        } else {
            0
        };
        if is_on_same_line_as_control_flow_keyword(ast, parent) {
            it += 1;
        }
        if ast.element_type(child) == VALUE_ARGUMENT {
            it += 1;
        }
        let indent_level_fix = it;
        let indent_level = self.editor_config_indent.indent_level_from(&ast.indent_without_newline_prefix(parent)) + indent_level_fix;
        format!("\n{}", self.editor_config_indent.indent.repeat(indent_level.max(0) as usize))
    }

    fn wrap_argument_in_list(&self, ast: &mut Ast, child: NodeId, emit: &mut Emit<'_>) {
        match ast.element_type(child) {
            LPAR => {
                if let Some(prev_leaf) = ast.prev_leaf(child)
                    && ast.is_white_space_with_newline(prev_leaf)
                {
                    emit(ast, ast.start_offset(child), error_message(ast, child), true).if_autocorrect_allowed(|| psi::delete(ast, prev_leaf));
                }
            }

            VALUE_ARGUMENT | RPAR => {
                // aiming for
                // ... LPAR
                // <line indent + indentSize> VALUE_PARAMETER...
                // <line indent> RPAR
                let intended_indent = self.intended_indent(ast, child);
                let prev_leaf = prev_white_space_with_new_line(ast, child).or_else(|| ast.prev_leaf(child));
                if ast.is_white_space_with_newline(prev_leaf) {
                    // Already wrapped; fixing the size of the indent is the responsibility of the IndentationRule.
                } else if let Some(prev_leaf) = prev_leaf.filter(|&it| ast.is_white_space(it)) {
                    // The current child needs to be wrapped to a newline. The indent is purely based on the previous leaf;
                    // the indent rule, if enabled, runs after this rule and determines the final indentation.
                    emit(ast, ast.start_offset(child), error_message(ast, child), true)
                        .if_autocorrect_allowed(|| ast.replace_text_with(prev_leaf, &intended_indent));
                } else {
                    // Insert a new whitespace element in order to wrap the current child to a new line.
                    emit(ast, ast.start_offset(child), error_message(ast, child), true).if_autocorrect_allowed(|| {
                        if let Some(parent) = ast.parent(child) {
                            let white_space = ast.new_leaf(WHITE_SPACE, &intended_indent);
                            ast.add_child(parent, white_space, Some(child));
                        }
                    });
                }
                // Indentation of child nodes need to be fixed by the IndentationRule.
            }

            _ => {}
        }
    }
}

fn error_message(ast: &Ast, node: NodeId) -> &'static str {
    match ast.element_type(node) {
        LPAR => "Unnecessary newline before \"(\"",
        VALUE_ARGUMENT => "Argument should be on a separate line (unless all arguments can fit a single line)",
        RPAR => "Missing newline before \")\"",
        _ => panic!("UnsupportedOperationException"),
    }
}

fn text_contains_ignoring_lambda(ast: &Ast, node: NodeId, char: char) -> bool {
    ast.children(node).any(|child| {
        is_whitespace_containing(ast, child, char)
            || is_collection_literal_containing(ast, child, char)
            || is_value_argument_containing(ast, child, char)
    })
}

fn is_whitespace_containing(ast: &Ast, node: NodeId, char: char) -> bool {
    ast.is_white_space(node) && ast.text_contains(node, char)
}

fn is_collection_literal_containing(ast: &Ast, node: NodeId, char: char) -> bool {
    ast.element_type(node) == COLLECTION_LITERAL_EXPRESSION && ast.text_contains(node, char)
}

fn is_value_argument_containing(ast: &Ast, node: NodeId, char: char) -> bool {
    ast.element_type(node) == VALUE_ARGUMENT && ast.children(node).any(|it| text_contains_ignoring_lambda(ast, it, char))
}

fn has_type_argument_list_in_front(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .and_then(|it| ast.find_child_by_type(it, TYPE_ARGUMENT_LIST))
        .is_some_and(|it| ast.children(it).any(|it| ast.is_white_space_with_newline(it)))
}

fn is_part_of_dot_qualified_assignment_expression(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .and_then(|it| ast.parent(it))
        .filter(|&it| ast.element_type(it) == BINARY_EXPRESSION)
        .is_some_and(|binary_expression| {
            ast.first_child_node(binary_expression).map(|it| ast.element_type(it)) == Some(DOT_QUALIFIED_EXPRESSION)
                && ast
                    .find_child_by_type(binary_expression, OPERATION_REFERENCE)
                    .and_then(|it| ast.first_child_node(it))
                    .map(|it| ast.element_type(it))
                    == Some(EQ)
        })
}

fn prev_white_space_with_new_line(ast: &Ast, node: NodeId) -> Option<NodeId> {
    let mut prev = ast.prev_leaf(node);
    while let Some(p) = prev.filter(|&p| ast.is_white_space(p) || ast.is_part_of_comment(p)) {
        if ast.is_white_space_with_newline(p) {
            return Some(p);
        }
        prev = ast.prev_leaf(p);
    }
    None
}

fn is_on_same_line_as_control_flow_keyword(ast: &Ast, node: NodeId) -> bool {
    let Some(mut prev_leaf) = ast.prev_leaf(node) else { return false };
    while !CONTROL_FLOW_KEYWORDS.contains(ast.element_type(prev_leaf)) {
        if ast.is_white_space_with_newline(prev_leaf) {
            return false;
        }
        let Some(p) = ast.prev_leaf(prev_leaf) else { return false };
        prev_leaf = p;
    }
    true
}

const UNSET_IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY: i32 = i32::MAX;

static IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE: PropertyType<i32> = PropertyType {
    name: "ktlint_argument_list_wrapping_ignore_when_parameter_count_greater_or_equal_than",
    description: "Do not wrap parameters on separate lines when at least the specified number of parameters are specified. Use 'unset' to always wrap each parameter.",
    parser: positive_int_value_parser,
    possible_values: &["1", "2", "3", "4", "5", "6", "7", "8", "9", "unset"],
    lower_casing: true,
};

pub static IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY: LazyLock<EditorConfigProperty<i32>> =
    LazyLock::new(|| EditorConfigProperty {
        // Historically, all code styles have used 8 as the magic value.
        ktlint_official_code_style_default_value: UNSET_IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY,
        property_mapper: Some(|property, _| {
            if property.is_some_and(|p| p.is_unset()) {
                Some(UNSET_IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY)
            } else {
                property.and_then(|p| p.get_value_as(&IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE))
            }
        }),
        property_writer: |property| {
            if *property == UNSET_IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY {
                "unset".to_owned()
            } else {
                property.to_string()
            }
        },
        ..EditorConfigProperty::new(&IGNORE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE, 8)
    });
