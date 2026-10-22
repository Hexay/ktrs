//! Port of `rules/ModifierShouldBeForwardedToEveryBranch.kt`.

use ktrs_ast::psi::{
    KtBlockExpression, KtCallExpression, KtFile, KtFunction, KtIfExpression, KtNameReferenceExpression,
    KtNamedDeclaration, KtNamedFunction, KtParameter, KtParenthesizedExpression, KtQualifiedExpression,
    KtReturnExpression, KtWhenExpression, name as declaration_name,
};
use ktrs_ast::{Ast, NodeId};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_annotateds::{is_annotated_with, is_composable};
use crate::core::util::kt_functions::has_any_context_arguments;
use crate::core::util::modifiers::{arguments_using_modifiers, modifier_parameters, modifier_type_names};

/// `BranchModifierForwardingResolver`: true for forwarding calls, false for missing forwarding, and omits unknown
/// calls (the Kotlin `Map`, in insertion order).
pub type BranchModifierForwardingResolver =
    fn(&Ast, KtNamedFunction, &[KtCallExpression], &dyn ComposeKtConfig) -> Vec<(KtCallExpression, bool)>;

pub struct ModifierShouldBeForwardedToEveryBranch {
    resolve_forwarding: BranchModifierForwardingResolver,
}

impl ModifierShouldBeForwardedToEveryBranch {
    pub fn new(resolve_forwarding: BranchModifierForwardingResolver) -> Self {
        ModifierShouldBeForwardedToEveryBranch { resolve_forwarding }
    }
}

impl Default for ModifierShouldBeForwardedToEveryBranch {
    fn default() -> Self {
        Self::new(psi_branch_forwarding)
    }
}

impl ComposeKtVisitor for ModifierShouldBeForwardedToEveryBranch {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let Some(named_function) = KtNamedFunction::cast(ast, function.node()) else { return };
        let Some(root) = function.body_expression(ast).and_then(|body| single_statement(ast, body)) else { return };
        if !KtIfExpression::is(ast, root) && !KtWhenExpression::is(ast, root) {
            return;
        }
        let forwarding = (self.resolve_forwarding)(ast, named_function, &branch_calls(ast, root), config);
        if !forwarding.iter().any(|&(_, forwards)| forwards) {
            return;
        }
        for (call, _) in forwarding.into_iter().filter(|&(_, forwards)| !forwards) {
            emitter.report(ast, call.node(), MISSING_BRANCH_MODIFIER, false);
        }
    }
}

/// `KtExpression.singleStatement()`.
fn single_statement(ast: &Ast, expression: NodeId) -> Option<NodeId> {
    if let Some(block) = KtBlockExpression::cast(ast, expression) {
        let statements = block.statements(ast);
        return if statements.len() == 1 { single_statement(ast, statements[0]) } else { None };
    }
    if let Some(parenthesized) = KtParenthesizedExpression::cast(ast, expression) {
        return single_statement(ast, parenthesized.expression(ast)?);
    }
    if let Some(return_expression) = KtReturnExpression::cast(ast, expression) {
        return single_statement(ast, return_expression.returned_expression(ast)?);
    }
    Some(expression)
}

