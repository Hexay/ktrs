//! Port of ktlint-ruleset-standard `StringTemplateRule.kt`. Upstream keeps a disabled "redundant string template"
//! check commented out (`"$x"` -> `x.toString()` is not clearly better).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    CALL_EXPRESSION, CLOSING_QUOTE, DOT_QUALIFIED_EXPRESSION, LITERAL_STRING_TEMPLATE_ENTRY, LONG_STRING_TEMPLATE_ENTRY,
    LONG_TEMPLATE_ENTRY_END, LONG_TEMPLATE_ENTRY_START, PROPERTY, REFERENCE_EXPRESSION, SHORT_STRING_TEMPLATE_ENTRY, STRING_TEMPLATE,
    SUPER_EXPRESSION, THIS_EXPRESSION,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::internal::kotlin_string::is_letter_or_digit;

pub struct StringTemplateRule;

impl RuleV2 for StringTemplateRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:string-template")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == LONG_STRING_TEMPLATE_ENTRY {
            remove_redundant_to_string(ast, node, emit);
            check_for_redundant_curly_braces(ast, node, emit);
        }
    }
}

fn remove_redundant_to_string(ast: &mut Ast, this: NodeId, emit: &mut Emit<'_>) {
    let Some(dot_qualified_expression) = ast.find_child_by_type(this, DOT_QUALIFIED_EXPRESSION) else { return };
    let (receiver, dot, selector) = split_code_children(ast, dot_qualified_expression);
    if ast.element_type(receiver) != SUPER_EXPRESSION
        && ast.element_type(selector) == CALL_EXPRESSION
        && ast.text(selector) == "toString()"
    {
        emit(ast, ast.start_offset(dot), "Redundant \".toString()\" call in string template", true).if_autocorrect_allowed(|| {
            if let Some(parent) = ast.parent(dot_qualified_expression) {
                ast.add_child(parent, receiver, Some(dot_qualified_expression));
            }
            ast.remove(dot_qualified_expression);
            if is_string_template(ast, this) {
                remove_curly_braces_if_redundant(ast, this);
            }
        });
    }
}

fn split_code_children(ast: &Ast, this: NodeId) -> (NodeId, NodeId, NodeId) {
    assert!(ast.element_type(this) == DOT_QUALIFIED_EXPRESSION, "IllegalArgumentException: Failed requirement.");
    let children: Vec<NodeId> = ast.children(this).filter(|&it| ast.is_code(it)).collect();
    assert!(children.len() == 3, "IllegalArgumentException: Failed requirement.");
    (children[0], children[1], children[2])
}

fn check_for_redundant_curly_braces(ast: &mut Ast, this: NodeId, emit: &mut Emit<'_>) {
    if !is_string_template(ast, this) {
        return;
    }
    let redundant = ast
        .children(this)
        .find(|&it| ast.element_type(it) != LONG_TEMPLATE_ENTRY_START)
        .is_some_and(|it| matches!(ast.element_type(it), REFERENCE_EXPRESSION | THIS_EXPRESSION))
        && ast.next_sibling(this).is_some_and(|next_sibling| {
            ast.element_type(next_sibling) == CLOSING_QUOTE
                || (ast.element_type(next_sibling) == LITERAL_STRING_TEMPLATE_ENTRY
                    && !is_part_of_identifier(&first_unit(&ast.text(next_sibling))))
        });
    if redundant {
        let prev_sibling = ast.prev_sibling(this).expect("NullPointerException: prevSibling");
        emit(ast, ast.start_offset(prev_sibling) + 2, "Redundant curly braces", true)
            .if_autocorrect_allowed(|| remove_curly_braces_if_redundant(ast, this));
    }
}

/// `text.substring(0, 1)`: the first UTF-16 unit.
fn first_unit(text: &str) -> Vec<u16> {
    let first = text.encode_utf16().next().expect("StringIndexOutOfBoundsException: begin 0, end 1, length 0");
    vec![first]
}

fn remove_curly_braces_if_redundant(ast: &mut Ast, this: NodeId) {
    if is_string_template(ast, this) {
        if let Some(start) = ast.find_child_by_type(this, LONG_TEMPLATE_ENTRY_START) {
            ast.remove(start);
        }
        if let Some(end) = ast.find_child_by_type(this, LONG_TEMPLATE_ENTRY_END) {
            ast.remove(end);
        }
        let first_child_node = ast.first_child_node(this).expect("NullPointerException: firstChildNode");
        let short_string_template_node = to_short_string_template_node(ast, first_child_node);
        let first_child_node = ast.first_child_node(this).expect("NullPointerException: firstChildNode");
        ast.replace_child(this, first_child_node, short_string_template_node);
    }
}

/// `text.startsWith("${") && text.substring(2, text.length - 1).isPartOfIdentifier()`, in UTF-16 units.
fn is_string_template(ast: &Ast, this: NodeId) -> bool {
    let text = ast.text(this);
    if !text.starts_with("${") {
        return false;
    }
    let units: Vec<u16> = text.encode_utf16().collect();
    assert!(units.len() >= 3, "StringIndexOutOfBoundsException: begin 2, end {}, length {}", units.len() as i64 - 1, units.len());
    is_part_of_identifier(&units[2..units.len() - 1])
}

fn is_part_of_identifier(this: &[u16]) -> bool {
    this == [u16::from(b'_')] || this.iter().all(|&it| is_letter_or_digit(it))
}

fn to_short_string_template_node(ast: &mut Ast, this: NodeId) -> NodeId {
    let text = ast.text(this);
    ast.create_ast_node_from_text(&format!("val foo = \"${text}\""))
        .and_then(|it| ast.find_child_by_type(it, PROPERTY))
        .and_then(|it| ast.find_child_by_type(it, STRING_TEMPLATE))
        .and_then(|it| ast.find_child_by_type(it, SHORT_STRING_TEMPLATE_ENTRY))
        .unwrap_or_else(|| panic!("IllegalStateException: Cannot create short string template for string '{text}"))
}
