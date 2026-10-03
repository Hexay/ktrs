//! Port of `rules/LambdaParameterInRestartableEffect.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtCallExpression, KtFile, KtFunction, KtIfExpression, KtParameter, KtReferenceExpression, is_dot_selector};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::is_restartable_effect;
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::kt_call_expressions::find_shadowing_redeclarations;
use crate::core::util::lambdas::{is_lambda, lambda_types};
use crate::core::util::psi_elements::find_all_children;

pub struct LambdaParameterInRestartableEffect;

impl ComposeKtVisitor for LambdaParameterInRestartableEffect {
    fn visit_file(&self, ast: &mut Ast, file: KtFile, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let lambda_types = lambda_types(ast, file.node(), config);
        let composables: Vec<_> =
            find_all_children::<KtFunction>(ast, file.node()).into_iter().filter(|it| is_composable(ast, it.node())).collect();
        for composable in composables {
            let effects: Vec<KtCallExpression> = find_all_children::<KtCallExpression>(ast, composable.node())
                .into_iter()
                .filter(|it| is_restartable_effect(ast, *it))
                .collect();
            if effects.is_empty() {
                continue;
            }
            // `associateBy { it.name!! }`: a later duplicate replaces the value, the key keeps its first position.
            let mut lambda_parameters: Vec<(String, KtParameter)> = Vec::new();
            for parameter in composable.value_parameters(ast) {
                let is_lambda_like = parameter.is_lambda_parameter(ast)
                    || parameter.type_reference(ast).is_some_and(|t| is_lambda(ast, t, &lambda_types));
                let Some(name) = parameter.name(ast).filter(|_| is_lambda_like) else { continue };
                match lambda_parameters.iter_mut().find(|(n, _)| *n == name) {
                    Some(entry) => entry.1 = parameter,
                    None => lambda_parameters.push((name, parameter)),
                }
            }
            let lambda_parameter_names: Vec<String> = lambda_parameters.iter().map(|(n, _)| n.clone()).collect();
            if lambda_parameter_names.is_empty() {
                continue;
            }
            let mut used_lambda_parameter_names: Vec<String> = Vec::new();
            for effect in &effects {
                for name in used_in_effect(ast, *effect, &lambda_parameter_names) {
                    if !used_lambda_parameter_names.contains(&name) {
                        used_lambda_parameter_names.push(name);
                    }
                }
            }
            let keyed_lambda_parameter_names: Vec<String> = effects
                .iter()
                .flat_map(|it| it.value_arguments(ast))
                .filter_map(|it| it.argument_expression(ast))
                .filter(|&it| KtReferenceExpression::is(ast, it))
                .map(|it| ast.text(it))
                .filter(|it| used_lambda_parameter_names.contains(it))
                .collect();
            let shadowed_parameters: Vec<&String> = used_lambda_parameter_names
                .iter()
                .filter(|it| {
                    effects
                        .iter()
                        .any(|effect| !find_shadowing_redeclarations(ast, *effect, it, composable.node()).is_empty())
                })
                .collect();
            for parameter_name in &used_lambda_parameter_names {
                if keyed_lambda_parameter_names.contains(parameter_name) || shadowed_parameters.contains(&parameter_name) {
                    continue;
                }
                let parameter = lambda_parameters.iter().find(|(n, _)| n == parameter_name).unwrap().1;
                emitter.report(ast, parameter.node(), LAMBDA_USED_IN_RESTARTABLE_EFFECT, false);
            }
        }
    }
}

/// The `flatMap { effect -> }` body: the lambda parameters invoked or forwarded in the effect's trailing lambda.
fn used_in_effect(ast: &Ast, effect: KtCallExpression, lambda_parameter_names: &[String]) -> Vec<String> {
    let Some(body) = effect.lambda_arguments(ast).last().and_then(|it| it.lambda_expression(ast)).and_then(|it| it.body_expression(ast))
    else {
        return Vec::new();
    };
    let call_expressions: Vec<KtCallExpression> = find_all_children::<KtCallExpression>(ast, body.node())
        .into_iter()
        .filter(|it| !is_dot_selector(ast, it.node()))
        .collect();
    let callee_text = |call: KtCallExpression| call.callee_expression(ast).map(|c| ast.text(c));
    let effect_callee = callee_text(effect);
    let is_disposable_effect = effect_callee.as_deref() == Some("DisposableEffect");
    let is_lifecycle_effect = matches!(effect_callee.as_deref(), Some("LifecycleStartEffect" | "LifecycleResumeEffect"));
    let mut used: Vec<String> = call_expressions
        .iter()
        .filter(|it| {
            let callee = callee_text(**it);
            if is_disposable_effect {
                callee.as_deref() != Some("onDispose")
            } else if is_lifecycle_effect {
                !callee.is_some_and(|c| LIFECYCLE_EFFECT_SCOPE_FUNCTIONS.contains(&c.as_str()))
            } else {
                true
            }
        })
        .filter_map(|it| callee_text(*it))
        .filter(|it| lambda_parameter_names.contains(it))
        .collect();
    let in_names = |text: Option<String>| text.filter(|t| lambda_parameter_names.contains(t));
    for call in &call_expressions {
        for argument in call.value_arguments(ast) {
            let Some(expression) = argument.argument_expression(ast) else { continue };
            let forwarded = if KtReferenceExpression::is(ast, expression) {
                in_names(Some(ast.text(expression)))
            } else if let Some(if_expression) = KtIfExpression::cast(ast, expression) {
                in_names(if_expression.then(ast).map(|t| ast.text(t)))
                    .or_else(|| in_names(if_expression.r#else(ast).map(|e| ast.text(e))))
            } else {
                None
            };
            used.extend(forwarded);
        }
    }
    used
}

const LIFECYCLE_EFFECT_SCOPE_FUNCTIONS: &[&str] = &["onStopOrDispose", "onPauseOrDispose"];

pub const LAMBDA_USED_IN_RESTARTABLE_EFFECT: &str = "\
Lambda parameters in a @Composable that are referenced directly inside of restarting effects can cause issues or unpredictable behavior.
If restarting the effect is ok, you can add the reference to this parameter as a key in that effect, so when the parameter changes, a new effect is created.
However, if the effect is not to be restarted, you will need to use `rememberUpdatedState` on the parameter and use its result in the effect.
See https://mrmans0n.github.io/compose-rules/rules/#be-mindful-of-the-arguments-you-use-inside-of-a-restarting-effect for more information.";