/// `KtExpression.branchCalls()`.
fn branch_calls(ast: &Ast, expression: NodeId) -> Vec<KtCallExpression> {
    let Some(expression) = single_statement(ast, expression) else { return Vec::new() };
    if let Some(if_expression) = KtIfExpression::cast(ast, expression) {
        let mut calls = if_expression.then(ast).map(|it| branch_calls(ast, it)).unwrap_or_default();
        calls.extend(if_expression.r#else(ast).map(|it| branch_calls(ast, it)).unwrap_or_default());
        calls
    } else if let Some(when_expression) = KtWhenExpression::cast(ast, expression) {
        when_expression.entries(ast).into_iter().flat_map(|it| it.expression(ast).map(|e| branch_calls(ast, e)).unwrap_or_default()).collect()
    } else if let Some(qualified) = KtQualifiedExpression::cast(ast, expression) {
        qualified.selector_expression(ast).and_then(|s| KtCallExpression::cast(ast, s)).into_iter().collect()
    } else if let Some(call) = KtCallExpression::cast(ast, expression) {
        vec![call]
    } else {
        Vec::new()
    }
}

pub const MISSING_BRANCH_MODIFIER: &str = "One branch forwards the modifier parameter, but this branch does not. Pass it to every branch whose composable accepts a modifier.";

/// `modifiers.firstOrNull { it.name == "modifier" } ?: modifiers.singleOrNull()`.
fn modifier_or_single(ast: &Ast, modifiers: &[KtParameter]) -> Option<KtParameter> {
    modifiers
        .iter()
        .copied()
        .find(|it| it.name(ast).as_deref() == Some("modifier"))
        .or_else(|| if modifiers.len() == 1 { Some(modifiers[0]) } else { None })
}

fn psi_branch_forwarding(
    ast: &Ast,
    function: KtNamedFunction,
    calls: &[KtCallExpression],
    config: &dyn ComposeKtConfig,
) -> Vec<(KtCallExpression, bool)> {
    let kotlin = config.embedded_kotlin();
    // Without resolution, limit analysis to top-level Unit functions with an identifiable modifier parameter.
    let Some(file) = ast.tree_parent(function.node()).and_then(|it| KtFile::cast(ast, it)) else { return Vec::new() };
    if function.receiver_type_reference(ast).is_some() || has_any_context_arguments(ast, function, kotlin) {
        return Vec::new();
    }
    if function.type_reference(ast).is_some_and(|it| it.text(ast) != "Unit") {
        return Vec::new();
    }
    let modifiers = modifier_parameters(ast, KtFunction::of(ast, function.node()), config);
    let Some(modifier) = modifier_or_single(ast, &modifiers) else { return Vec::new() };
    let Some(modifier_name) = modifier.name(ast) else { return Vec::new() };
    let type_names = modifier_type_names(config);
    // Only unqualified, unambiguous same-file calls can be matched without semantic resolution.
    // A when subject declaration may shadow either the modifier or the callee.
    calls
        .iter()
        .copied()
        .filter_map(|call| {
            if ast.tree_parent(call.node()).is_some_and(|it| KtQualifiedExpression::is(ast, it)) {
                return None;
            }
            if ast
                .parents(call.node())
                .take_while(|&it| it != function.node())
                .any(|it| KtWhenExpression::cast(ast, it).is_some_and(|w| w.subject_variable(ast).is_some()))
            {
                return None;
            }
            let name = KtNameReferenceExpression::cast(ast, call.callee_expression(ast)?)?.referenced_name(ast);
            // Parameters and imports can hide a same-file declaration. Require a single local candidate.
            if function.value_parameters(ast).iter().any(|it| it.name(ast).as_deref() == Some(name.as_str())) {
                return None;
            }
            if file.import_directives(ast).iter().any(|it| {
                it.alias_name(ast).or_else(|| it.imported_fq_name(ast).and_then(|fq| fq.short_name())).as_deref()
                    == Some(name.as_str())
            }) {
                return None;
            }
            let candidates: Vec<NodeId> = file
                .declarations(ast)
                .into_iter()
                .filter(|&it| KtNamedDeclaration::is(ast, it))
                .filter(|&it| declaration_name(ast, it).as_deref() == Some(name.as_str()))
                .collect();
            let [candidate] = candidates[..] else { return None };
            let target = KtNamedFunction::cast(ast, candidate)?;
            // Only Unit composables accepting the same modifier type participate in the forwarding check.
            if !is_composable(ast, target.node()) || is_annotated_with(ast, target.node(), &["ReadOnlyComposable".to_owned()]) {
                return None;
            }
            if target.receiver_type_reference(ast).is_some() || has_any_context_arguments(ast, target, kotlin) {
                return None;
            }
            let not_unit = match target.type_reference(ast) {
                Some(type_reference) => type_reference.text(ast) != "Unit",
                None => !target.has_block_body(ast),
            };
            if not_unit {
                return None;
            }
            let target_parameters = target.value_parameters(ast);
            if target_parameters.iter().any(|it| it.is_var_arg(ast)) {
                return None;
            }
            let target_modifiers = modifier_parameters(ast, KtFunction::of(ast, target.node()), config);
            let target_modifier = modifier_or_single(ast, &target_modifiers)?;
            if target_modifier.type_reference(ast).map(|it| it.text(ast)) != modifier.type_reference(ast).map(|it| it.text(ast)) {
                return None;
            }

            // Named arguments identify the parameter directly. Positional matching is safe only before named arguments.
            let modifier_index = target_parameters.iter().position(|&it| it == target_modifier)?;
            let arguments = call.value_arguments(ast);
            let target_modifier_name = target_modifier.name(ast);
            let named: Vec<_> = arguments
                .iter()
                .copied()
                .filter(|it| it.argument_name(ast).map(|n| n.text(ast)) == target_modifier_name)
                .collect();
            let named_argument = if named.len() == 1 { Some(named[0]) } else { None };
            let positional_candidate = arguments.get(modifier_index).copied();
            if named_argument.is_none()
                && positional_candidate.is_some_and(|it| !it.is_named(ast))
                && arguments.iter().take(modifier_index).any(|it| it.is_named(ast))
            {
                return None;
            }
            let positional_argument = positional_candidate
                .filter(|_| arguments.iter().take(modifier_index + 1).all(|argument| !argument.is_named(ast)));
            let forwards = match named_argument.or(positional_argument) {
                None => Some(false),
                Some(argument)
                    if arguments_using_modifiers(ast, call, std::slice::from_ref(&modifier_name), &type_names)
                        .contains(&argument) =>
                {
                    Some(true)
                }
                Some(argument) if argument.argument_expression(ast).is_some_and(|it| type_names.contains(&ast.text(it))) => {
                    Some(false)
                }
                Some(_) => None, // Unknown expressions and aliases are not proof of missing forwarding.
            };
            forwards.map(|it| (call, it))
        })
        .collect()
}
