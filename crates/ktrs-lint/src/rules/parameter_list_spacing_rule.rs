//! Port of ktlint-ruleset-standard `ParameterListSpacingRule.kt` (id `parameter-list-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    ANNOTATION_ENTRY, COLON, COMMA, MODIFIER_LIST, RPAR, TYPE_REFERENCE, VALUE_PARAMETER, VALUE_PARAMETER_LIST, WHITE_SPACE,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::{MAX_LINE_LENGTH_PROPERTY, PropertyRef};
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;
use crate::rules::max_line_length_rule::max_line_length;

const VISITED_TYPES: TokenSet = TokenSet::create(&[VALUE_PARAMETER_LIST]);

/// Ensures consistent spacing inside the parameter list. This rule partly overlaps with other rules like spacing around
/// commas and colons. However, it does have a more complete view on the higher concept of the parameter-list without
/// interfering of the parameter-list-wrapping rule.
pub struct ParameterListSpacingRule {
    max_line_length: i32,
}

impl ParameterListSpacingRule {
    pub fn new() -> ParameterListSpacingRule {
        ParameterListSpacingRule { max_line_length: MAX_LINE_LENGTH_PROPERTY.default_value }
    }
}

impl Default for ParameterListSpacingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for ParameterListSpacingRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:parameter-list-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![PropertyRef::from(&*MAX_LINE_LENGTH_PROPERTY)]
    }

    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.max_line_length = max_line_length(editor_config);
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == VALUE_PARAMETER_LIST {
            self.visit_value_parameter_list(ast, node, emit);
        }
    }
}

impl ParameterListSpacingRule {
    fn visit_value_parameter_list(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        require(ast.element_type(node) == VALUE_PARAMETER_LIST);
        let count_value_parameters = ast.children(node).filter(|&it| ast.element_type(it) == VALUE_PARAMETER).count();
        let mut value_parameter_count = 0;
        // Store elements in list before changing them as otherwise only one element is being changed
        let children: Vec<NodeId> = ast.children(node).collect();
        for el in children {
            match ast.element_type(el) {
                WHITE_SPACE => {
                    if count_value_parameters == 0 && contains_no_comments(ast, node) {
                        remove_unexpected_white_space(ast, el, emit);
                    } else if value_parameter_count == 0 && is_not_indent(ast, el) {
                        // whitespace before first parameter; with comments, avoid conflict with comment spacing rule which
                        // requires a whitespace before the EOL-comment
                        if contains_no_comments(ast, node) {
                            remove_unexpected_white_space(ast, el, emit);
                        }
                    } else if value_parameter_count == count_value_parameters && is_not_indent(ast, el) {
                        // whitespace after the last parameter; with comments, as above
                        if contains_no_comments(ast, node) {
                            remove_unexpected_white_space(ast, el, emit);
                        }
                    } else if ast.next_code_sibling(el).map(|it| ast.element_type(it)) == Some(COMMA) {
                        // No whitespace between parameter name and comma allowed
                        remove_unexpected_white_space(ast, el, emit);
                    } else if ast.is_white_space(el) && is_not_indent(ast, el) && is_not_single_space(ast, el) {
                        require(ast.prev_code_sibling(el).map(|it| ast.element_type(it)) == Some(COMMA));
                        replace_with_single_space(ast, el, emit);
                    }
                }
                COMMA => {
                    // Comma, except when it is the trailing comma, must be followed by whitespace
                    if ast.next_leaf(el).is_some_and(|it| !(ast.is_white_space(it) || ast.element_type(it) == RPAR)) {
                        add_missing_white_space_after_me(ast, el, emit);
                    }
                }
                VALUE_PARAMETER => {
                    value_parameter_count += 1;
                    self.visit_value_parameter(ast, el, emit);
                }
                _ => {}
            }
        }
    }

    fn visit_value_parameter(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        visit_modifier_list(ast, node, emit);
        remove_white_space_between_parameter_identifier_and_colon(ast, node, emit);
        self.fix_white_space_after_colon_in_parameter(ast, node, emit);
    }

    fn fix_white_space_after_colon_in_parameter(&self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let Some(colon_node) = ast.find_child_by_type(node, COLON) else { return };
        match ast.next_leaf(colon_node).filter(|&it| ast.is_white_space(it)) {
            None => add_missing_white_space_after_me(ast, colon_node, emit),
            Some(white_space_after_colon) => {
                if is_type_reference_with_modifier_list(ast, Some(node)) && is_indent(ast, white_space_after_colon) {
                    // Allow the type to be wrapped to the next line when it has a modifier:
                    //   data class Foo(
                    //       val bar:
                    //           @FooBar("foobar")
                    //           Bar,
                    //   )
                } else if self.has_type_reference_which_does_not_fit_on_same_line_as_colon(ast, white_space_after_colon) {
                    // Allow the type to be wrapped to the next line when the type does not fit on same line as colon:
                    //   class Foo(
                    //       val someReallyLongFieldNameUsedInMyClass:
                    //           SomeReallyLongDependencyClass
                    //   )
                } else if is_not_single_space(ast, white_space_after_colon) {
                    replace_with_single_space(ast, white_space_after_colon, emit);
                }
            }
        }
    }

