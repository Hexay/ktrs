//! Port of `rules/ModifierClickableOrder.kt`.

use ktrs_ast::psi::{KtCallExpression, KtDotQualifiedExpression, KtFunction, KtIfExpression, KtReferenceExpression, KtValueArgument};
use ktrs_ast::{Ast, NodeId};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::modifiers::{MODIFIER_NAMES, arguments_using_modifiers, modifier_parameters, obtain_all_modifier_names};
use crate::core::util::psi_elements::find_all_children;

pub struct ModifierClickableOrder;

impl ComposeKtVisitor for ModifierClickableOrder {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let Some(code) = function.body_block_expression(ast) else { return };
        let initial_modifier_names: Vec<String> =
            modifier_parameters(ast, function, config).into_iter().filter_map(|it| it.name(ast)).collect();
        let mut modifiers: Vec<String> = Vec::new();
        for name in initial_modifier_names.iter().flat_map(|it| obtain_all_modifier_names(ast, code, it)) {
            insert(&mut modifiers, name);
        }
        insert(&mut modifiers, "Modifier".to_owned());
        let type_names: Vec<String> = MODIFIER_NAMES.iter().map(|s| s.to_string()).collect();
        let suspicious_order_modifiers: Vec<KtCallExpression> = find_all_children::<KtCallExpression>(ast, code.node())
            .into_iter()
            .filter(|it| callee_starts_upper_case(ast, *it))
            .flat_map(|call| arguments_using_modifiers(ast, call, &modifiers, &type_names))
            .filter_map(|argument| argument.argument_expression(ast))
            .filter_map(|it| KtDotQualifiedExpression::cast(ast, it))
            .filter_map(|chain| find_call_expression_suspicious_order(ast, chain))
            .collect();
        for method_invocation in suspicious_order_modifiers {
            emitter.report(ast, method_invocation.node(), MODIFIER_CHAIN_WITH_SUSPICIOUS_ORDER, false);
        }
    }
}

fn insert(set: &mut Vec<String>, name: String) {
    if !set.contains(&name) {
        set.push(name);
    }
}

/// `calleeExpression?.text?.first()?.isUpperCase() == true`.
pub(crate) fn callee_starts_upper_case(ast: &Ast, call: KtCallExpression) -> bool {
    callee_text(ast, call).and_then(|t| t.chars().next()).is_some_and(char::is_uppercase)
}

fn callee_text(ast: &Ast, call: KtCallExpression) -> Option<String> {
    call.callee_expression(ast).map(|c| ast.text(c))
}

