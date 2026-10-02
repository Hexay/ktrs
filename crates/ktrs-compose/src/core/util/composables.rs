//! Port of `core/util/Composables.kt`. The emitter allow/deny lists are in `composable_lists.rs`.

use ktrs_ast::psi::{
    KtBinaryExpression, KtBlockExpression, KtCallExpression, KtDeclarationWithBody, KtDotQualifiedExpression,
    KtFunction, KtIfExpression, KtLoopExpression, KtNamedFunction, KtProperty, KtReturnExpression,
    KtSafeQualifiedExpression, KtWhenExpression, reference_expression,
};
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::ELVIS;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::util::composable_lists::{
    COMPOSABLE_EMITTERS_LIST, COMPOSABLE_NON_EMITTERS_LIST, COMPOSITION_LOCAL_REFERENCE_EXPRESSIONS, RESTARTABLE_EFFECTS,
};
use crate::core::util::kotlin_utils::KOTLIN_SCOPE_FUNCTIONS;
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::kt_dot_qualified_expressions::root_expression;
use crate::core::util::kt_functions::is_nested;
use crate::core::util::psi_elements::find_all_children_by_class;

/// A `Map<KtFunction, Int>` in insertion order.
pub type EmissionCountMapping = Vec<(KtFunction, i32)>;

fn callee_text(ast: &Ast, call: KtCallExpression) -> Option<String> {
    call.callee_expression(ast).map(|c| ast.text(c))
}

/// `KtFunction.emitsContent(config)`: a composable with a content-emitting call outside denylisted calls.
pub fn function_emits_content(ast: &Ast, function: KtFunction, config: &dyn ComposeKtConfig) -> bool {
    if !is_composable(ast, function.node()) {
        return false;
    }
    find_all_children_by_class::<KtCallExpression>(ast, function.node(), |ast, current| {
        KtCallExpression::cast(ast, current).is_none_or(|call| !is_in_content_emitters_denylist(ast, call, config))
    })
    .into_iter()
    .any(|it| emits_content(ast, it, config))
}

/// `KtCallExpression.emitsContent(config)`.
pub fn emits_content(ast: &Ast, call: KtCallExpression, config: &dyn ComposeKtConfig) -> bool {
    let Some(method_name) = callee_text(ast, call) else { return false };
    if is_in_content_emitters_denylist(ast, call, config) {
        return false;
    }
    COMPOSABLE_EMITTERS_LIST.contains(&method_name.as_str())
        || config.get_set("contentEmitters", &[]).contains(&method_name)
        || contains_composables_with_modifiers(ast, call)
}

/// `KtCallExpression.isInContentEmittersDenylist(config)`.
pub fn is_in_content_emitters_denylist(ast: &Ast, call: KtCallExpression, config: &dyn ComposeKtConfig) -> bool {
    let Some(method_name) = callee_text(ast, call) else { return false };
    COMPOSABLE_NON_EMITTERS_LIST.contains(&method_name.as_str())
        || config.get_set("contentEmittersDenylist", &[]).contains(&method_name)
}

/// A named `modifier` argument, or an argument chained off `Modifier`.
fn contains_composables_with_modifiers(ast: &Ast, call: KtCallExpression) -> bool {
    let arguments = call.value_arguments(ast);
    let has_named_modifier = arguments
        .iter()
        .filter(|it| it.is_named(ast))
        .any(|it| it.argument_name(ast).is_some_and(|n| n.text(ast) == "modifier"));
    if has_named_modifier {
        return true;
    }
    arguments
        .iter()
        .filter_map(|it| it.argument_expression(ast))
        .filter_map(|it| KtDotQualifiedExpression::cast(ast, it))
        .any(|it| ast.text(root_expression(ast, it)) == "Modifier")
}

/// The state of `KtExpression.uiEmitterCount`'s recursive counter (upstream captures it in the closure).
struct EmitterCounter<'a> {
    ast: &'a Ast,
    config: &'a dyn ComposeKtConfig,
    origin: NodeId,
    total_emitters_found: i32,
    current_block_started_at: i32,
}

