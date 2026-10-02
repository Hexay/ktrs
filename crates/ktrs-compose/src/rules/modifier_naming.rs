//! Port of `rules/ModifierNaming.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::modifiers::is_modifier;

pub struct ModifierNaming;

impl ComposeKtVisitor for ModifierNaming {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let modifiers: Vec<_> = function.value_parameters(ast).into_iter().filter(|it| is_modifier(ast, it.node(), config)).collect();
        if modifiers.is_empty() {
            return;
        }
        if modifiers.len() == 1 {
            if modifiers[0].name(ast).as_deref() != Some("modifier") {
                emitter.report(ast, modifiers[0].node(), MODIFIERS_ARE_SUPPOSED_TO_BE_CALLED_MODIFIER_WHEN_ALONE, false);
            }
        } else {
            for modifier in modifiers {
                let valid = modifier.name(ast).is_some_and(|name| name.to_lowercase().ends_with("modifier"));
                if !valid {
                    emitter.report(ast, modifier.node(), MODIFIERS_ARE_SUPPOSED_TO_END_IN_MODIFIER_WHEN_MULTIPLE, false);
                }
            }
        }
    }
}

pub const MODIFIERS_ARE_SUPPOSED_TO_BE_CALLED_MODIFIER_WHEN_ALONE: &str = "\
Modifier parameters should be called `modifier`.
See https://mrmans0n.github.io/compose-rules/rules/#naming-modifiers-properly for more information.";

pub const MODIFIERS_ARE_SUPPOSED_TO_END_IN_MODIFIER_WHEN_MULTIPLE: &str = "\
Modifier parameters should be called `modifier` or end in `Modifier` if there are more than one in the same @Composable.
See https://mrmans0n.github.io/compose-rules/rules/#naming-modifiers-properly for more information.";
