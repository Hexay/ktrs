//! Port of `rules/ModifierWithoutDefault.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtFunction, kt_psi_factory};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_functions::{defined_in_interface, is_abstract, is_actual, is_open, is_override};
use crate::core::util::modifiers::{is_modifier, is_modifier_receiver};

pub struct ModifierWithoutDefault;

impl ComposeKtVisitor for ModifierWithoutDefault {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if defined_in_interface(ast, function)
            || is_actual(ast, function)
            || is_override(ast, function)
            || is_abstract(ast, function)
            || is_open(ast, function)
            || is_modifier_receiver(ast, function.node(), config)
        {
            return;
        }
        let modifier_parameters: Vec<_> = function
            .value_parameters(ast)
            .into_iter()
            .filter(|it| is_modifier(ast, it.node(), config))
            .filter(|it| !it.has_default_value(ast))
            .collect();
        for modifier_parameter in modifier_parameters {
            emitter.report(ast, modifier_parameter.node(), MISSING_MODIFIER_DEFAULT_PARAM, true).if_fix(|| {
                // Re-parsed, not leaf-patched: a patched `Modifier` leaf would make no-unused-imports drop the import.
                let text = format!("{} = Modifier", ast.text(modifier_parameter.node()));
                let new_parameter = kt_psi_factory::create_parameter(ast, &text);
                let parent = ast.tree_parent(modifier_parameter.node()).expect("NullPointerException: treeParent");
                ast.replace_child(parent, modifier_parameter.node(), new_parameter.node());
            });
        }
    }
}

pub const MISSING_MODIFIER_DEFAULT_PARAM: &str = "\
This @Composable function has a modifier parameter but it doesn't have a default value.
See https://mrmans0n.github.io/compose-rules/rules/#modifiers-should-have-default-parameters for more information.";
