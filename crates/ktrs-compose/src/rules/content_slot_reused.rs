//! Port of `rules/ContentSlotReused.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtCallExpression, KtFunction, KtParameter, KtSafeQualifiedExpression, containing_kt_file};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_call_expressions::find_shadowing_redeclarations;
use crate::core::util::kt_callable_declarations::content_slots;
use crate::core::util::kt_parameters::is_type_nullable;
use crate::core::util::lambdas::{composable_lambda_types, lambda_types};
use crate::core::util::psi_elements::find_all_children;

pub struct ContentSlotReused;

impl ComposeKtVisitor for ContentSlotReused {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let file = containing_kt_file(ast, function.node()).expect("containingKtFile").node();
        let lambda_types = lambda_types(ast, file, config);
        let composable_lambda_types = composable_lambda_types(ast, file, config);
        let slots_with_multiple_usages: Vec<_> = content_slots(ast, function.node(), &lambda_types, &composable_lambda_types)
            .into_iter()
            .filter(|slot| find_not_shadowed_usages_of(ast, function, *slot).len() >= 2)
            .collect();
        for slot in slots_with_multiple_usages {
            emitter.report(ast, slot.node(), CONTENT_SLOTS_SHOULD_NOT_BE_REUSED, false);
        }
    }
}

fn callee_text(ast: &Ast, call: KtCallExpression) -> Option<String> {
    call.callee_expression(ast).map(|c| ast.text(c))
}

/// `KtFunction.findNotShadowedUsagesOf(slot)`.
fn find_not_shadowed_usages_of(ast: &Ast, function: KtFunction, slot: KtParameter) -> Vec<KtCallExpression> {
    let Some(slot_name) = slot.name(ast).filter(|it| !it.is_empty()) else { return Vec::new() };
    let slots: Vec<KtCallExpression> = if is_type_nullable(ast, slot) {
        find_all_children::<KtSafeQualifiedExpression>(ast, function.node())
            .into_iter()
            .filter(|it| it.receiver_expression(ast).map(|r| ast.text(r)).as_deref() == Some(slot_name.as_str()))
            .filter_map(|it| KtCallExpression::cast(ast, it.selector_expression(ast)?))
            .filter(|it| callee_text(ast, *it).as_deref() == Some("invoke"))
            .collect()
    } else {
        find_all_children::<KtCallExpression>(ast, function.node())
            .into_iter()
            .filter(|it| callee_text(ast, *it).as_deref() == Some(slot_name.as_str()))
            .collect()
    };
    slots.into_iter().filter(|it| find_shadowing_redeclarations(ast, *it, &slot_name, function.node()).is_empty()).collect()
}

pub const CONTENT_SLOTS_SHOULD_NOT_BE_REUSED: &str = "\
Content slots should not be reused in different code branches/scopes of a composable function, to preserve the slot internal state.
You can wrap the slot in a remember { movableContentOf { ... }} block to make sure their internal state is preserved correctly.
See https://mrmans0n.github.io/compose-rules/rules/#content-slots-should-not-be-reused-in-branching-code for more information.";
