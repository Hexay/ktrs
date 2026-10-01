//! Port of ktlint-ruleset-standard `ChainMethodContinuationRule.kt`, split in upstream order: this file (the rule
//! and its companion object, `beforeVisitChildNodes` .. `isPrecededByComment`), `whitespace.rs`
//! (`insertWhiteSpaceBeforeChainOperator` .. `fixWhiteSpaceAfterChainOperators`) and `chained_expression.rs`.

mod chained_expression;
mod whitespace;

use std::rc::Rc;
use std::sync::LazyLock;

use ktrs_ast::{Ast, NodeId};
use ktrs_editorconfig::PropertyType;
use ktrs_editorconfig::property_type::positive_int_value_parser;
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{
    BINARY_EXPRESSION, CALL_EXPRESSION, CLASS_LITERAL_EXPRESSION, CLOSING_QUOTE, DOT, DOT_QUALIFIED_EXPRESSION, FUNCTION_LITERAL,
    IMPORT_DIRECTIVE, LAMBDA_ARGUMENT, LAMBDA_EXPRESSION, LBRACE, LONG_STRING_TEMPLATE_ENTRY, PACKAGE_DIRECTIVE, RBRACE, RBRACKET,
    REFERENCE_EXPRESSION, RPAR, SAFE_ACCESS, SAFE_ACCESS_EXPRESSION, STRING_TEMPLATE,
};

use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::engine::verifying_shortcuts;
use crate::editorconfig::{
    CODE_STYLE_PROPERTY, EditorConfigProperty, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, MAX_LINE_LENGTH_PROPERTY, PropertyRef,
};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;

use chained_expression::ChainedExpression;

const CHAIN_OPERATOR_EXPRESSION_CONVERTER_TOKEN_SET: TokenSet = TokenSet::create(&[DOT_QUALIFIED_EXPRESSION, SAFE_ACCESS_EXPRESSION]);
const CHAIN_OPERATOR_TOKEN_SET: TokenSet = TokenSet::create(&[DOT, SAFE_ACCESS]);
const GROUP_CLOSING_ELEMENT_TYPE: TokenSet = TokenSet::create(&[CLOSING_QUOTE, RBRACE, RBRACKET, RPAR]);

const FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_UNSET: i32 = i32::MAX;

pub static FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE: PropertyType<i32> = PropertyType {
    name: "ktlint_chain_method_rule_force_multiline_when_chain_operator_count_greater_or_equal_than",
    description: "Force wrapping of chained methods in case an expression contains at least the specified number of chain \
                  operators. By default this parameter is set to 4.",
    parser: positive_int_value_parser,
    possible_values: &["1", "2", "3", "4", "5", "6", "7", "8", "9", "unset"],
    lower_casing: true,
};

pub static FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY: LazyLock<EditorConfigProperty<i32>> =
    LazyLock::new(|| EditorConfigProperty {
        property_mapper: Some(|property, _| {
            if property.is_some_and(|p| p.is_unset()) {
                Some(FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_UNSET)
            } else {
                property.and_then(|p| p.get_value_as(&FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE))
            }
        }),
        property_writer: |property| {
            if *property == FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_UNSET {
                "unset".to_owned()
            } else {
                property.to_string()
            }
        },
        ..EditorConfigProperty::new(&FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY_TYPE, 4)
    });

/// Methods chained with `.` or `?.` all fit on one line, or each goes on its own line. Stricter than the Kotlin coding
/// conventions (<https://kotlinlang.org/docs/coding-conventions.html#wrap-chained-calls>), hence ktlint_official only.
pub struct ChainMethodContinuationRule {
    indent_config: IndentConfig,
    max_line_length: i32,
    force_multiline_when_chain_operator_count_greater_or_equal_than_property: i32,
    traversal: TraversalState,
    /// The last chain built, keyed by its chain parent and the tree's modification count.
    last_chained_expression: Option<((NodeId, u64), Rc<ChainedExpression>)>,
}

impl ChainMethodContinuationRule {
    pub fn new() -> ChainMethodContinuationRule {
        ChainMethodContinuationRule {
            indent_config: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
            force_multiline_when_chain_operator_count_greater_or_equal_than_property:
                FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY.default_value,
            traversal: TraversalState::default(),
            last_chained_expression: None,
        }
    }
}

