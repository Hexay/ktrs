//! Port of `rules/ParameterOrder.kt`.

use std::cmp::Reverse;

use ktrs_ast::Ast;
use ktrs_ast::psi::{KtFile, KtFunction, KtParameter};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::lambdas::{composable_lambda_types, is_composable_lambda, is_lambda, lambda_types};
use crate::core::util::modifiers::is_modifier;
use crate::core::util::psi_elements::find_all_children;

pub struct ParameterOrder;

impl ComposeKtVisitor for ParameterOrder {
    fn visit_file(&self, ast: &mut Ast, file: KtFile, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        let lambda_types = lambda_types(ast, file.node(), config);
        let composable_lambda_types = composable_lambda_types(ast, file.node(), config);
        let composables: Vec<_> =
            find_all_children::<KtFunction>(ast, file.node()).into_iter().filter(|it| is_composable(ast, it.node())).collect();
        for function in composables {
            let current_order = function.value_parameters(ast);
            let has_trailing_function = current_order
                .last()
                .and_then(|it| it.type_reference(ast))
                .is_some_and(|t| is_lambda(ast, t, &lambda_types));
            let trailing_lambda: Vec<KtParameter> =
                if has_trailing_function { vec![*current_order.last().unwrap()] } else { Vec::new() };
            let leading = if has_trailing_function { &current_order[..current_order.len() - 1] } else { &current_order[..] };
            let (mut with_defaults, without_defaults): (Vec<KtParameter>, Vec<KtParameter>) =
                leading.iter().partition(|it| it.has_default_value(ast));
            // Stable, like `sortedWith`.
            with_defaults.sort_by_key(|it| {
                (Reverse(is_modifier(ast, it.node(), config)), Reverse(it.name(ast).as_deref() == Some("modifier")))
            });
            let proper_order: Vec<KtParameter> =
                without_defaults.into_iter().chain(with_defaults).chain(trailing_lambda).collect();
            let should_be_lenient = !has_trailing_function
                && proper_order.last().is_some_and(|last_param| {
                    let type_ref = last_param.type_reference(ast);
                    last_param.has_default_value(ast)
                        && type_ref.is_some_and(|t| {
                            is_lambda(ast, t, &lambda_types)
                                && !is_composable_lambda(ast, t, &lambda_types, &composable_lambda_types)
                        })
                });
            if !should_be_lenient && current_order != proper_order {
                let message = create_error_message(ast, &current_order, &proper_order);
                emitter.report(ast, function.node(), &message, false);
            }
        }
    }
}

fn join_texts(ast: &Ast, parameters: &[KtParameter]) -> String {
    parameters.iter().map(|it| it.text(ast)).collect::<Vec<_>>().join(", ")
}

pub fn create_error_message(ast: &Ast, current_order: &[KtParameter], proper_order: &[KtParameter]) -> String {
    create_error_message_text(&join_texts(ast, current_order), &join_texts(ast, proper_order))
}

/// The raw string is interpolated before `trimIndent()`, so multi-line parameter texts change the common indent.
pub fn create_error_message_text(current_order: &str, proper_order: &str) -> String {
    trim_indent(&format!(
        "
            Parameters in a composable function should be ordered following this pattern: params without defaults, modifiers, params with defaults and optionally, a trailing function that might not have a default param.
            Current params are: [{current_order}] but could be [{proper_order}].
            See https://mrmans0n.github.io/compose-rules/rules/#ordering-composable-parameters-properly for more information.
        "
    ))
}

/// Kotlin `String.trimIndent()`.
fn trim_indent(s: &str) -> String {
    let lines: Vec<&str> = s.split("\r\n").flat_map(|l| l.split(['\n', '\r'])).collect();
    let is_blank = |l: &str| l.chars().all(char::is_whitespace);
    let indent_width = |l: &str| l.chars().position(|c| !c.is_whitespace()).unwrap_or(l.chars().count());
    let min_common_indent = lines.iter().filter(|l| !is_blank(l)).map(|l| indent_width(l)).min().unwrap_or(0);
    let last_index = lines.len() - 1;
    lines
        .iter()
        .enumerate()
        .filter(|&(index, line)| !((index == 0 || index == last_index) && is_blank(line)))
        .map(|(_, line)| line.chars().skip(min_common_indent).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
