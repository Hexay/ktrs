//! Port of `rules/RememberContentMissing.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtCallExpression, KtFunction};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::is_remembered;
use crate::core::util::psi_elements::find_all_children;

pub struct RememberContentMissing;

const CONTENT_THAT_NEEDS_REMEMBERING: &[&str] = &["movableContentOf", "movableContentWithReceiverOf"];

fn callee_text(ast: &Ast, call: KtCallExpression) -> Option<String> {
    call.callee_expression(ast).map(|c| ast.text(c))
}

impl ComposeKtVisitor for RememberContentMissing {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        let not_remembered: Vec<KtCallExpression> = find_all_children::<KtCallExpression>(ast, function.node())
            .into_iter()
            .filter(|it| callee_text(ast, *it).is_some_and(|c| CONTENT_THAT_NEEDS_REMEMBERING.contains(&c.as_str())))
            .filter(|it| !is_remembered(ast, *it, function.node()))
            .collect();
        for call_expression in not_remembered {
            match callee_text(ast, call_expression).expect("calleeExpression!!").as_str() {
                "movableContentOf" => {
                    emitter.report(ast, call_expression.node(), MOVABLE_CONTENT_OF_NOT_REMEMBERED, false);
                }
                "movableContentWithReceiverOf" => {
                    emitter.report(ast, call_expression.node(), MOVABLE_CONTENT_WITH_RECEIVER_OF_NOT_REMEMBERED, false);
                }
                _ => {}
            }
        }
    }
}

pub const MOVABLE_CONTENT_OF_NOT_REMEMBERED: &str = "\
Using `movableContentOf` in a @Composable function without it being remembered can cause visual problems, as the content would be recycled when detached from the composition.
See https://mrmans0n.github.io/compose-rules/rules/#movable-content-should-be-remembered for more information.";

pub const MOVABLE_CONTENT_WITH_RECEIVER_OF_NOT_REMEMBERED: &str = "\
Using `movableContentWithReceiverOf` in a @Composable function without it being remembered can cause visual problems, as the content would be recycled when detached from the composition.
See https://mrmans0n.github.io/compose-rules/rules/#movable-content-should-be-remembered for more information.";
