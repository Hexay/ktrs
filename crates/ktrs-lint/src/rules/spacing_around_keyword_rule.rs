//! Port of ktlint-ruleset-standard `SpacingAroundKeywordRule.kt` (id `keyword-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{
    BLOCK, CATCH_KEYWORD, DO_KEYWORD, ELSE_KEYWORD, FINALLY_KEYWORD, FOR_KEYWORD, GET_KEYWORD, IF_KEYWORD, KDOC_NAME,
    PROPERTY_ACCESSOR, RBRACE, SET_KEYWORD, TRY_KEYWORD, VALUE_PARAMETER_LIST, WHEN_ENTRY, WHEN_KEYWORD, WHILE_KEYWORD,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

const NO_LF_BEFORE_SET: TokenSet = TokenSet::create(&[ELSE_KEYWORD, CATCH_KEYWORD, FINALLY_KEYWORD]);
const TOKEN_SET: TokenSet = TokenSet::create(&[
    CATCH_KEYWORD, DO_KEYWORD, ELSE_KEYWORD, FINALLY_KEYWORD, FOR_KEYWORD, IF_KEYWORD, TRY_KEYWORD, WHEN_KEYWORD, WHILE_KEYWORD,
]);
const KEYWORDS_WITHOUT_SPACES: TokenSet = TokenSet::create(&[GET_KEYWORD, SET_KEYWORD]);
const VISITED_TYPES: TokenSet = TokenSet::or_set(&[TOKEN_SET, KEYWORDS_WITHOUT_SPACES, NO_LF_BEFORE_SET]);

pub struct SpacingAroundKeywordRule;

impl RuleV2 for SpacingAroundKeywordRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:keyword-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let element_type = ast.element_type(node);
        let parent_type = |ast: &Ast| ast.parent(node).map(|p| ast.element_type(p));
        if TOKEN_SET.contains(element_type) && parent_type(ast) != Some(KDOC_NAME) && !ast.is_white_space(ast.next_leaf(node)) {
            let message = format!("Missing spacing after \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(node) + ast.text_length(node), &message, true)
                .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(node, " "));
        }
        if KEYWORDS_WITHOUT_SPACES.contains(ast.element_type(node))
            && is_property_accessor_with_value_parameter_list(ast, node)
            && let Some(next_leaf) = ast.next_leaf(node).filter(|&it| ast.is_white_space(it))
        {
            let message = format!("Unexpected spacing after \"{}\"", ast.text(node));
            emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| ast.remove(next_leaf));
        }
        if NO_LF_BEFORE_SET.contains(ast.element_type(node)) {
            let prev_leaf = ast.prev_leaf(node);
            let is_else_keyword = ast.element_type(node) == ELSE_KEYWORD;
            if ast.is_white_space_with_newline(prev_leaf) && (!is_else_keyword || parent_type(ast) != Some(WHEN_ENTRY)) {
                let r_brace = prev_leaf.and_then(|it| ast.prev_leaf(it)).filter(|&it| ast.element_type(it) == RBRACE);
                let block = r_brace
                    .and_then(|it| ast.parent(it))
                    .filter(|&it| ast.element_type(it) == BLOCK)
                    .filter(|&it| !is_else_keyword || ast.parent(it).and_then(|p| ast.parent(p)) == ast.parent(node));
                if block.is_some() {
                    let prev_leaf = prev_leaf.expect("prevLeaf is a whitespace");
                    let message = format!("Unexpected newline before \"{}\"", ast.text(node));
                    emit(ast, ast.start_offset(node), &message, true).if_autocorrect_allowed(|| ast.replace_text_with(prev_leaf, " "));
                }
            }
        }
    }
}

fn is_property_accessor_with_value_parameter_list(ast: &Ast, node: NodeId) -> bool {
    ast.parent(node)
        .filter(|&it| ast.element_type(it) == PROPERTY_ACCESSOR)
        .and_then(|it| ast.find_child_by_type(it, VALUE_PARAMETER_LIST))
        .is_some()
}
