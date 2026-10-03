//! Port of `rules/ComposableNestingDepth.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtCallExpression, KtClassOrObject, KtFunction, KtNamedFunction};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::emits_content;
use crate::core::util::psi_elements::find_all_children;

pub struct ComposableNestingDepth;

impl ComposeKtVisitor for ComposableNestingDepth {
    fn is_opt_in(&self) -> bool {
        true
    }

    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let Some(body) = function.body_expression(ast) else { return };
        let threshold = config.get_int("composableNestingDepthThreshold", 3);
        let body_base_depth = i32::from(KtCallExpression::cast(ast, body).is_some_and(|it| emits_content(ast, it, config)));
        let deepest = find_all_children::<KtCallExpression>(ast, body)
            .into_iter()
            .filter(|call| {
                !ast.parents(call.node())
                    .take_while(|&it| it != body)
                    .any(|it| KtNamedFunction::is(ast, it) || KtClassOrObject::is(ast, it))
            })
            .filter(|it| emits_content(ast, *it, config))
            .map(|call| {
                body_base_depth
                    + ast.parents(call.node())
                        .take_while(|&it| it != body)
                        .filter_map(|it| KtCallExpression::cast(ast, it))
                        .filter(|it| emits_content(ast, *it, config))
                        .count() as i32
            })
            .max();
        let Some(deepest) = deepest else { return };
        if deepest > threshold {
            emitter.report(ast, function.node(), COMPOSABLE_TOO_DEEPLY_NESTED, false);
        }
    }
}

pub const COMPOSABLE_TOO_DEEPLY_NESTED: &str = "\
This @Composable function nests content emitters more deeply than the configured threshold. Extract inner sections into dedicated private @Composable functions to keep the structure readable.
See https://mrmans0n.github.io/compose-rules/rules/#avoid-deeply-nested-composables for more information.";
