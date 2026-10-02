//! Port of `rules/RememberStateMissing.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtCallExpression, KtFunction};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::is_remembered;
use crate::core::util::psi_elements::find_all_children;

pub struct RememberStateMissing;

const METHODS_THAT_NEED_REMEMBERING: &[&str] = &[
    "derivedStateOf",
    "mutableStateOf",
    "mutableIntStateOf",
    "mutableFloatStateOf",
    "mutableDoubleStateOf",
    "mutableLongStateOf",
    "mutableIntListOf",
    "mutableLongListOf",
    "mutableFloatListOf",
    "mutableIntSetOf",
    "mutableLongSetOf",
    "mutableFloatSetOf",
    "mutableIntIntMapOf",
    "mutableIntLongMapOf",
    "mutableIntFloatMapOf",
    "mutableLongIntMapOf",
    "mutableLongLongMapOf",
    "mutableLongFloatMapOf",
    "mutableFloatIntMapOf",
    "mutableFloatLongMapOf",
    "mutableFloatFloatMapOf",
];

fn callee_text(ast: &Ast, call: KtCallExpression) -> Option<String> {
    call.callee_expression(ast).map(|c| ast.text(c))
}

impl ComposeKtVisitor for RememberStateMissing {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        let not_remembered: Vec<(KtCallExpression, String)> = find_all_children::<KtCallExpression>(ast, function.node())
            .into_iter()
            .filter_map(|it| callee_text(ast, it).filter(|c| METHODS_THAT_NEED_REMEMBERING.contains(&c.as_str())).map(|c| (it, c)))
            .filter(|(it, _)| !is_remembered(ast, *it, function.node()))
            .collect();
        for (call_expression, name) in not_remembered {
            emitter.report(ast, call_expression.node(), &error_message(&name), false);
        }
    }
}

pub fn error_message(name: &str) -> String {
    format!(
        "Using `{name}` in a @Composable function without it being inside of a remember function.\n\
         If you don't remember the state instance, a new state instance will be created when the function is recomposed.\n\
         See https://mrmans0n.github.io/compose-rules/rules/#state-should-be-remembered-in-composables for more information."
    )
}
