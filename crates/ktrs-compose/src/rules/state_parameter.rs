//! Port of `rules/StateParameter.kt`.

use std::sync::LazyLock;

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;
use ktrs_lint::rules::internal::KotlinRegex;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;

pub struct StateParameter;

impl ComposeKtVisitor for StateParameter {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        let parameters: Vec<_> = function
            .value_parameters(ast)
            .into_iter()
            .filter(|it| it.type_reference(ast).is_some_and(|t| STATE_REGEX.matches(&t.text(ast))))
            .collect();
        for parameter in parameters {
            emitter.report(ast, parameter.node(), STATE_PARAMETER_IN_COMPOSE, false);
        }
    }
}

static STATE_REGEX: LazyLock<KotlinRegex> = LazyLock::new(|| KotlinRegex::new("(State<.*>|(Int|Float|Double|Long)State)\\??"));

pub const STATE_PARAMETER_IN_COMPOSE: &str = "\
State shouldn't be used as a parameter in a @Composable function, as it encourages components that are not fully stateless.
Instead, pass the snapshot state value and provide event callbacks for changes.
See https://github.com/androidx/androidx/blob/androidx-main/compose/docs/compose-component-api-guidelines.md#statet-as-a-parameter for more information.";
