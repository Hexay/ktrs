//! Port of ktlint-ruleset-standard `FunctionLiteralRule.kt`, split in upstream order: this file (the rule ..
//! `isFunctionLiteralLambdaWithNonEmptyValueParameterList`) and `rewrite.rs` (`rewriteToMultilineParameterList` ..
//! `wrapAfterLbrace`). The debug logging is dropped.

mod rewrite;

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ARROW, BLOCK, FUNCTION_LITERAL, LAMBDA_ARGUMENT, LAMBDA_EXPRESSION, LBRACE, RBRACE, VALUE_PARAMETER, VALUE_PARAMETER_LIST,
};

use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{CODE_STYLE_PROPERTY, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, KtlintVersion, MAX_LINE_LENGTH_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet, TraversalState, VisitorModifier};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;

const VISITED_TYPES: TokenSet = TokenSet::create(&[FUNCTION_LITERAL]);

/// Parameter names of a multiline lambda go on the first line, followed by the arrow and a newline; a parameter list
/// too long for that line gets one parameter per line and the arrow on its own line
/// (<https://kotlinlang.org/docs/coding-conventions.html#lambdas>).
pub struct FunctionLiteralRule {
    indent_config: IndentConfig,
    max_line_length: i32,
    traversal: TraversalState,
    ktlint_version: KtlintVersion,
}

impl FunctionLiteralRule {
    pub fn new() -> FunctionLiteralRule {
        FunctionLiteralRule {
            indent_config: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
            traversal: TraversalState::default(),
            ktlint_version: KtlintVersion::default(),
        }
    }
}

impl Default for FunctionLiteralRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for FunctionLiteralRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:function-literal")
    }

    fn visitor_modifiers(&self) -> &'static [VisitorModifier] {
        const MODIFIERS: &[VisitorModifier] = &[VisitorModifier::run_after("standard:chain-method-continuation")];
        MODIFIERS
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*CODE_STYLE_PROPERTY),
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
            PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY),
        ]
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal)
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.max_line_length = max_line_length(editor_config);
        self.ktlint_version = KtlintVersion::of(editor_config);
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        if self.indent_config.disabled() {
            self.traversal.stop_traversal_of_ast();
        }
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == FUNCTION_LITERAL {
            if let Some(it) = ast.find_child_by_type(node, VALUE_PARAMETER_LIST) {
                self.visit_value_parameter_list(ast, it, emit);
            }
            if let Some(it) = ast.find_child_by_type(node, ARROW) {
                rewrite::visit_arrow(ast, it, emit);
            }
            if let Some(it) = ast.find_child_by_type(node, BLOCK) {
                self.visit_block(ast, it, emit);
            }
        }
    }
}

impl FunctionLiteralRule {
    fn visit_value_parameter_list(&self, ast: &mut Ast, parameter_list: NodeId, emit: &mut Emit<'_>) {
        let value_parameters = ast.children(parameter_list).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).count();
        if value_parameters > 1 || self.wrap_first_parameter_to_newline(ast, parameter_list) {
            if ast.text_contains(parameter_list, '\n') || self.does_not_fit_on_same_line_as_start_of_function_literal(ast, parameter_list) {
                self.rewrite_to_multiline_parameter_list(ast, parameter_list, emit);
            } else {
                rewrite::rewrite_to_single_line_function_literal(ast, parameter_list, emit);
            }
        } else if ast.text_contains(parameter_list, '\n') {
            // Allow a single parameter which is multiline itself (e.g. an annotated type):
            //    val foo = {
            //            bar:
            //                @Baz("baz")
            //                Bar
            //        ->
            //        bar()
            //    }
        } else {
            // Disallow `{\n bar ->` and `{ bar\n ->`
            rewrite::rewrite_to_single_line_function_literal(ast, parameter_list, emit);
        }
    }

    fn does_not_fit_on_same_line_as_start_of_function_literal(&self, ast: &Ast, node: NodeId) -> bool {
        require_value_parameter_list_in_function_literal(ast, node);
        let line_length = line_length_including_lbrace(ast, node)
            + 1 // space before parameter list
            + length_of_parameter_list_when_on_single_line(ast, node)
            + 3; // space after parameter list followed by ->
        ast.has_no_max_line_length_suppression_in(node, self.ktlint_version) && line_length as i64 > self.max_line_length as i64
    }
}

