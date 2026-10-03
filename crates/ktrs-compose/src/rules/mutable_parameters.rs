//! Port of `rules/MutableParameters.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_callable_declarations::is_type_mutable;

pub struct MutableParameters;

impl ComposeKtVisitor for MutableParameters {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        for parameter in function.value_parameters(ast).into_iter().filter(|it| is_type_mutable(ast, it.node())) {
            emitter.report(ast, parameter.node(), MUTABLE_PARAMETER_IN_COMPOSE, false);
        }
    }
}

pub const MUTABLE_PARAMETER_IN_COMPOSE: &str = "\
Using mutable objects as state in Compose will cause your users to see incorrect or stale data in your app.
Mutable objects that are not observable, such as ArrayList<T> or a mutable data class, cannot be observed by Compose to trigger recomposition when they change.
See https://mrmans0n.github.io/compose-rules/rules/#do-not-use-inherently-mutable-types-as-parameters for more information.";
