//! Port of ktlint-ruleset-standard `SpacingAroundOperatorsRule.kt` (id `op-spacing`).

use ktrs_ast::{Ast, NodeId};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::{
    ANDAND, ARROW, DIV, DIVEQ, ELVIS, EQ, EQEQ, EQEQEQ, EXCLEQ, EXCLEQEQEQ, GT, GTEQ, IDENTIFIER, IMPORT_DIRECTIVE, LT, LTEQ,
    MINUS, MINUSEQ, MUL, MULTEQ, OPERATION_REFERENCE, OROR, PERC, PERCEQ, PLUS, PLUSEQ, PREFIX_EXPRESSION, VALUE_ARGUMENT,
};

use crate::ast_node_edit::AstNodeEdit;
use crate::ast_node_extension::AstNodeExtension;
use crate::rule::{About, Emit, RuleId, RuleV2};
use crate::rules::STANDARD_RULE_ABOUT;

const OPERATORS: TokenSet = TokenSet::create(&[
    ANDAND, ARROW, DIV, DIVEQ, ELVIS, EQ, EQEQ, EQEQEQ, EXCLEQ, EXCLEQEQEQ, GT, GTEQ, LT, LTEQ, MINUS, MINUSEQ, MUL, MULTEQ, OROR,
    PERC, PERCEQ, PLUS, PLUSEQ,
]);
const VISITED_TYPES: TokenSet = TokenSet::or_set(&[OPERATORS, TokenSet::create(&[IDENTIFIER])]);

pub struct SpacingAroundOperatorsRule;

impl RuleV2 for SpacingAroundOperatorsRule {
    fn rule_id(&self) -> RuleId {
        RuleId("standard:op-spacing")
    }

    fn visited_types(&self) -> Option<TokenSet> {
        Some(VISITED_TYPES)
    }

    fn about(&self) -> About {
        STANDARD_RULE_ABOUT
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        // Allow `val foo = -1`
        if is_unary_operator(ast, node) {
            return;
        }
        // Allow `foo(*array)`
        if is_spread_operator(ast, node) {
            return;
        }
        // Allow `import *`
        if is_import(ast, node) {
            return;
        }
        let element_type = ast.element_type(node);
        let parent_type = ast.parent(node).map(|p| ast.element_type(p));
        // Allow `<T> fun foo(...)`, `class Foo<T> { ... }` and `Foo<*>`
        if matches!(element_type, LT | GT | MUL) && parent_type != Some(OPERATION_REFERENCE) {
            return;
        }
        if OPERATORS.contains(element_type) || (element_type == IDENTIFIER && parent_type == Some(OPERATION_REFERENCE)) {
            let spacing_before = ast.is_white_space(ast.prev_leaf(node));
            let spacing_after = ast.is_white_space(ast.next_leaf(node));
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
                emit(ast, ast.start_offset(node) + ast.text_length(node), &message, true)
                    .if_autocorrect_allowed(|| ast.upsert_whitespace_after_me(node, " "));
            }
        }
    }
}

fn is_unary_operator(ast: &Ast, node: NodeId) -> bool {
    ast.parent_matching(node, |it| ast.element_type(it) == OPERATION_REFERENCE)
        .and_then(|it| ast.parent(it))
        .map(|it| ast.element_type(it))
        == Some(PREFIX_EXPRESSION)
}

fn is_spread_operator(ast: &Ast, node: NodeId) -> bool {
    // fn(*array)
    ast.element_type(node) == MUL && ast.parent(node).map(|p| ast.element_type(p)) == Some(VALUE_ARGUMENT)
}

fn is_import(ast: &Ast, node: NodeId) -> bool {
    // import *
    ast.is_part_of(node, IMPORT_DIRECTIVE)
}
