//! Port of ktlint-ruleset-standard `SpacingAroundAngleBracketsRule.kt` (id `spacing-around-angle-brackets`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{self, FUN_KEYWORD, TYPE_ARGUMENT_LIST, TYPE_PARAMETER_LIST, VAL_KEYWORD, VAR_KEYWORD};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2, TokenSet};
use crate::rules::STANDARD_RULE_ABOUT;

const VISITED_TYPES: TokenSet = TokenSet::create(&[TYPE_PARAMETER_LIST, TYPE_ARGUMENT_LIST]);

const ELEMENT_TYPES_ALLOWING_PRECEDING_WHITESPACE: [SyntaxKind; 3] = [VAL_KEYWORD, VAR_KEYWORD, FUN_KEYWORD];

pub struct SpacingAroundAngleBracketsRule;

impl RuleV2 for SpacingAroundAngleBracketsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:spacing-around-angle-brackets")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) != TYPE_PARAMETER_LIST && ast.element_type(node) != TYPE_ARGUMENT_LIST {
            return;
        }

        if let Some(opening_bracket) = ast.first_child_node(node) {
            // Check for rogue spacing before an opening bracket, e.g. Map <String, Int>
            if let Some(before_left_angle) = ast.prev_leaf(opening_bracket).filter(|&it| ast.is_white_space(it)) {
                // Ignore when the whitespace is preceded by certain keywords, e.g. fun <T> func(arg: T) {}
                let prev_type = ast.prev_leaf(before_left_angle).map(|it| ast.element_type(it));
                if !prev_type.is_some_and(|t| ELEMENT_TYPES_ALLOWING_PRECEDING_WHITESPACE.contains(&t)) {
                    emit(ast, ast.start_offset(before_left_angle), "Unexpected spacing before \"<\"", true)
                        .if_autocorrect_allowed(|| ast.remove(before_left_angle));
                }
            }

            // Check for rogue spacing after an opening bracket
            if let Some(after_left_angle) = ast.next_leaf(opening_bracket).filter(|&it| ast.is_white_space(it)) {
                if ast.is_white_space_without_newline(after_left_angle) {
                    // when spacing does not include any new lines, e.g. Map< String, Int>
                    emit(ast, ast.start_offset(after_left_angle), "Unexpected spacing after \"<\"", true)
                        .if_autocorrect_allowed(|| ast.remove(after_left_angle));
                } else {
                    // when spacing contains at least one new line, e.g. `SomeGenericType<[whitespace]\n\n   String, Int>`
                    // gets converted to `SomeGenericType<\n   String, Int>`
                    let new_line_with_indent = trim_before_last_line(ast.leaf_text(after_left_angle)).to_owned();
                    if new_line_with_indent != ast.leaf_text(after_left_angle) {
                        emit(ast, ast.start_offset(after_left_angle), "Single newline expected after \"<\"", true)
                            .if_autocorrect_allowed(|| ast.replace_text_with(after_left_angle, &new_line_with_indent));
                    }
                }
            }
        }

        let closing_bracket = ast.last_child_node(node);
        if let Some(before_right_angle) = closing_bracket.and_then(|it| ast.prev_leaf(it)).filter(|&it| ast.is_white_space(it)) {
            // Check for rogue spacing before a closing bracket
            if ast.is_white_space_without_newline(before_right_angle) {
                // when spacing does not include any new lines, e.g. Map<String, Int >
                emit(ast, ast.start_offset(before_right_angle), "Unexpected spacing before \">\"", true)
                    .if_autocorrect_allowed(|| ast.remove(before_right_angle));
            } else {
                // when spacing contains at least one new line, e.g. `SomeGenericType<String[whitespace]\n\n   >`
                // gets converted to `SomeGenericType<String\n   >`
                let new_line_with_indent = trim_before_last_line(ast.leaf_text(before_right_angle)).to_owned();
                if new_line_with_indent != ast.leaf_text(before_right_angle) {
                    emit(ast, ast.start_offset(before_right_angle), "Single newline expected before \">\"", true)
                        .if_autocorrect_allowed(|| ast.replace_text_with(before_right_angle, &new_line_with_indent));
                }
            }
        }
    }
}

fn trim_before_last_line(text: &str) -> &str {
    &text[text.rfind('\n').expect("StringIndexOutOfBoundsException: begin -1")..]
}
