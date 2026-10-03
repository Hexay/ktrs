//! Port of `rules/ModifierComposed.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtCallExpression, KtFunction, KtReturnExpression};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::modifiers::is_modifier_receiver;
use crate::core::util::psi_elements::find_direct_children_by_class;

pub struct ModifierComposed;

impl ComposeKtVisitor for ModifierComposed {
    fn visit_function(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if !is_modifier_receiver(ast, function.node(), config) || is_composable(ast, function.node()) {
            return;
        }
        let is_composed_call = |ast: &Ast, call: KtCallExpression| {
            call.callee_expression(ast).is_some_and(|c| ast.text(c) == "composed")
        };
        if function.body_expression(ast).and_then(|b| KtCallExpression::cast(ast, b)).is_some_and(|b| is_composed_call(ast, b)) {
            emitter.report(ast, function.node(), COMPOSED_MODIFIER, false);
        }
        let Some(body_block_expression) = function.body_block_expression(ast) else { return };
        let returns_composed = find_direct_children_by_class::<KtReturnExpression>(ast, body_block_expression.node())
            .into_iter()
            .filter_map(|it| it.returned_expression(ast))
            .filter_map(|it| KtCallExpression::cast(ast, it))
            .any(|it| is_composed_call(ast, it));
        if returns_composed {
            emitter.report(ast, function.node(), COMPOSED_MODIFIER, false);
        }
    }
}

pub const COMPOSED_MODIFIER: &str = "\
Using composed for modifiers is not recommended anymore, due to the performance issues it creates.
You should consider migrating this modifier to be based on Modifier.Node instead.
See https://mrmans0n.github.io/compose-rules/rules/#avoid-modifier-extension-factory-functions for more information.";
