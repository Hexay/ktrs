//! Port of ktlint-ruleset-standard `FunctionSignatureRule.kt`, in upstream order over `mod.rs` (up to
//! `getFunctionSignatureNodesLength`), `parameter_list.rs`, `body.rs` and `properties.rs` (the companion object).

mod body;
mod parameter_list;
mod properties;

pub use properties::{
    FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY, FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY,
    FunctionBodyExpressionWrapping,
};

use ktrs_ast::{Ast, NodeId};

use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{
    CODE_STYLE_PROPERTY, CodeStyleValue, EditorConfig, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, MAX_LINE_LENGTH_PROPERTY,
    MAX_LINE_LENGTH_PROPERTY_OFF, PropertyRef,
};
use crate::element_type::{
    ANNOTATION, ANNOTATION_ENTRY, BLOCK, BLOCK_COMMENT, CONTEXT_PARAMETER_LIST, EOL_COMMENT, EQ, FUN, FUN_KEYWORD, MODIFIER_LIST, RPAR,
    VALUE_PARAMETER, VALUE_PARAMETER_LIST, WHITE_SPACE,
};
use crate::indent_config::IndentConfig;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;

const VISITED_TYPES: TokenSet = TokenSet::create(&[FUN]);

// Shared with `ClassSignatureRule`, which has identical private copies upstream.
pub(crate) use body::{children_between, collect_leaves_recursively, join_text_length};
use body::count_parameters;

pub struct FunctionSignatureRule {
    code_style: CodeStyleValue,
    indent_config: IndentConfig,
    max_line_length: i32,
    function_signature_wrapping_minimum_parameters: i32,
    function_body_expression_wrapping: FunctionBodyExpressionWrapping,
}

impl FunctionSignatureRule {
    pub fn new() -> FunctionSignatureRule {
        FunctionSignatureRule {
            code_style: CODE_STYLE_PROPERTY.default_value,
            indent_config: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
            function_signature_wrapping_minimum_parameters: FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY.default_value,
            function_body_expression_wrapping: FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY.default_value,
        }
    }
}

impl Default for FunctionSignatureRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for FunctionSignatureRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:function-signature")
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
            PropertyRef::from(&*FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY),
        ]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.code_style = editor_config.get(&CODE_STYLE_PROPERTY);
        self.function_signature_wrapping_minimum_parameters =
            editor_config.get(&FORCE_MULTILINE_WHEN_PARAMETER_COUNT_GREATER_OR_EQUAL_THAN_PROPERTY);
        self.function_body_expression_wrapping = editor_config.get(&FUNCTION_BODY_EXPRESSION_WRAPPING_PROPERTY);
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        self.max_line_length = max_line_length(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == FUN {
            if function_signature_nodes(ast, node).iter().any(|&it| matches!(ast.element_type(it), EOL_COMMENT | BLOCK_COMMENT)) {
                // Rewriting function signatures in a consistent manner is hard or sometimes even impossible. For
                // example a multiline signature which could fit on one line can not be rewritten in case it
                // contains an EOL comment. Rewriting a single line signature which exceeds the max line length to a
                // multiline signature is hard when it contains block comments. For now, it does not seem worth the
                // effort to attempt it.
                return;
            }

            self.visit_function_signature(ast, node, emit);
        }
    }
}

fn function_signature_nodes(ast: &Ast, node: NodeId) -> Vec<NodeId> {
    // Find the signature including the element that has to be placed on the same line as the function signature
    //     fun foo(bar: String) {
    // or
    //     fun foo(bar: String) =
    let first_code_child = get_first_code_child(ast, node).map(|it| ast.first_child_leaf_or_self(it));
    let start_of_body_block = ast.find_child_by_type(node, BLOCK).and_then(|it| ast.first_child_node(it));
    let start_of_body_expression = ast.find_child_by_type(node, EQ);
    children_between(
        collect_leaves_recursively(ast, node),
        |it| Some(it) == first_code_child,
        |it| Some(it) == start_of_body_block || Some(it) == start_of_body_expression,
    )
}

fn get_first_code_child(ast: &Ast, node: NodeId) -> Option<NodeId> {
    let fun_node = if ast.element_type(node) == FUN_KEYWORD { ast.parent(node) } else { Some(node) };
    if let Some(modifier_list) = fun_node.and_then(|it| ast.find_child_by_type(it, MODIFIER_LIST)) {
        for current_node in ast.children(modifier_list) {
            if ast.element_type(current_node) == CONTEXT_PARAMETER_LIST {
                let leaf_after_composite_context_receiver_list = ast.next_leaf(ast.last_child_leaf_or_self(current_node));
                return if ast.is_white_space_with_newline(leaf_after_composite_context_receiver_list) {
                    // Ignore context receiver when followed by newline or EOL comment. Also, ignore that newline or EOL
                    // comment.
                    //     context(Foo)
                    //     fun foo()
                    // or
                    //     context(_: Foo)
                    //     fun foo()
                    // Same when using context parameter instead of context receiver
                    leaf_after_composite_context_receiver_list.and_then(|it| ast.next_leaf(it))
                } else {
                    Some(current_node)
                };
            } else if !matches!(ast.element_type(current_node), ANNOTATION | ANNOTATION_ENTRY | WHITE_SPACE) {
                return Some(current_node);
            }
        }
        return ast.next_code_sibling(modifier_list);
    }

    fun_node.and_then(|it| ast.next_code_leaf(it))
}

