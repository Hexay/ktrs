//! Port of `rules/MutableStateParameter.kt`.

use std::sync::LazyLock;

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;
use ktrs_lint::rules::internal::KotlinRegex;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;

pub struct MutableStateParameter;

impl ComposeKtVisitor for MutableStateParameter {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        let parameters: Vec<_> = function
            .value_parameters(ast)
            .into_iter()
            .filter(|it| it.type_reference(ast).is_some_and(|t| MUTABLE_STATE_REGEX.matches(&t.text(ast))))
            .collect();
        for parameter in parameters {
            emitter.report(ast, parameter.node(), MUTABLE_STATE_PARAMETER_IN_COMPOSE, false);
        }
    }
}

static MUTABLE_STATE_REGEX: LazyLock<KotlinRegex> = LazyLock::new(|| {
    KotlinRegex::new(concat!(
        "(MutableState<.*>|",
        "Mutable(Int|Float|Double|Long)State|",
        "MutableIntList|MutableLongList|MutableFloatList|",
        "MutableIntSet|MutableLongSet|MutableFloatSet|",
        "MutableIntIntMap|MutableIntLongMap|MutableIntFloatMap|",
        "MutableLongIntMap|MutableLongLongMap|MutableLongFloatMap|",
        "MutableFloatIntMap|MutableFloatLongMap|MutableFloatFloatMap)\\??",
    ))
});

pub const MUTABLE_STATE_PARAMETER_IN_COMPOSE: &str = "\
MutableState shouldn't be used as a parameter in a @Composable function, as it promotes joint ownership over a state between a component and its user.
If possible, consider making the component stateless and concede the state change to the caller. If mutation of the parent’s owned property is required in the component, consider creating a ComponentState class with the domain specific meaningful field that is backed by mutableStateOf().
See https://mrmans0n.github.io/compose-rules/rules/#do-not-use-mutablestate-as-a-parameter for more information.";