impl Default for ChainMethodContinuationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for ChainMethodContinuationRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:chain-method-continuation")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(CHAIN_OPERATOR_TOKEN_SET)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*CODE_STYLE_PROPERTY),
            PropertyRef::from(&*FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY),
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY),
        ]
    }

    fn is_official_code_style(&self) -> bool {
        true
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal)
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        if self.indent_config.disabled() {
            self.traversal.stop_traversal_of_ast();
        }
        self.max_line_length = max_line_length(editor_config);
        self.force_multiline_when_chain_operator_count_greater_or_equal_than_property =
            editor_config.get(&FORCE_MULTILINE_WHEN_CHAIN_OPERATOR_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if !CHAIN_OPERATOR_TOKEN_SET.contains(ast.element_type(node))
            || !ast.parent(node).is_some_and(|p| CHAIN_OPERATOR_EXPRESSION_CONVERTER_TOKEN_SET.contains(ast.element_type(p)))
        {
            return;
        }
        let chain_operator = node;
        // The AST of a chain is restructured into a ChainedExpression; it is processed at its first chain operator only
        let chained_expression = self.chained_expression(ast, chain_operator);
        if chain_operator != chained_expression.chain_operators[0] {
            return;
        }
        let root_parent_type = ast.parent(chained_expression.root_ast_node).map(|p| ast.element_type(p));
        if matches!(root_parent_type, Some(IMPORT_DIRECTIVE | PACKAGE_DIRECTIVE | LONG_STRING_TEMPLATE_ENTRY)) {
            return;
        }
        self.fix_whitespace_before_chain_operators(ast, &chained_expression, emit);
        whitespace::disallow_comment_between_dot_and_call_expression(ast, &chained_expression, emit);
        whitespace::fix_white_space_after_chain_operators(ast, &chained_expression, emit);
    }
}

impl ChainMethodContinuationRule {
    /// `ChainedExpression.createFrom(chainOperator)`. Every operator of a chain builds the same expression, so it is
    /// reused while the tree is unchanged (building only reads the tree); else a chain of n operators costs n².
    fn chained_expression(&mut self, ast: &Ast, chain_operator: NodeId) -> Rc<ChainedExpression> {
        let key = (ChainedExpression::chain_parent(ast, chain_operator), ast.modification_count());
        if let Some((cached_key, cached)) = &self.last_chained_expression
            && *cached_key == key
        {
            assert!(
                !verifying_shortcuts() || **cached == ChainedExpression::create_from(ast, chain_operator, key.0),
                "chained expression cache is stale"
            );
            return cached.clone();
        }
        let created = Rc::new(ChainedExpression::create_from(ast, chain_operator, key.0));
        self.last_chained_expression = Some((key, created.clone()));
        created
    }

    fn fix_whitespace_before_chain_operators(&self, ast: &mut Ast, chained_expression: &ChainedExpression, emit: &mut Emit<'_>) {
        let wrap_before_each_chain_operator = self.wrap_before_chain_operator(ast, chained_expression);
        let exceeds_max_line_length = self.exceeds_max_line_length(ast, chained_expression);
        let chain_operators: Vec<NodeId> = chained_expression
            .chain_operators
            .iter()
            .copied()
            .filter(|&it| !is_java_class_reference_expression(ast, it))
            .filter(|&it| !is_reference_expression(ast, it))
            .collect();
        for chain_operator in chain_operators {
            if whitespace::should_be_on_same_line_as_closing_element_of_previous_expression_in_method_chain(ast, chain_operator) {
                whitespace::remove_white_space_before_chain_operator(ast, chain_operator, emit);
            } else if wrap_before_each_chain_operator || exceeds_max_line_length || is_preceded_by_comment(ast, chain_operator) {
                self.insert_white_space_before_chain_operator(ast, chain_operator, emit);
            }
        }
    }
}

fn is_java_class_reference_expression(ast: &Ast, node: NodeId) -> bool {
    let next_code_sibling = ast.next_code_sibling(node);
    ast.parent(node).map(|p| ast.element_type(p)) == Some(DOT_QUALIFIED_EXPRESSION)
        && ast.prev_code_sibling(node).map(|it| ast.element_type(it)) == Some(CLASS_LITERAL_EXPRESSION)
        && next_code_sibling.map(|it| ast.element_type(it)) == Some(REFERENCE_EXPRESSION)
        && next_code_sibling.is_some_and(|it| ast.text(ast.first_child_leaf_or_self(it)) == "java")
}

