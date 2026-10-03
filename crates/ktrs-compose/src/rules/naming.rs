//! Port of `rules/Naming.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;
use ktrs_lint::rules::internal::KotlinRegex;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_annotateds::is_suppressed;
use crate::core::util::kt_functions::{has_receiver_type, is_operator, is_override, returns_value};

pub struct Naming;

impl ComposeKtVisitor for Naming {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if !function.has_block_body(ast) {
            return;
        }
        if is_operator(ast, function) {
            return;
        }
        if is_override(ast, function) {
            return;
        }
        if is_suppressed(ast, function.node(), "ComposableNaming") {
            return;
        }
        let Some(function_name) = function.name(ast).filter(|n| !n.is_empty()) else { return };
        // Kotlin's `first()` is a UTF-16 unit: a surrogate is neither upper nor lower case.
        let first_letter = function_name.chars().next().filter(|c| c.len_utf16() == 1);
        if returns_value(ast, function) {
            if first_letter.is_some_and(char::is_uppercase) {
                let is_allowed = config
                    .get_set("allowedComposableFunctionNames", &[])
                    .iter()
                    .any(|it| KotlinRegex::new(it).matches(&function_name));
                if is_allowed {
                    return;
                }
                emitter.report(ast, function.node(), COMPOSABLES_THAT_RETURN_RESULTS_SHOULD_BE_LOWERCASE, false);
            }
        } else if first_letter.is_some_and(char::is_lowercase) && !has_receiver_type(ast, function) {
            emitter.report(ast, function.node(), COMPOSABLES_THAT_DO_NOT_RETURN_RESULTS_SHOULD_BE_CAPITALIZED, false);
        }
    }
}

pub const COMPOSABLES_THAT_DO_NOT_RETURN_RESULTS_SHOULD_BE_CAPITALIZED: &str = "\
Composable functions that return Unit should start with an uppercase letter.
They are considered declarative entities that can be either present or absent in a composition and therefore follow the naming rules for classes.
See https://mrmans0n.github.io/compose-rules/rules/#naming-composable-functions-properly for more information.";

pub const COMPOSABLES_THAT_RETURN_RESULTS_SHOULD_BE_LOWERCASE: &str = "\
Composable functions that return a value should start with a lowercase letter.
While useful and accepted outside of @Composable functions, this factory function convention has drawbacks that set inappropriate expectations for callers when used with @Composable functions.
See https://mrmans0n.github.io/compose-rules/rules/#naming-composable-functions-properly for more information.";
