//! Port of `rules/ViewModelForwarding.kt`.

use std::collections::HashSet;

use ktrs_ast::Ast;
use ktrs_ast::psi::{
    KtCallExpression, KtDotQualifiedExpression, KtFunction, KtNameReferenceExpression, KtReferenceExpression, KtThisExpression,
};
use ktrs_lint::rules::internal::KotlinRegex;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::is_restartable_effect;
use crate::core::util::kotlin_utils::{KOTLIN_IT_OBJECT_SCOPE_FUNCTIONS, KOTLIN_SCOPE_FUNCTIONS, join_to_regex, join_to_regex_or_null};
use crate::core::util::kt_functions::{defined_in_interface, is_actual, is_override};
use crate::core::util::psi_elements::{find_all_children, find_direct_children_by_class};

pub struct ViewModelForwarding;

impl ComposeKtVisitor for ViewModelForwarding {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if is_override(ast, function) || defined_in_interface(ast, function) || is_actual(ast, function) {
            return;
        }
        let Some(body_block) = function.body_block_expression(ast) else { return };
        let parameters = function.value_parameter_list(ast).map(|l| l.parameters(ast)).unwrap_or_default();
        if parameters.is_empty() {
            return;
        }
        let mut state_holder_names = config.get_set("allowedStateHolderNames", &[]);
        for name in DEFAULT_STATE_HOLDER_NAMES {
            if !state_holder_names.iter().any(|it| it == name) {
                state_holder_names.push((*name).to_owned());
            }
        }
        let state_holder_valid_names = join_to_regex(&state_holder_names);
        let allowed_forwarding_target_names = join_to_regex_or_null(&config.get_set("allowedForwarding", &[]));
        let allowed_forwarding_of_types = join_to_regex_or_null(&config.get_set("allowedForwardingOfTypes", &[]));

        let type_text_matches = |parameter: &ktrs_ast::psi::KtParameter, regex: &KotlinRegex| {
            parameter.type_reference(ast).is_some_and(|t| regex.matches(&t.text(ast)))
        };
        let mut view_model_parameter_names: Vec<String> = Vec::new();
        for parameter in &parameters {
            if !type_text_matches(parameter, &state_holder_valid_names) {
                continue;
            }
            if allowed_forwarding_of_types.as_ref().is_some_and(|regex| type_text_matches(parameter, regex)) {
                continue;
            }
            if let Some(name) = parameter.name(ast)
                && !view_model_parameter_names.contains(&name)
            {
                view_model_parameter_names.push(name);
            }
        }

        let mut scan = Scan {
            view_model_parameter_names,
            allowed_forwarding_target_names,
            already_processed_call_expressions: HashSet::new(),
            emitter,
        };
        let call_expressions = find_all_children::<KtCallExpression>(ast, body_block.node());
        scan.scan_call_expressions(ast, &call_expressions, None, false);
    }
}

/// The state of upstream's local `scanCallExpressions`. Its sequences are lazy: `filterNot { it in
/// alreadyProcessedCallExpressions }` is checked per element as each pass reaches it.
struct Scan<'a> {
    view_model_parameter_names: Vec<String>,
    allowed_forwarding_target_names: Option<KotlinRegex>,
    already_processed_call_expressions: HashSet<KtCallExpression>,
    emitter: &'a mut dyn Emitter,
}

