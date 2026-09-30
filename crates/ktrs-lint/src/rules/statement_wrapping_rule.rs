//! Port of ktlint-ruleset-standard `StatementWrappingRule.kt`.

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATION_ENTRY, ARROW, BLOCK, CLASS, CLASS_BODY, ENUM_ENTRY, ENUM_KEYWORD, FUNCTION_LITERAL, LBRACE, MODIFIER_LIST, RBRACE,
    SEMICOLON, VALUE_PARAMETER_LIST, WHEN,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::{INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct StatementWrappingRule {
    indent_config: IndentConfig,
}

impl StatementWrappingRule {
    pub fn new() -> StatementWrappingRule {
        StatementWrappingRule { indent_config: IndentConfig::default_indent_config() }
    }
}

impl Default for StatementWrappingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for StatementWrappingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:statement-wrapping")
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
        match ast.element_type(node) {
            BLOCK => {
                let parent = ast.parent(node);
                if let Some(parent) = parent.filter(|&p| ast.element_type(p) == FUNCTION_LITERAL) {
                    // LBRACE and RBRACE are outside of BLOCK
                    self.visit_block(ast, parent, emit);
                } else {
                    self.visit_block(ast, node, emit);
                }
            }
            CLASS_BODY | WHEN => self.visit_block(ast, node, emit),
            SEMICOLON => visit_semi_colon(ast, node, emit),
            _ => {}
        }
    }
}

impl StatementWrappingRule {
    fn visit_block(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        // Allow `val foo = {}`
        if is_block_without_statements(ast, node) {
            return;
        }
        // Allow `val foo = { /* no-op */ }` and `val foo = { a, b -> println(a + b) }`
        if is_function_literal_with_single_statement_on_single_line(ast, node) {
            return;
        }
        // Allow `enum class FooBar { FOO, BAR }`, also below a comment
        if is_enum_class_on_single_line(ast, ast.parent(node).expect("NullPointerException: parent!!")) {
            return;
        }
        let Some(lbrace) = ast.find_child_by_type(node, LBRACE) else { return };
        // Allow `foo { bar ->\n doSomething()\n }`
        let lbrace_or_arrow =
            if is_function_literal_with_parameter_list(ast, node) { ast.find_child_by_type(node, ARROW) } else { Some(lbrace) };
        let Some(lbrace_or_arrow) = lbrace_or_arrow else { return };

        let next_code_leaf = ast.next_code_leaf(lbrace_or_arrow);
        if let Some(next_code_leaf) = next_code_leaf
            && ast.no_new_line_in_closed_range(lbrace_or_arrow, next_code_leaf)
        {
            let message = format!("Missing newline after '{}'", ast.text(lbrace_or_arrow));
            emit(ast, ast.start_offset(next_code_leaf), &message, true).if_autocorrect_allowed(|| {
                let indent = if ast.element_type(node) == WHEN {
                    self.indent_as_child(ast, lbrace_or_arrow)
                } else {
                    self.indent_as_sibling(ast, lbrace_or_arrow)
                };
                ast.upsert_whitespace_after_me(lbrace_or_arrow, &indent);
            });
        }

        if let Some(rbrace) = ast.find_child_by_type(node, RBRACE) {
            let prev_code_leaf = ast.prev_code_leaf(rbrace);
            if let Some(prev_code_leaf) = prev_code_leaf
                && ast.no_new_line_in_closed_range(prev_code_leaf, rbrace)
            {
                emit(ast, ast.start_offset(rbrace), "Missing newline before '}'", true).if_autocorrect_allowed(|| {
                    let indent = indent_as_parent(ast, rbrace);
                    ast.upsert_whitespace_before_me(rbrace, &indent);
                });
            }
        }
    }

    fn indent_as_child(&self, ast: &Ast, node: NodeId) -> String {
        ast.indent(node) + &self.indent_config.indent
    }

    fn indent_as_sibling(&self, ast: &Ast, node: NodeId) -> String {
        ast.indent(ast.parent(node).expect("NullPointerException: parent!!")) + &self.indent_config.indent
    }
}

fn is_block_without_statements(ast: &Ast, node: NodeId) -> bool {
    ast.find_child_by_type(node, LBRACE).and_then(|it| ast.next_code_leaf(it)).map(|it| ast.element_type(it)) == Some(RBRACE)
}

fn is_function_literal_with_parameter_list(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == FUNCTION_LITERAL && ast.find_child_by_type(node, VALUE_PARAMETER_LIST).is_some()
}

fn is_function_literal_with_single_statement_on_single_line(ast: &Ast, node: NodeId) -> bool {
    Some(node)
        .filter(|&it| ast.element_type(it) == FUNCTION_LITERAL)
        .filter(|&it| !ast.text_contains(it, '\n'))
        .and_then(|it| ast.find_child_by_type(it, BLOCK))
        .map(|block| ast.children(block).filter(|&it| !matches!(ast.element_type(it), VALUE_PARAMETER_LIST | ARROW)).count())
        .is_some_and(|count| count <= 1)
}

fn is_enum_class_on_single_line(ast: &Ast, node: NodeId) -> bool {
    if is_enum_class(ast, node) {
        let last_child_leaf = ast.last_child_leaf_or_self(node);
        // Ignore the leading comment
        let first_code_leaf = first_code_leaf_or_null(ast, node).expect("NullPointerException: firstCodeLeafOrNull!!");
        ast.no_new_line_in_closed_range(first_code_leaf, last_child_leaf)
    } else {
        false
    }
}

/// The first leaf of the modifier list after the annotations, which skips the comment on top of the node.
fn first_code_leaf_or_null(ast: &Ast, node: NodeId) -> Option<NodeId> {
    let modifier_list = ast.find_child_by_type(node, MODIFIER_LIST)?;
    ast.children(modifier_list)
        .find(|&it| !(ast.element_type(it) == ANNOTATION_ENTRY || ast.is_white_space(it)))
        .map(|it| ast.first_child_leaf_or_self(it))
}

fn is_enum_class(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == CLASS && ast.has_modifier(node, ENUM_KEYWORD)
}

fn indent_as_parent(ast: &Ast, node: NodeId) -> String {
    ast.indent(ast.parent(node).expect("NullPointerException: parent!!"))
}

fn visit_semi_colon(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let Some(previous_code_leaf) = ast.prev_code_leaf(node).map(|it| ast.last_child_leaf_or_self(it)) else { return };
    let Some(next_code_leaf) = ast.next_code_leaf(node).map(|it| ast.first_child_leaf_or_self(it)) else { return };
    if ast.parent(previous_code_leaf).map(|p| ast.element_type(p)) == Some(ENUM_ENTRY) && ast.element_type(next_code_leaf) == RBRACE {
        // Allow `enum class INDEX2 { ONE, TWO, THREE; }`
        return;
    }
    if ast.no_new_line_in_closed_range(previous_code_leaf, next_code_leaf) {
        let message = format!("Missing newline after '{}'", ast.text(node));
        emit(ast, ast.start_offset(node) + 1, &message, true).if_autocorrect_allowed(|| {
            let indent = ast.indent(previous_code_leaf);
            ast.upsert_whitespace_after_me(node, &indent);
        });
    }
}