    fn has_type_reference_which_does_not_fit_on_same_line_as_colon(&self, ast: &Ast, node: NodeId) -> bool {
        let Some(type_reference) = Some(node)
            .filter(|&it| ast.is_white_space_with_newline(it))
            .and_then(|it| ast.next_code_sibling(it))
            .filter(|&it| ast.element_type(it) == TYPE_REFERENCE)
        else {
            return false;
        };
        let text = ast.leaf_text(node);
        let indent = &text[text.rfind('\n').map_or(0, |i| i + 1)..];
        // length of the previous line + single space before type reference - length of current indent before
        // typeReference + length of line containing typeReference
        let length = ast.line_length(ast.drop_trailing_eol_comment(ast.leaves_on_line(node))) as i64 + 1
            - indent.encode_utf16().count() as i64
            + ast.line_length(ast.drop_trailing_eol_comment(ast.leaves_on_line(type_reference))) as i64;
        length > self.max_line_length as i64
    }
}

fn require(value: bool) {
    assert!(value, "IllegalArgumentException: Failed requirement.");
}

fn contains_no_comments(ast: &Ast, node: NodeId) -> bool {
    !ast.children(node).any(|it| ast.is_part_of_comment(it))
}

fn visit_modifier_list(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    let Some(modifier_list) = ast.find_child_by_type(node, MODIFIER_LIST) else { return };
    remove_white_space_between_modifiers_in_list(ast, modifier_list, emit);
    remove_white_space_between_modifier_list_and_parameter_identifier(ast, modifier_list, emit);
}

fn remove_white_space_between_modifiers_in_list(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    require(ast.element_type(node) == MODIFIER_LIST);
    // Store elements in the list before changing them as otherwise only the first whitespace is being changed
    let white_spaces: Vec<NodeId> = ast.children(node).filter(|&it| ast.is_white_space(it)).collect();
    for it in white_spaces {
        visit_white_space_after_modifier(ast, it, emit);
    }
}

fn visit_white_space_after_modifier(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    // Ignore when the modifier is an annotation which is placed on a separate line
    let is_annotation_on_separate_line =
        is_indent(ast, node) && get_preceding_modifier(ast, node).map(|it| ast.element_type(it)) == Some(ANNOTATION_ENTRY);
    if !is_annotation_on_separate_line && is_not_single_space(ast, node) {
        replace_with_single_space(ast, node, emit);
    }
}

fn remove_white_space_between_modifier_list_and_parameter_identifier(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    require(ast.element_type(node) == MODIFIER_LIST);
    if let Some(it) = ast.next_sibling(node).filter(|&it| ast.is_white_space(it)) {
        visit_white_space_after_modifier(ast, it, emit);
    }
}

fn remove_white_space_between_parameter_identifier_and_colon(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    if let Some(white_space_before_colon) =
        ast.find_child_by_type(node, COLON).and_then(|it| ast.prev_leaf(it)).filter(|&it| ast.is_white_space(it))
    {
        remove_unexpected_white_space(ast, white_space_before_colon, emit);
    }
}

fn add_missing_white_space_after_me(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    require(ast.element_type(node) == COLON || ast.element_type(node) == COMMA);
    let message = format!("Whitespace after '{}' is missing", ast.text(node));
    emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(node, " "));
}

fn is_not_indent(ast: &Ast, node: NodeId) -> bool {
    !is_indent(ast, node)
}

fn is_indent(ast: &Ast, node: NodeId) -> bool {
    ast.is_white_space_with_newline(node)
}

fn is_not_single_space(ast: &Ast, node: NodeId) -> bool {
    require(ast.is_white_space(node));
    ast.leaf_text(node) != " "
}

fn remove_unexpected_white_space(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    emit(ast, ast.start_offset(node), "Unexpected whitespace", true).if_autocorrect_allowed(|| ast.raw_remove(node));
}

fn replace_with_single_space(ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
    emit(ast, ast.start_offset(node), "Expected a single space", true).if_autocorrect_allowed(|| ast.replace_text_with(node, " "));
}

fn get_preceding_modifier(ast: &Ast, node: NodeId) -> Option<NodeId> {
    ast.prev_code_sibling(node).and_then(|prev_code_sibling| {
        if ast.element_type(prev_code_sibling) == MODIFIER_LIST {
            ast.last_child_node(prev_code_sibling)
        } else {
            require(ast.parent(prev_code_sibling).map(|p| ast.element_type(p)) == Some(MODIFIER_LIST));
            Some(prev_code_sibling)
        }
    })
}

fn is_type_reference_with_modifier_list(ast: &Ast, node: Option<NodeId>) -> bool {
    node.and_then(|it| ast.find_child_by_type(it, TYPE_REFERENCE)).and_then(|it| ast.find_child_by_type(it, MODIFIER_LIST)).is_some()
}