impl Scan<'_> {
    fn scan_call_expressions(
        &mut self,
        ast: &Ast,
        call_expressions: &[KtCallExpression],
        scoped_parameter: Option<String>,
        uses_it_object_ref: bool,
    ) {
        for &call_expression in call_expressions {
            if self.already_processed_call_expressions.contains(&call_expression) || !is_scope_function(ast, call_expression) {
                continue;
            }
            let lambda_bodies: Vec<_> = call_expression
                .lambda_arguments(ast)
                .into_iter()
                .filter_map(|it| it.lambda_expression(ast)?.body_expression(ast))
                .collect();
            for lambda_body_expression in lambda_bodies {
                let expressions = find_direct_children_by_class::<KtCallExpression>(ast, lambda_body_expression.node());
                self.scan_call_expressions(
                    ast,
                    &expressions,
                    get_scoped_parameter_value(ast, call_expression),
                    has_it_object_reference(ast, call_expression),
                );
            }
        }

        for &call_expression in call_expressions {
            if self.already_processed_call_expressions.contains(&call_expression) {
                continue;
            }
            let Some(callee) = call_expression.callee_expression(ast).map(|c| ast.text(c)) else { continue };
            let first = callee.chars().next().expect("NoSuchElementException: Char sequence is empty.");
            if !(first.len_utf16() == 1 && first.is_uppercase()) {
                continue;
            }
            if is_restartable_effect(ast, call_expression) {
                continue;
            }
            if self.allowed_forwarding_target_names.as_ref().is_some_and(|regex| regex.matches(&callee)) {
                continue;
            }
            self.already_processed_call_expressions.insert(call_expression);
            let scoped_in_vm_params =
                scoped_parameter.as_ref().is_some_and(|p| self.view_model_parameter_names.contains(p));
            let matches = call_expression
                .value_arguments(ast)
                .into_iter()
                .filter_map(|it| it.argument_expression(ast))
                .filter(|&it| KtReferenceExpression::is(ast, it) || KtThisExpression::is(ast, it))
                .filter(|&argument_expression| {
                    let text = ast.text(argument_expression);
                    let is_it_ref_and_scoped_in_vm_params = uses_it_object_ref && text == "it" && scoped_in_vm_params;
                    let is_this_ref_and_scoped_in_vm_params = !uses_it_object_ref && text == "this" && scoped_in_vm_params;
                    self.view_model_parameter_names.contains(&text)
                        || is_it_ref_and_scoped_in_vm_params
                        || is_this_ref_and_scoped_in_vm_params
                })
                .count();
            for _ in 0..matches {
                self.emitter.report(ast, call_expression.node(), AVOID_VIEW_MODEL_FORWARDING, false);
            }
        }
    }
}

fn referenced_name(ast: &Ast, call: KtCallExpression) -> Option<String> {
    let callee = call.callee_expression(ast)?;
    Some(KtNameReferenceExpression::cast(ast, callee)?.referenced_name(ast))
}

fn is_scope_function(ast: &Ast, call: KtCallExpression) -> bool {
    referenced_name(ast, call).is_some_and(|n| KOTLIN_SCOPE_FUNCTIONS.contains(&n.as_str()))
}

fn is_with_scope(ast: &Ast, call: KtCallExpression) -> bool {
    referenced_name(ast, call).as_deref() == Some("with")
}

fn has_it_object_reference(ast: &Ast, call: KtCallExpression) -> bool {
    referenced_name(ast, call).is_some_and(|n| KOTLIN_IT_OBJECT_SCOPE_FUNCTIONS.contains(&n.as_str()))
}

fn get_scoped_parameter_value(ast: &Ast, call: KtCallExpression) -> Option<String> {
    if is_with_scope(ast, call) {
        call.value_arguments(ast).first()?.argument_expression(ast).map(|e| ast.text(e))
    } else {
        let parent = KtDotQualifiedExpression::cast(ast, ast.tree_parent(call.node())?)?;
        parent.receiver_expression(ast).map(|e| ast.text(e))
    }
}

const DEFAULT_STATE_HOLDER_NAMES: &[&str] = &[".*ViewModel", ".*Presenter"];

pub const AVOID_VIEW_MODEL_FORWARDING: &str = "\
Forwarding a ViewModel/Presenter through multiple @Composable functions should be avoided. Consider using state hoisting.
See https://mrmans0n.github.io/compose-rules/rules/#hoist-all-the-things for more information.";
