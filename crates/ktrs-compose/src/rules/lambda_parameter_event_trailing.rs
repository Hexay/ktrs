//! Port of `rules/LambdaParameterEventTrailing.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::function_emits_content;
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::lambdas::is_lambda;
use crate::core::util::modifiers::modifier_parameter;

pub struct LambdaParameterEventTrailing;

impl ComposeKtVisitor for LambdaParameterEventTrailing {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if !function_emits_content(ast, function, config) {
            return;
        }
        if modifier_parameter(ast, function, config).is_none() {
            return;
        }
        let Some(trailing_param) = function.value_parameters(ast).last().copied() else { return };
        let Some(type_reference) = trailing_param.type_reference(ast) else { return };
        if !is_lambda(ast, type_reference, &[]) {
            return;
        }
        if is_composable(ast, type_reference.node()) {
            return;
        }
        if trailing_param.has_default_value(ast) {
            return;
        }
        let Some(name) = trailing_param.name(ast) else { return };
        if !name.starts_with("on") {
            return;
        }
        emitter.report(ast, trailing_param.node(), EVENT_LAMBDA_IS_TRAILING_LAMBDA, false);
    }
}

pub const EVENT_LAMBDA_IS_TRAILING_LAMBDA: &str = "\
Lambda parameters in a @Composable that are for events (e.g. onClick, onChange, etc) and are required (they don't have a default value) should not be used as the trailing parameter.
Composable functions that emit content usually reserve the trailing lambda syntax for the content slot, and that can lead to an assumption that other composables can be used in that lambda.
See https://mrmans0n.github.io/compose-rules/rules/#avoid-using-the-trailing-lambda-for-event-lambdas-in-ui-composables for more information.";