impl FunctionSignatureRule {
    fn visit_function_signature(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        assert!(ast.element_type(node) == FUN, "IllegalArgumentException: Failed requirement.");

        let force_multiline_signature = self.has_minimum_number_of_parameters(ast, node)
            || contains_multiline_parameter(ast, node)
            || (self.code_style == CodeStyleValue::KtlintOfficial && contains_annotated_parameter(ast, node));
        if self.is_max_line_length_set() {
            let single_line_function_signature_length = self.calculate_function_signature_length_as_single_line_signature(ast, node, emit);
            // Function signatures not having parameters, should not be reformatted automatically. It would result in function signatures
            // like below, which are not acceptable:
            //     fun aVeryLongFunctionName(
            //     ) = "some-value"
            //
            //     fun aVeryLongFunctionName(
            //     ): SomeVeryLongTypeName =
            //         SomeVeryLongTypeName(...)
            // Leave it up to the max-line-length rule to detect those violations so that the developer can handle it manually.
            let rewrite_function_signature_with_parameters = count_parameters(ast, node) > 0
                && (ast.has_no_max_line_length_suppression(node) && single_line_function_signature_length > self.max_line_length);
            if force_multiline_signature || rewrite_function_signature_with_parameters {
                self.fix_white_spaces_in_value_parameter_list(ast, node, emit, true, false);
                if ast.find_child_by_type(node, EQ).is_none() {
                    self.fix_whitespace_before_function_body_block(ast, node, emit, false);
                } else {
                    // Due to rewriting the function signature, the remaining length on the last line of the multiline signature needs to be
                    // recalculated
                    let length_of_last_line = recalculate_remaining_length_for_first_line_of_body_expression(ast, node);
                    self.fix_function_body_expression(ast, node, emit, self.max_line_length - length_of_last_line);
                }
            } else {
                self.fix_white_spaces_in_value_parameter_list(ast, node, emit, false, false);
                if ast.find_child_by_type(node, EQ).is_none() {
                    self.fix_whitespace_before_function_body_block(ast, node, emit, false);
                } else {
                    self.fix_function_body_expression(ast, node, emit, self.max_line_length - single_line_function_signature_length);
                }
            }
        } else {
            // When max line length is not set then keep it as single line function signature only when the original
            // signature already was a single line signature. Otherwise, rewrite the entire signature as a multiline
            // signature.
            let rewrite_to_single_line_function_signature = !function_signature_nodes(ast, node).iter().any(|&it| ast.text_contains(it, '\n'));
            if !force_multiline_signature && rewrite_to_single_line_function_signature {
                self.fix_white_spaces_in_value_parameter_list(ast, node, emit, false, false);
            } else {
                self.fix_white_spaces_in_value_parameter_list(ast, node, emit, true, false);
            }
            self.fix_function_body_expression(ast, node, emit, MAX_LINE_LENGTH_PROPERTY_OFF);
        }
    }
}

fn recalculate_remaining_length_for_first_line_of_body_expression(ast: &Ast, node: NodeId) -> i32 {
    let closing_parenthesis = ast.find_child_by_type(node, VALUE_PARAMETER_LIST).and_then(|it| ast.find_child_by_type(it, RPAR));
    let tail_nodes_of_function_signature = children_between(function_signature_nodes(ast, node), |it| Some(it) == closing_parenthesis, |_| false);

    utf16_len(&ast.indent_without_newline_prefix(node)) + tail_nodes_of_function_signature.iter().map(|&it| ast.text_length_utf16(it) as i32).sum::<i32>()
}

fn contains_multiline_parameter(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, VALUE_PARAMETER_LIST)
        .is_some_and(|list| ast.children(list).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).any(|it| ast.text_contains(it, '\n')))
}

fn contains_annotated_parameter(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, VALUE_PARAMETER_LIST)
        .is_some_and(|list| ast.children(list).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).any(|it| is_annotated(ast, it)))
}

fn is_annotated(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, MODIFIER_LIST)
        .is_some_and(|list| ast.children(list).any(|it| ast.element_type(it) == ANNOTATION_ENTRY))
}

impl FunctionSignatureRule {
    fn calculate_function_signature_length_as_single_line_signature(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) -> i32 {
        let actual_function_signature_length = get_function_signature_length(ast, node);

        // Calculate the length of the function signature in case it would be rewritten as single line (and without a
        // maximum line length). The white space correction will be calculated via a dry run of the actual fix.
        actual_function_signature_length
            // Calculate the white space correction in case the signature would be rewritten to a single line
            + self.fix_white_spaces_in_value_parameter_list(ast, node, emit, false, true)
            + if ast.find_child_by_type(node, EQ).is_none() { self.fix_whitespace_before_function_body_block(ast, node, emit, true) } else { 0 }
    }
}

fn get_function_signature_length(ast: &Ast, node: NodeId) -> i32 {
    utf16_len(&ast.indent_without_newline_prefix(node)) + get_function_signature_nodes_length(ast, node)
}

fn get_function_signature_nodes_length(ast: &Ast, node: NodeId) -> i32 {
    join_text_length(ast, &function_signature_nodes(ast, node))
}

/// Kotlin's `String.length`.
fn utf16_len(text: &str) -> i32 {
    text.encode_utf16().count() as i32
}