impl EmitterCounter<'_> {
    fn count(&mut self, current: Option<NodeId>) -> i32 {
        let ast = self.ast;
        let Some(current) = current else { return 0 };
        if let Some(function) = KtNamedFunction::cast(ast, current) {
            return if current != self.origin && is_composable(ast, current) && is_nested(ast, function) {
                0
            } else {
                self.count(function.body_block_expression(ast).map(|b| b.node()))
            };
        }
        if let Some(declaration) = KtDeclarationWithBody::cast(ast, current) {
            return self.count(declaration.body_block_expression(ast).map(|b| b.node()));
        }
        if let Some(block) = KtBlockExpression::cast(ast, current) {
            self.current_block_started_at = self.total_emitters_found;
            return block.statements(ast).into_iter().fold(0, |acc, next| acc + self.count(Some(next)));
        }
        if let Some(call) = KtCallExpression::cast(ast, current) {
            if emits_content(ast, call, self.config) {
                self.total_emitters_found += 1;
                return 1;
            }
            if callee_text(ast, call).is_some_and(|c| KOTLIN_SCOPE_FUNCTIONS.contains(&c.as_str())) {
                return self.count(single_lambda_body(ast, call));
            }
            return 0;
        }
        if let Some(r#loop) = KtLoopExpression::cast(ast, current) {
            let emitters = self.count(r#loop.body(ast));
            return if emitters > 0 {
                self.total_emitters_found += 1;
                emitters + 1
            } else {
                0
            };
        }
        if let Some(safe) = KtSafeQualifiedExpression::cast(ast, current) {
            let selector = safe.selector_expression(ast).and_then(|s| KtCallExpression::cast(ast, s));
            let scoped = selector.filter(|s| callee_text(ast, *s).is_some_and(|c| KOTLIN_SCOPE_FUNCTIONS.contains(&c.as_str())));
            return self.count(scoped.and_then(|s| single_lambda_body(ast, s)));
        }
        if let Some(binary) = KtBinaryExpression::cast(ast, current) {
            if binary.operation_token(ast) != Some(ELVIS) {
                return 0;
            }
            let left_count = self.count(binary.left(ast));
            let right_count = self.count(binary.right(ast));
            return left_count.max(right_count);
        }
        if let Some(r#if) = KtIfExpression::cast(ast, current) {
            let if_count = self.count(r#if.then(ast));
            let else_count = self.count(r#if.r#else(ast));
            return if_count.max(else_count);
        }
        if let Some(when) = KtWhenExpression::cast(ast, current) {
            return when.entries(ast).into_iter().map(|it| self.count(it.expression(ast))).max().unwrap_or(0);
        }
        if let Some(r#return) = KtReturnExpression::cast(ast, current) {
            if r#return.labeled_expression(ast).is_some() {
                return 0;
            }
            // An early return after the only emitter of a block that started without any cancels that emitter.
            let current_block = self.total_emitters_found - self.current_block_started_at;
            return if current_block == 1 && self.current_block_started_at == 0 { -1 } else { 0 };
        }
        0
    }
}

/// `lambdaArguments.singleOrNull()?.getLambdaExpression()?.bodyExpression`.
fn single_lambda_body(ast: &Ast, call: KtCallExpression) -> Option<NodeId> {
    let lambda_arguments = call.lambda_arguments(ast);
    let [single] = lambda_arguments.as_slice() else { return None };
    Some(single.lambda_expression(ast)?.body_expression(ast)?.node())
}

/// `KtExpression.uiEmitterCount(config)`: how many UI emitters the expression (a composable) runs at most.
fn ui_emitter_count(ast: &Ast, expression: NodeId, config: &dyn ComposeKtConfig) -> i32 {
    let mut counter =
        EmitterCounter { ast, config, origin: expression, total_emitters_found: 0, current_block_started_at: 0 };
    counter.count(Some(expression)).max(0)
}

/// `KtFunction.indirectUiEmitterCount(mapping, config)`: the body's direct calls that emit content or call a
/// function of `mapping` known to emit.
fn indirect_ui_emitter_count(ast: &Ast, function: KtFunction, mapping: &EmissionCountMapping, config: &dyn ComposeKtConfig) -> i32 {
    let Some(body_block) = function.body_block_expression(ast) else { return 0 };
    let by_name = |name: &str| mapping.iter().rev().find(|(f, _)| f.name(ast).as_deref() == Some(name)).map(|(_, v)| *v);
    body_block
        .statements(ast)
        .into_iter()
        .filter_map(|it| KtCallExpression::cast(ast, it))
        .filter(|call| {
            if emits_content(ast, *call, config) {
                return true;
            }
            let Some(name) = callee_text(ast, *call) else { return false };
            by_name(&name).is_some_and(|value| value > 0)
        })
        .count() as i32
}

/// `Sequence<KtFunction>.createDirectComposableToEmissionCountMapping(config)`.
pub fn create_direct_composable_to_emission_count_mapping(
    ast: &Ast,
    functions: &[KtFunction],
    config: &dyn ComposeKtConfig,
) -> EmissionCountMapping {
    functions.iter().map(|&it| (it, ui_emitter_count(ast, it.node(), config))).collect()
}

/// `refineComposableToEmissionCountMapping(initialMapping, config)`: recount through known emitters until stable.
pub fn refine_composable_to_emission_count_mapping(
    ast: &Ast,
    initial_mapping: EmissionCountMapping,
    config: &dyn ComposeKtConfig,
) -> EmissionCountMapping {
    let mut current = initial_mapping;
    loop {
        let updated: EmissionCountMapping =
            current.iter().map(|&(function, _)| (function, indirect_ui_emitter_count(ast, function, &current, config))).collect();
        if updated == current {
            return current;
        }
        current = updated;
    }
}

/// `KtProperty.declaresCompositionLocal`: a `val` initialized by a `compositionLocalOf`-like call.
pub fn declares_composition_local(ast: &Ast, property: KtProperty) -> bool {
    !property.is_var(ast)
        && property.has_initializer(ast)
        && property.initializer(ast).is_some_and(|initializer| {
            KtCallExpression::is(ast, initializer)
                && reference_expression(ast, initializer)
                    .is_some_and(|r| COMPOSITION_LOCAL_REFERENCE_EXPRESSIONS.contains(&r.text(ast).as_str()))
        })
}

/// `KtCallExpression.isRestartableEffect`.
pub fn is_restartable_effect(ast: &Ast, call: KtCallExpression) -> bool {
    callee_text(ast, call).is_some_and(|c| RESTARTABLE_EFFECTS.contains(&c.as_str()))
}

/// `KtCallExpression.isRemembered(stopAt)`: inside a `remember*`/`retain` call below `stop_at`.
pub fn is_remembered(ast: &Ast, call: KtCallExpression, stop_at: NodeId) -> bool {
    ast.parents(call.node())
        .take_while(|&it| it != stop_at)
        .filter_map(|it| KtCallExpression::cast(ast, it))
        .filter_map(|it| callee_text(ast, it))
        .any(|name| name.starts_with("remember") || name == "retain")
}