fn require_value_parameter_list_in_function_literal(ast: &Ast, node: NodeId) {
    assert!(
        ast.element_type(node) == VALUE_PARAMETER_LIST && ast.parent(node).map(|p| ast.element_type(p)) == Some(FUNCTION_LITERAL),
        "IllegalArgumentException: Failed requirement."
    );
}

fn line_length_including_lbrace(ast: &Ast, node: NodeId) -> usize {
    require_value_parameter_list_in_function_literal(ast, node);
    let lbrace = ast
        .parent(node)
        .and_then(|p| ast.find_child_by_type(p, LBRACE))
        .expect("NullPointerException: findChildByType(LBRACE)!!");
    ast.line_length(ast.drop_trailing_eol_comment(ast.leaves_on_line(lbrace)).take_while(|&it| ast.prev_leaf(it) != Some(lbrace)))
}

fn length_of_parameter_list_when_on_single_line(ast: &Ast, node: NodeId) -> usize {
    assert!(ast.element_type(node) == VALUE_PARAMETER_LIST, "IllegalArgumentException: Failed requirement.");
    let stop_at_leaf = ast.next_leaf(ast.last_child_leaf_or_self(node));
    ast.leaves_forwards_including_self(ast.first_child_leaf_or_self(node))
        .take_while(|&it| Some(it) != stop_at_leaf)
        // Eliminate newlines and redundant spaces
        .map(|it| if ast.is_white_space(it) { 1 } else { ast.text_length_utf16(it) })
        .sum()
}

impl FunctionLiteralRule {
    /// 1.8 measures the whole line, not just up to the lambda's `}` (#3252).
    fn exceeds_max_line_length(&self, ast: &Ast, node: NodeId) -> bool {
        if self.ktlint_version.is_1_8() {
            return (self.max_line_length as i64) < ast.line_length(ast.drop_trailing_eol_comment(ast.leaves_on_line(node))) as i64;
        }
        assert!(ast.element_type(node) == BLOCK, "IllegalArgumentException: Failed requirement.");
        let stop_at_leaf = ast.next_sibling_matching(node, |it| ast.element_type(it) == RBRACE);
        ast.has_no_max_line_length_suppression_in(node, self.ktlint_version)
            && (self.max_line_length as i64)
                < ast.line_length(ast.drop_trailing_eol_comment(ast.leaves_on_line(node)).take_while(|&it| ast.prev_leaf(it) != stop_at_leaf))
                    as i64
    }

    /// Disallow when max line is exceeded: `val foo = someCallExpression { someLongParameterName ->`.
    fn wrap_first_parameter_to_newline(&self, ast: &Ast, node: NodeId) -> bool {
        if is_function_literal_lambda_with_non_empty_value_parameter_list(ast, node) && ast.has_no_max_line_length_suppression_in(node, self.ktlint_version) {
            let first_parameter = ast
                .children(node)
                .find(|&it| ast.element_type(it) == VALUE_PARAMETER)
                .expect("NoSuchElementException: Sequence contains no element matching the predicate.");
            let stop_at_leaf = ast.next_leaf_matching(ast.last_child_leaf_or_self(first_parameter), |it| {
                !ast.is_white_space_without_newline(it) && !ast.is_part_of_comment(it)
            });
            let leaves = ast.drop_trailing_eol_comment(ast.leaves_on_line(node)).take_while(|&it| ast.prev_leaf(it) != stop_at_leaf);
            ast.line_length(leaves) as i64 > self.max_line_length as i64
        } else {
            false
        }
    }
}

fn is_function_literal_lambda_with_non_empty_value_parameter_list(ast: &Ast, node: NodeId) -> bool {
    let parent_type = |n: NodeId| ast.parent(n).map(|p| ast.element_type(p));
    Some(node)
        .filter(|&it| ast.element_type(it) == VALUE_PARAMETER_LIST)
        .filter(|&it| ast.find_child_by_type(it, VALUE_PARAMETER).is_some())
        .filter(|&it| parent_type(it) == Some(FUNCTION_LITERAL))
        .and_then(|it| ast.parent(it))
        .filter(|&it| parent_type(it) == Some(LAMBDA_EXPRESSION))
        .and_then(|it| ast.parent(it))
        .filter(|&it| parent_type(it) == Some(LAMBDA_ARGUMENT))
        .is_some()
}
