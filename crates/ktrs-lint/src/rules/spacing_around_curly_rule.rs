//! Port of ktlint-ruleset-standard `SpacingAroundCurlyRule.kt` (id `curly-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{
    AT, BLOCK, CLASS_BODY, COLONCOLON, COMMA, DOT, EXCLEXCL, LAMBDA_EXPRESSION, LBRACE, LBRACKET, LPAR, RANGE, RANGE_UNTIL,
    RBRACE, RBRACKET, RPAR, SAFE_ACCESS, SEMICOLON,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::{AstNodeExtension, AstNodeLines};
use crate::editorconfig::{CODE_STYLE_PROPERTY, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY, PropertyRef};
use crate::indent_config::IndentConfig;
use crate::rule::{About, EditorConfig, Emit, RuleId, RuleV2, TraversalState};
use crate::rules::STANDARD_RULE_ABOUT;

pub struct SpacingAroundCurlyRule {
    indent_config: IndentConfig,
    traversal: TraversalState,
}

impl SpacingAroundCurlyRule {
    pub fn new() -> SpacingAroundCurlyRule {
        SpacingAroundCurlyRule { indent_config: IndentConfig::default_indent_config(), traversal: TraversalState::default() }
    }
}

impl Default for SpacingAroundCurlyRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleV2 for SpacingAroundCurlyRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:curly-spacing")
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn uses_editor_config_properties(&self) -> Vec<PropertyRef> {
        vec![
            PropertyRef::from(&*CODE_STYLE_PROPERTY),
            PropertyRef::from(&*INDENT_SIZE_PROPERTY),
            PropertyRef::from(&*INDENT_STYLE_PROPERTY),
        ]
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal)
    }

    // Upstream also reads `codeStyle` here but never uses it.
    fn before_first_node(&mut self, editor_config: &EditorConfig) {
        self.indent_config = IndentConfig::new(editor_config.get(&INDENT_STYLE_PROPERTY), editor_config.get(&INDENT_SIZE_PROPERTY));
        if self.indent_config.disabled() {
            self.traversal.stop_traversal_of_ast();
        }
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if !(ast.is_leaf(node) && !ast.is_part_of_string(node)) {
            return;
        }
        let prev_leaf = ast.prev_leaf(node);
        let next_leaf = ast.next_leaf(node);
        let type_of = |ast: &Ast, n: Option<NodeId>| n.map(|n| ast.element_type(n));
        let spacing_before: bool;
        let spacing_after: bool;
        match ast.element_type(node) {
            LBRACE => {
                spacing_before = ast.is_white_space(prev_leaf)
                    || type_of(ast, prev_leaf) == Some(AT)
                    || ((type_of(ast, prev_leaf) == Some(LPAR) || type_of(ast, prev_leaf) == Some(LBRACKET))
                        && (type_of(ast, ast.parent(node)) == Some(LAMBDA_EXPRESSION)
                            || type_of(ast, ast.parent(node).and_then(|p| ast.parent(p))) == Some(LAMBDA_EXPRESSION)));
                spacing_after = ast.is_white_space(next_leaf) || type_of(ast, next_leaf) == Some(RBRACE);
                if ast.is_white_space_without_newline(prev_leaf) {
                    let prev_leaf = prev_leaf.expect("NullPointerException: prevLeaf!!");
                    if is_preceded_by(ast, prev_leaf, |it| matches!(ast.element_type(it), LPAR | AT)) {
                        let message = format!("Unexpected space before \"{}\"", ast.text(node));
                        emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| ast.remove(prev_leaf));
                    }
                }
                if let Some(prev_leaf) = prev_leaf
                    && has_unexpected_newline_before_lbrace(ast, node)
                {
                    let message = format!("Unexpected newline before \"{}\"", ast.text(node));
                    emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| {
                        if is_preceded_by_eol_comment(ast, prev_leaf) {
                            // All consecutive whitespaces and comments preceding the curly have to be moved after the curly brace
                            let mut leaves_to_move_after_curly: Vec<NodeId> =
                                ast.leaves_backwards_including_self(prev_leaf).take_while(|&it| !ast.is_code(it)).collect();
                            leaves_to_move_after_curly.reverse();
                            if let (Some(&first), Some(&last)) = (leaves_to_move_after_curly.first(), leaves_to_move_after_curly.last())
                                && let Some(parent) = ast.parent(node)
                            {
                                let anchor = ast.next_sibling(node);
                                ast.add_children(parent, first, Some(last), anchor);
                            }
                        }
                        ast.replace_text_with(prev_leaf, " ");
                    });
                }
            }
            RBRACE => {
                spacing_before = ast.is_white_space(prev_leaf) || type_of(ast, prev_leaf) == Some(LBRACE);
                spacing_after =
                    next_leaf.is_none() || ast.is_white_space(next_leaf) || should_not_to_be_separated_by_space(ast, next_leaf);
                if let Some(leaf) = next_leaf
                    .filter(|&it| ast.is_white_space_without_newline(it))
                    .filter(|&it| should_not_to_be_separated_by_space(ast, ast.next_leaf(it)))
                {
                    let message = format!("Unexpected space after \"{}\"", ast.text(node));
                    emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| ast.remove(leaf));
                }
            }
            _ => return,
        }
        if !spacing_before && !spacing_after {
            let message = format!("Missing spacing around \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| {
                ast.upsert_whitespace_before_me(node, " ");
                ast.upsert_whitespace_after_me(node, " ");
            });
        } else if !spacing_before {
            let message = format!("Missing spacing before \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| ast.upsert_whitespace_before_me(node, " "));
        } else if !spacing_after {
            let message = format!("Missing spacing after \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(node) + 1, &message, true).if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(node, " "));
        }
    }
}

fn has_unexpected_newline_before_lbrace(ast: &Ast, node: NodeId) -> bool {
    assert!(ast.element_type(node) == LBRACE, "IllegalArgumentException: Failed requirement.");
    ast.is_white_space_with_newline(ast.prev_leaf(node))
        && ast.parent(node).is_some_and(|p| matches!(ast.element_type(p), CLASS_BODY | BLOCK))
}

fn is_preceded_by(ast: &Ast, node: NodeId, predicate: impl Fn(NodeId) -> bool) -> bool {
    ast.prev_leaf(node).is_some_and(predicate)
}

fn is_preceded_by_eol_comment(ast: &Ast, node: NodeId) -> bool {
    ast.prev_leaf(node).is_some_and(|it| ast.is_part_of_comment(it))
}

fn should_not_to_be_separated_by_space(ast: &Ast, leaf: Option<NodeId>) -> bool {
    leaf.is_some_and(|it| {
        matches!(
            ast.element_type(it),
            DOT | COMMA | RBRACKET | RPAR | SEMICOLON | SAFE_ACCESS | EXCLEXCL | LBRACKET | LPAR | COLONCOLON | RANGE | RANGE_UNTIL
        )
    })
}
