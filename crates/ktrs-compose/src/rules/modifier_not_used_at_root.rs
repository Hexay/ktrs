//! Port of `rules/ModifierNotUsedAtRoot.kt`.

use ktrs_ast::psi::{KtCallExpression, KtDotQualifiedExpression, KtFunction, KtReferenceExpression, KtValueArgument};
use ktrs_ast::{Ast, NodeId};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::{emits_content, is_in_content_emitters_denylist};
use crate::core::util::kt_call_expressions::is_fully_shadowed;
use crate::core::util::kt_dot_qualified_expressions::root_expression;
use crate::core::util::modifiers::{
    arguments_using_modifiers, modifier_parameter, modifier_type_names, obtain_all_modifier_names, shadowing_names,
};
use crate::core::util::psi_elements::find_all_children;
use crate::rules::modifier_clickable_order::callee_starts_upper_case;

pub struct ModifierNotUsedAtRoot;

impl ComposeKtVisitor for ModifierNotUsedAtRoot {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let Some(modifier) = modifier_parameter(ast, function, config) else { return };
        if modifier.name(ast).as_deref() != Some("modifier") {
            return;
        }
        let Some(code) = function.body_block_expression(ast) else { return };
        let modifiers = obtain_all_modifier_names(ast, code, "modifier");
        let type_names = modifier_type_names(config);
        let errors: Vec<KtValueArgument> = find_all_children::<KtCallExpression>(ast, code.node())
            .into_iter()
            .filter(|it| callee_starts_upper_case(ast, *it))
            .filter_map(|call_expression| {
                let args = arguments_using_modifiers(ast, call_expression, &modifiers, &type_names);
                let first = *args.first()?;
                let shadowed_names = shadowed_modifier_names_up_to(ast, call_expression, function.node(), &modifiers);
                let usage = args
                    .iter()
                    .copied()
                    .find(|arg| {
                        let Some(expr) = arg.argument_expression(ast) else { return true };
                        if KtReferenceExpression::is(ast, expr) {
                            !shadowed_names.contains(&ast.text(expr))
                        } else if let Some(dot) = KtDotQualifiedExpression::cast(ast, expr) {
                            !shadowed_names.contains(&ast.text(root_expression(ast, dot)))
                        } else {
                            true
                        }
                    })
                    .unwrap_or(first);
                Some((call_expression, usage))
            })
            .filter(|(call_expression, _)| !is_fully_shadowed(ast, *call_expression, &modifiers, function.node(), &type_names))
            .filter(|(call_expression, _)| {
                ast.parents(call_expression.node())
                    .take_while(|&it| it != code.node())
                    .filter_map(|it| KtCallExpression::cast(ast, it))
                    .take_while(|it| !is_in_content_emitters_denylist(ast, *it, config))
                    .any(|it| emits_content(ast, it, config))
            })
            .map(|(_, usage)| usage)
            .collect();
        for value_argument in errors {
            emitter.report(ast, value_argument.node(), COMPOSABLE_MODIFIER_SHOULD_BE_USED_AT_THE_TOP_MOST_POSSIBLE_PLACE, false);
        }
    }
}

/// `KtCallExpression.shadowedModifierNamesUpTo(stopAt, modifiers)`.
fn shadowed_modifier_names_up_to(ast: &Ast, call: KtCallExpression, stop_at: NodeId, modifiers: &[String]) -> Vec<String> {
    let mut set: Vec<String> = Vec::new();
    for func in ast.parents(call.node()).take_while(|&it| it != stop_at).filter_map(|it| KtFunction::cast(ast, it)) {
        for name in func.value_parameters(ast).into_iter().flat_map(|param| shadowing_names(ast, param, modifiers)) {
            if !set.contains(&name) {
                set.push(name);
            }
        }
    }
    set
}

pub const COMPOSABLE_MODIFIER_SHOULD_BE_USED_AT_THE_TOP_MOST_POSSIBLE_PLACE: &str = "\
The main Modifier of a @Composable should be applied once as a first modifier in the chain to the root-most layout in the component implementation.
You should move the modifier usage to the appropriate parent Composable.
See https://mrmans0n.github.io/compose-rules/rules/#modifiers-should-be-used-at-the-top-most-layout-of-the-component for more information.";
