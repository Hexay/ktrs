//! Port of `rules/ModifierReused.kt`.

use ktrs_ast::psi::{KtBlockExpression, KtCallExpression, KtFunction, KtReturnExpression};
use ktrs_ast::{Ast, NodeId};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::function_emits_content;
use crate::core::util::kt_call_expressions::is_fully_shadowed;
use crate::core::util::modifiers::{is_using_modifiers, modifier_parameters, modifier_type_names, obtain_all_modifier_names};
use crate::core::util::psi_elements::{find_all_children, walk_backwards};
use crate::rules::modifier_clickable_order::callee_starts_upper_case;

pub struct ModifierReused;

/// The context `modifierUsagesSet()` (a local fun upstream) closes over.
struct Usages<'a> {
    ast: &'a Ast,
    modifier_names: &'a [String],
    function: KtFunction,
    type_names: &'a [String],
}

impl Usages<'_> {
    fn uses_modifiers(&self, call: KtCallExpression) -> bool {
        is_using_modifiers(self.ast, call, self.modifier_names, self.type_names)
            && !is_fully_shadowed(self.ast, call, self.modifier_names, self.function.node(), self.type_names)
    }

    /// `Sequence<PsiElement>.modifierUsagesSet()`.
    fn modifier_usages_set(&self, elements: impl Iterator<Item = NodeId>) -> Vec<KtCallExpression> {
        let mut set = Vec::new();
        for call in elements.filter_map(|it| KtCallExpression::cast(self.ast, it)).filter(|&it| self.uses_modifiers(it)) {
            if !set.contains(&call) {
                set.push(call);
            }
        }
        set
    }

    fn hits(&self, call_expression: KtCallExpression, composable_block_expression: KtBlockExpression) -> Vec<KtCallExpression> {
        let ast = self.ast;
        let composable_hits = self.modifier_usages_set(
            walk_backwards(ast, call_expression.node(), Some(composable_block_expression.node())).into_iter(),
        );
        if composable_hits.len() != 1 {
            return composable_hits;
        }
        let prev_local_hits = self.modifier_usages_set(ast.siblings_with_itself(call_expression.node(), false, true));
        let same = prev_local_hits.len() == composable_hits.len() && prev_local_hits.iter().all(|it| composable_hits.contains(it));
        if !same {
            return composable_hits;
        }
        let is_followed_by_early_return = ast
            .siblings_with_itself(call_expression.node(), true, true)
            .filter_map(|it| KtReturnExpression::cast(ast, it))
            .any(|it| it.labeled_expression(ast).is_none());
        if is_followed_by_early_return { Vec::new() } else { composable_hits }
    }
}

impl ComposeKtVisitor for ModifierReused {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if !function_emits_content(ast, function, config) {
            return;
        }
        let Some(composable_block_expression) = function.body_block_expression(ast) else { return };
        let mut initial_modifier_names: Vec<String> = Vec::new();
        for name in modifier_parameters(ast, function, config).into_iter().filter_map(|it| it.name(ast)) {
            if !initial_modifier_names.contains(&name) {
                initial_modifier_names.push(name);
            }
        }
        if initial_modifier_names.is_empty() {
            return;
        }
        let type_names = modifier_type_names(config);
        let all_modifier_names: Vec<Vec<String>> =
            initial_modifier_names.iter().map(|it| obtain_all_modifier_names(ast, composable_block_expression, it)).collect();
        for modifier_names in all_modifier_names {
            let usages = Usages { ast, modifier_names: &modifier_names, function, type_names: &type_names };
            let mut reported: Vec<KtCallExpression> = Vec::new();
            for call in find_all_children::<KtCallExpression>(ast, composable_block_expression.node())
                .into_iter()
                .filter(|it| callee_starts_upper_case(ast, *it))
                .filter(|&it| usages.uses_modifiers(it))
                .map(|call_expression| usages.hits(call_expression, composable_block_expression))
                .filter(|it| it.len() > 1)
                .flatten()
            {
                if !reported.contains(&call) {
                    reported.push(call);
                }
            }
            for call_expression in reported {
                emitter.report(ast, call_expression.node(), MODIFIER_SHOULD_BE_USED_ONCE_ONLY, false);
            }
        }
    }
}

pub const MODIFIER_SHOULD_BE_USED_ONCE_ONLY: &str = "\
Modifiers should only be used once and by the root level layout of a Composable. This is true even if appended to or with other modifiers e.g. 'modifier.fillMaxWidth()'.
Use Modifier (with a capital 'M') to construct a new Modifier that you can pass to other composables.
See https://mrmans0n.github.io/compose-rules/rules/#dont-re-use-modifiers for more information.";