fn find_call_expression_suspicious_order(ast: &Ast, chain: KtDotQualifiedExpression) -> Option<KtCallExpression> {
    let mut current_receiver: NodeId = chain.receiver_expression(ast).expect("NullPointerException: receiverExpression");
    let mut current_selector: Option<NodeId> = chain.selector_expression(ast);
    let mut shape_altering_candidate = false;
    while let Some(selector) = current_selector {
        if let Some(selector) = KtCallExpression::cast(ast, selector) {
            if shape_altering_candidate && is_clickable_interaction(ast, selector) {
                return Some(selector);
            } else if is_clip_with_shape(ast, selector)
                || is_border_with_shape(ast, selector)
                || is_background_with_shape(ast, selector)
                || is_shadow_with_shape(ast, selector)
            {
                shape_altering_candidate = true;
            } else if is_then(ast, selector)
                && let Some(param) = selector.value_arguments(ast).first().copied()
                && let Some(argument_expression) = param.argument_expression(ast).and_then(|e| KtIfExpression::cast(ast, e))
            {
                let suspicious = [argument_expression.then(ast), argument_expression.r#else(ast)]
                    .into_iter()
                    .flatten()
                    .flat_map(|it| modifier_chain_calls(ast, it))
                    .any(|it| {
                        is_clip_with_shape(ast, it)
                            || is_background_with_shape(ast, it)
                            || is_border_with_shape(ast, it)
                            || is_shadow_with_shape(ast, it)
                    });
                if suspicious {
                    shape_altering_candidate = true;
                }
            }
        }
        if let Some(receiver) = KtDotQualifiedExpression::cast(ast, current_receiver) {
            current_selector = receiver.selector_expression(ast);
            current_receiver = receiver.receiver_expression(ast).expect("NullPointerException: receiverExpression");
        } else {
            current_selector = None;
        }
    }
    None
}

fn is_clickable_interaction(ast: &Ast, call: KtCallExpression) -> bool {
    callee_text(ast, call).is_some_and(|t| INTERACTION_MODIFIERS.contains(&t.as_str()))
}

fn is_then(ast: &Ast, call: KtCallExpression) -> bool {
    callee_text(ast, call).as_deref() == Some("then")
}

fn is_clip_with_shape(ast: &Ast, call: KtCallExpression) -> bool {
    callee_text(ast, call).as_deref() == Some("clip")
}

fn is_background_with_shape(ast: &Ast, call: KtCallExpression) -> bool {
    callee_text(ast, call).as_deref() == Some("background")
        && call.value_arguments(ast).into_iter().any(|it| is_named_shape(ast, it) || references_shape(ast, it))
}

fn is_border_with_shape(ast: &Ast, call: KtCallExpression) -> bool {
    callee_text(ast, call).as_deref() == Some("border")
        && call.value_arguments(ast).into_iter().any(|it| is_named_shape(ast, it) || references_shape(ast, it))
}

fn is_shadow_with_shape(ast: &Ast, call: KtCallExpression) -> bool {
    callee_text(ast, call).as_deref() == Some("shadow") && has_shadow_shape(ast, call) && !is_shadow_clip_disabled(ast, call)
}

fn has_shadow_shape(ast: &Ast, call: KtCallExpression) -> bool {
    let arguments = call.value_arguments(ast);
    arguments.iter().any(|&it| is_named_shape(ast, it) || references_shape(ast, it))
        || arguments.get(1).is_some_and(|it| !it.is_named(ast))
}

fn argument_named(ast: &Ast, arguments: &[KtValueArgument], name: &str) -> Option<KtValueArgument> {
    arguments.iter().copied().find(|it| it.argument_name(ast).and_then(|n| n.as_name(ast)).as_deref() == Some(name))
}

fn argument_expression_text(ast: &Ast, argument: KtValueArgument) -> Option<String> {
    argument.argument_expression(ast).map(|e| ast.text(e))
}

fn is_shadow_clip_disabled(ast: &Ast, call: KtCallExpression) -> bool {
    let arguments = call.value_arguments(ast);
    if let Some(named_clip) = argument_named(ast, &arguments, "clip") {
        return argument_expression_text(ast, named_clip).as_deref() == Some("false");
    }
    if let Some(positional_clip) = arguments.get(2).copied().filter(|it| !it.is_named(ast)) {
        return argument_expression_text(ast, positional_clip).as_deref() == Some("false");
    }
    is_zero_elevation(ast, &arguments)
}

fn is_zero_elevation(ast: &Ast, arguments: &[KtValueArgument]) -> bool {
    let elevation = argument_named(ast, arguments, "elevation")
        .or_else(|| arguments.first().copied().filter(|it| !it.is_named(ast)));
    elevation.and_then(|it| argument_expression_text(ast, it)).as_deref() == Some("0.dp")
}

fn modifier_chain_calls(ast: &Ast, expression: NodeId) -> Vec<KtCallExpression> {
    if let Some(call) = KtCallExpression::cast(ast, expression) {
        return vec![call];
    }
    let Some(dot) = KtDotQualifiedExpression::cast(ast, expression) else { return Vec::new() };
    let mut calls = modifier_chain_calls(ast, dot.receiver_expression(ast).expect("NullPointerException: receiverExpression"));
    calls.extend(dot.selector_expression(ast).and_then(|s| KtCallExpression::cast(ast, s)));
    calls
}

fn is_named_shape(ast: &Ast, argument: KtValueArgument) -> bool {
    argument.is_named(ast) && argument.name(ast).as_deref() == Some("shape")
}

fn references_shape(ast: &Ast, argument: KtValueArgument) -> bool {
    let Some(expression) = argument.argument_expression(ast) else { return false };
    if let Some(call) = KtCallExpression::cast(ast, expression) {
        callee_text(ast, call).is_some_and(|t| t.ends_with("Shape"))
    } else if KtReferenceExpression::is(ast, expression) {
        ast.text(expression).ends_with("Shape")
    } else if let Some(if_expression) = KtIfExpression::cast(ast, expression) {
        if_expression.then(ast).is_some_and(|t| ast.text(t).ends_with("Shape"))
            || if_expression.r#else(ast).is_some_and(|e| ast.text(e).ends_with("Shape"))
    } else if KtDotQualifiedExpression::is(ast, expression) {
        let text = ast.text(expression);
        text.starts_with("MaterialTheme.shapes") || text.contains("Shape")
    } else {
        false
    }
}

const INTERACTION_MODIFIERS: &[&str] = &["clickable", "selectable", "toggleable", "triStateToggleable", "combinedClickable"];

pub const MODIFIER_CHAIN_WITH_SUSPICIOUS_ORDER: &str = "\
This order of modifiers is likely to cause visual issues. You should have your clickable modifiers after modifiers that use shapes, so that the clickable selected area takes into account the change in shape as well.
See https://mrmans0n.github.io/compose-rules/rules/#modifier-order-matters for more information.";