fn is_reference_expression(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node).is_some_and(|p| is_nested_reference_expression(ast, p))
}

fn is_nested_reference_expression(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == DOT_QUALIFIED_EXPRESSION
        && ast.first_child_node(node).map(|it| {
            ast.element_type(it) == REFERENCE_EXPRESSION || has_right_hand_side_reference_expression(ast, it)
        }) == Some(true)
        && has_right_hand_side_reference_expression(ast, node)
}

fn has_right_hand_side_reference_expression(ast: &Ast, node: NodeId) -> bool {
    let last = ast.last_child_node(node).expect("NoSuchElementException: Sequence is empty.");
    ast.element_type(last) == REFERENCE_EXPRESSION
}

impl ChainMethodContinuationRule {
    fn wrap_before_chain_operator(&self, ast: &Ast, chained_expression: &ChainedExpression) -> bool {
        let it = chained_expression;
        if it.has_newline_between_first_and_last_chain_operator {
            // Disallow a partly wrapped chain, like `listOf(1, 2, 3, 4)\n .filter { it > 2 }?.filter { it > 3 }` or
            // `listOf(1, 2, 3, 4).filter {\n it > 2\n }?.filter { it > 3 }`
            true
        } else if is_chained_expression_on_string_template(ast, it) && !it.has_newline_after_last_chain_operator {
            // Allow `"""\nsome text\n""".uppercase().replace("foo bar", "bar foo").trimIndent()`
            false
        } else if !it.has_newline_before_first_chain_operator && !it.has_newline_after_last_chain_operator {
            // Allow `listOf(1, 2, 3).filter { it > 2 }.filter { it > 3 }`, also when the last lambda is multiline
            it.chain_operators.len() as i64 >= self.force_multiline_when_chain_operator_count_greater_or_equal_than_property as i64
        } else {
            false
        }
    }
}

fn is_chained_expression_on_string_template(ast: &Ast, chained_expression: &ChainedExpression) -> bool {
    ast.prev_code_sibling(chained_expression.chain_operators[0]).map(|it| ast.element_type(it)) == Some(STRING_TEMPLATE)
}

impl ChainMethodContinuationRule {
    /// Chains inside a binary expression are skipped: which of the two to wrap first depends on the situation.
    fn exceeds_max_line_length(&self, ast: &Ast, chained_expression: &ChainedExpression) -> bool {
        let root = chained_expression.root_ast_node;
        if ast.parent(root).map(|p| ast.element_type(p)) == Some(BINARY_EXPRESSION) {
            false
        } else {
            let last_chain_operator = *chained_expression.chain_operators.last().expect("NoSuchElementException: List is empty.");
            let stop_at_leaf = start_of_lambda_argument_in_call_expression_or_null(ast, last_chain_operator)
                .or_else(|| ast.next_leaf(ast.last_child_leaf_or_self(root)));
            ast.has_no_max_line_length_suppression(root)
                && ast.line_length(ast.drop_trailing_eol_comment(ast.leaves_on_line(root)).take_while(|&it| ast.prev_leaf(it) != stop_at_leaf))
                    as i64
                    > self.max_line_length as i64
        }
    }
}

fn start_of_lambda_argument_in_call_expression_or_null(ast: &Ast, node: NodeId) -> Option<NodeId> {
    assert!(CHAIN_OPERATOR_TOKEN_SET.contains(ast.element_type(node)), "IllegalArgumentException: Failed requirement.");
    ast.next_code_sibling(node)
        .filter(|&it| ast.element_type(it) == CALL_EXPRESSION)
        .and_then(|it| ast.find_child_by_type(it, LAMBDA_ARGUMENT))
        .and_then(|it| ast.find_child_by_type(it, LAMBDA_EXPRESSION))
        .and_then(|it| ast.find_child_by_type(it, FUNCTION_LITERAL))
        .and_then(|it| ast.find_child_by_type(it, LBRACE))
}

fn is_preceded_by_comment(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node).is_some_and(|p| ast.children(p).any(|it| ast.is_part_of_comment(it)))
}
