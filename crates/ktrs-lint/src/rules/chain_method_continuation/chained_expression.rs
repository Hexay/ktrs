//! `ChainMethodContinuationRule.ChainedExpression` and its companion object.

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{
    ARRAY_ACCESS_EXPRESSION, CALL_EXPRESSION, DOT_QUALIFIED_EXPRESSION, POSTFIX_EXPRESSION, PREFIX_EXPRESSION, SAFE_ACCESS_EXPRESSION,
};

use super::CHAIN_OPERATOR_TOKEN_SET;
use crate::ast_node_extension::AstNodeExtension;

const CHAINABLE_ELEMENT_TYPES: TokenSet = TokenSet::create(&[
    ARRAY_ACCESS_EXPRESSION,
    CALL_EXPRESSION,
    DOT_QUALIFIED_EXPRESSION,
    POSTFIX_EXPRESSION,
    PREFIX_EXPRESSION,
    SAFE_ACCESS_EXPRESSION,
]);

/// The chain operators of one expression, which live at different levels of the AST, flattened.
#[derive(Clone, Debug)]
pub(super) struct ChainedExpression {
    pub root_ast_node: NodeId,
    pub chain_operators: Vec<NodeId>,
    pub has_newline_before_first_chain_operator: bool,
    pub has_newline_between_first_and_last_chain_operator: bool,
    pub has_newline_after_last_chain_operator: bool,
}

impl ChainedExpression {
    pub fn create_from(ast: &Ast, ast_node: NodeId) -> ChainedExpression {
        assert!(CHAIN_OPERATOR_TOKEN_SET.contains(ast.element_type(ast_node)), "IllegalArgumentException: Failed requirement.");
        let mut chain_parent = ast.parent(ast_node).expect("IllegalArgumentException: Required value was null.");
        while let Some(parent) = ast.parent(chain_parent).filter(|&p| CHAINABLE_ELEMENT_TYPES.contains(ast.element_type(p))) {
            chain_parent = parent;
        }
        to_chained_expression(ast, chain_parent).unwrap_or_else(|| {
            let text = ast.parent(ast_node).map(|p| ast.text(p)).unwrap_or_else(|| "null".to_owned());
            panic!("IllegalArgumentException: Failed to create chained expression from {text}")
        })
    }
}

fn to_chained_expression(ast: &Ast, node: NodeId) -> Option<ChainedExpression> {
    match ast.element_type(node) {
        DOT_QUALIFIED_EXPRESSION | SAFE_ACCESS_EXPRESSION => {
            let chain_operator = ast.children(node).find(|&it| CHAIN_OPERATOR_TOKEN_SET.contains(ast.element_type(it)))?;
            let chained_expression = create_base_chained_expression(ast, node, chain_operator);
            Some(match chained_expression.chain_operators.len() {
                1 => modify_for_first_operator(ast, chained_expression, chain_operator),
                2 => modify_for_second_operator(ast, chained_expression),
                _ => chained_expression,
            })
        }
        CALL_EXPRESSION | ARRAY_ACCESS_EXPRESSION | PREFIX_EXPRESSION | POSTFIX_EXPRESSION => {
            let mut chained = ast.children(node).filter_map(|it| to_chained_expression(ast, it));
            let first = chained.next()?;
            chained.next().is_none().then_some(first)
        }
        _ => None,
    }
}

fn create_base_chained_expression(ast: &Ast, node: NodeId, chain_operator: NodeId) -> ChainedExpression {
    let chain_before = ast.prev_code_sibling(chain_operator).and_then(|it| to_chained_expression(ast, it));
    let chain_after = ast.next_code_sibling(chain_operator).and_then(|it| to_chained_expression(ast, it));
    let newline_after = contains_newline(chain_after.as_ref())
        || ast.text_contains(ast.next_code_sibling(chain_operator).expect("NullPointerException: nextCodeSibling!!"), '\n')
        || ast.next_sibling_matching(chain_operator, |it| ast.is_white_space_with_newline(it)).is_some();
    let mut chain_operators = Vec::new();
    chain_operators.extend(chain_before.iter().flat_map(|it| it.chain_operators.iter().copied()));
    chain_operators.push(chain_operator);
    chain_operators.extend(chain_after.iter().flat_map(|it| it.chain_operators.iter().copied()));
    let newline_before = chain_before.as_ref().is_some_and(|it| it.has_newline_between_first_and_last_chain_operator)
        || chain_before.as_ref().is_some_and(|it| it.has_newline_after_last_chain_operator)
        || is_preceded_by_newline_sibling(ast, chain_operator);
    ChainedExpression {
        root_ast_node: node,
        chain_operators,
        has_newline_before_first_chain_operator: chain_before.as_ref().is_some_and(|it| it.has_newline_before_first_chain_operator),
        has_newline_between_first_and_last_chain_operator: newline_before,
        has_newline_after_last_chain_operator: newline_after,
    }
}

fn modify_for_first_operator(ast: &Ast, chained_expression: ChainedExpression, chain_operator: NodeId) -> ChainedExpression {
    let prev_code_sibling = ast.prev_code_sibling(chain_operator).expect("NullPointerException: prevCodeSibling!!");
    ChainedExpression {
        has_newline_before_first_chain_operator: ast.text_contains(prev_code_sibling, '\n')
            || is_preceded_by_newline_sibling(ast, chain_operator),
        has_newline_between_first_and_last_chain_operator: false,
        ..chained_expression
    }
}

/// A newline inside the expression before the first chain operator is ignored, one between its last leaf and the chain
/// operator is not. Allows `"""\nsome text\n""".uppercase().trimIndent()`.
fn modify_for_second_operator(ast: &Ast, chained_expression: ChainedExpression) -> ChainedExpression {
    if chained_expression.has_newline_between_first_and_last_chain_operator {
        chained_expression
    } else {
        let first = chained_expression.chain_operators[0];
        ChainedExpression {
            has_newline_between_first_and_last_chain_operator: is_preceded_by_newline_sibling(ast, first),
            ..chained_expression
        }
    }
}

fn is_preceded_by_newline_sibling(ast: &Ast, node: NodeId) -> bool {
    ast.prev_sibling_matching(node, |it| ast.is_white_space_with_newline(it)).is_some()
}

fn contains_newline(chained_expression: Option<&ChainedExpression>) -> bool {
    chained_expression.is_some_and(|it| {
        it.has_newline_before_first_chain_operator
            || it.has_newline_between_first_and_last_chain_operator
            || it.has_newline_after_last_chain_operator
    })
}
