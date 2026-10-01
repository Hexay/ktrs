//! Port of ktlint-ruleset-standard `ContextParameterListWrappingRule.kt` (id `context-parameter-list-wrapping`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    self, CONTEXT_PARAMETER_LIST, CONTEXT_RECEIVER, FUN, FUNCTION_TYPE, GT, RPAR, TYPE_ARGUMENT_LIST, TYPE_PROJECTION,
    TYPE_REFERENCE, VALUE_PARAMETER, VALUE_PARAMETER_LIST,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines, AstNodeQueries};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, MAX_LINE_LENGTH_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;

/// Wrapping of context receiver list to a separate line. Only affects a context receiver list without a context receiver
/// (those are wrapped by `context-receiver-wrapping`).
pub struct ContextParameterListWrappingRule {
    indent_config: IndentConfig,
    max_line_length: i32,
}

impl ContextParameterListWrappingRule {
    pub fn new() -> ContextParameterListWrappingRule {
        ContextParameterListWrappingRule {
            indent_config: IndentConfig::default_indent_config(),
            max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value,
        }
    }
}

impl Default for ContextParameterListWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

const VISITED_TYPES: TokenSet = TokenSet::create(&[CONTEXT_PARAMETER_LIST, TYPE_ARGUMENT_LIST]);

impl RuleV2 for ContextParameterListWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:context-parameter-list-wrapping")
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
        ]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        self.max_line_length = max_line_length(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let element_type = ast.element_type(node);
        if element_type == CONTEXT_PARAMETER_LIST && is_context_parameter(ast, node) {
            self.visit_context_receiver_list(ast, node, emit);
        } else if element_type == TYPE_ARGUMENT_LIST && is_context_parameter(ast, node) {
            self.visit_context_receiver_type_argument_list(ast, node, emit);
        }
    }
}

fn is_context_parameter(ast: &Ast, node: NodeId) -> bool {
    ast.is_part_of(node, CONTEXT_PARAMETER_LIST) && ast.find_child_by_type(node, CONTEXT_RECEIVER).is_none()
}

impl ContextParameterListWrappingRule {
    fn visit_context_receiver_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        // Context receiver must be followed by new line or comment unless it is a type reference of a parameter
        if let Some(node_after_context_receiver) = Some(node)
            .filter(|&it| !is_type_reference_parameter_in_function(ast, it))
            .map(|it| ast.last_child_leaf_or_self(it))
            .and_then(|it| ast.next_leaf_matching(it, |it| !ast.is_white_space_without_newline(it) && !ast.is_part_of_comment(it)))
            .filter(|&it| !ast.is_white_space_with_newline(it))
        {
            emit(ast, ast.start_offset(node_after_context_receiver), "Expected a newline after the context parameter", true)
                .if_autocorrect_allowed(|| {
                    let indent = self.indent_config.parent_indent_of(ast, node);
                    let first = ast.first_child_leaf_or_self(node_after_context_receiver);
                    ast.upsert_whitespace_before_me(first, &indent);
                });
        }

        // Check line length assuming that the context receiver is indented correctly. Wrapping rule must however run before indenting.
        if !ast.text_contains(node, '\n')
            && ast.has_no_max_line_length_suppression(node)
            && (ast.indent_without_newline_prefix(node).encode_utf16().count() + ast.text_length_utf16(node)) as i64
                > self.max_line_length as i64
        {
            let parameters: Vec<NodeId> = ast.children(node).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).collect();
            for it in parameters {
                emit(ast, ast.start_offset(it), "Newline expected before context parameter as max line length is violated", true)
                    .if_autocorrect_allowed(|| {
                        let indent = self.indent_config.child_indent_of(ast, node);
                        if let Some(prev_leaf) = ast.prev_leaf(it) {
                            ast.upsert_whitespace_after_me(prev_leaf, &indent);
                        }
                    });
            }
            if let Some(rpar) = ast.find_child_by_type(node, RPAR) {
                emit(ast, ast.start_offset(rpar), "Newline expected before closing parenthesis as max line length is violated", true)
                    .if_autocorrect_allowed(|| {
                        let indent = ast.indent(node);
                        ast.upsert_whitespace_before_me(rpar, &indent);
                    });
            }
        }
    }

    fn visit_context_receiver_type_argument_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let context_receiver_text = ast.parent(node).map(|it| ast.text(it)).unwrap_or_default();
        // Check line length assuming that the context receiver is indented correctly. Wrapping rule must however run
        // before indenting.
        if !context_receiver_text.contains('\n')
            && ast.has_no_max_line_length_suppression(node)
            && (ast.indent_without_newline_prefix(node).encode_utf16().count() + context_receiver_text.encode_utf16().count()) as i64
                > self.max_line_length as i64
        {
            let projections: Vec<NodeId> = ast.children(node).filter(|&it| ast.element_type(it) == TYPE_PROJECTION).collect();
            for it in projections {
                emit(
                    ast,
                    ast.start_offset(it),
                    "Newline expected before context parameter type projection as max line length is violated",
                    true,
                )
                .if_autocorrect_allowed(|| {
                    let indent = self.indent_config.child_indent_of(ast, node);
                    ast.upsert_whitespace_before_me(it, &indent);
                });
            }
            if let Some(gt) = ast.find_child_by_type(node, GT) {
                emit(ast, ast.start_offset(gt), "Newline expected before closing angle bracket as max line length is violated", true)
                    .if_autocorrect_allowed(|| {
                        // Not de-indented like ")", "}" and "]", to match the IntelliJ IDEA formatter for type argument lists.
                        let indent = self.indent_config.child_indent_of(ast, node);
                        ast.upsert_whitespace_before_me(gt, &indent);
                    });
            }
        }
    }
}

fn is_type_reference_parameter_in_function(ast: &Ast, node: NodeId) -> bool {
    let parent_of = |n: Option<NodeId>, element_type: SyntaxKind| n.filter(|&it| ast.element_type(it) == element_type).and_then(|it| ast.parent(it));
    let n = parent_of(Some(node), CONTEXT_PARAMETER_LIST);
    let n = parent_of(n, FUNCTION_TYPE);
    let n = parent_of(n, TYPE_REFERENCE);
    let n = parent_of(n, VALUE_PARAMETER);
    let n = parent_of(n, VALUE_PARAMETER_LIST);
    n.is_some_and(|it| ast.element_type(it) == FUN)
}
