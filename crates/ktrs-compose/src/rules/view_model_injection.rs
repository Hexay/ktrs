//! Port of `rules/ViewModelInjection.kt`.

use ktrs_ast::psi::{self, KtCallExpression, KtFunction, KtFunctionType, KtProperty, kt_psi_factory};
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::WHITE_SPACE;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::ast_nodes::{first_child_leaf_or_self, last_child_leaf_or_self, next_code_sibling};
use crate::core::util::kt_functions::{defined_in_interface, is_override};
use crate::core::util::psi_elements::{ChildrenByClass, find_direct_children_by_class, find_direct_first_child_by_class};

pub struct ViewModelInjection;

impl ComposeKtVisitor for ViewModelInjection {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if is_override(ast, function) || defined_in_interface(ast, function) {
            return;
        }
        let Some(body_block) = function.body_block_expression(ast) else { return };
        let mut known_view_model_factories: Vec<String> = DEFAULT_KNOWN_VIEW_MODEL_FACTORIES.iter().map(|s| (*s).to_owned()).collect();
        for factory in config.get_set("viewModelFactories", &[]) {
            if !known_view_model_factories.contains(&factory) {
                known_view_model_factories.push(factory);
            }
        }

        // Lazy upstream, and the fix deletes the property: step the walk.
        let mut properties = ChildrenByClass::new(body_block.node());
        while let Some(property) = properties.next::<KtProperty>(ast, |_, _| true) {
            let matches: Vec<String> = find_direct_children_by_class::<KtCallExpression>(ast, property.node())
                .into_iter()
                .filter_map(|it| {
                    let callee = ast.text(it.callee_expression(ast)?);
                    known_view_model_factories.contains(&callee).then_some((it, callee))
                })
                .filter(|&(it, _)| !is_navigation(ast, it, body_block.node()))
                .map(|(_, callee)| callee)
                .collect();
            for view_model_factory_name in matches {
                emitter.report(ast, property.node(), &error_message(&view_model_factory_name), true).if_fix(|| {
                    fix(ast, function, property, &view_model_factory_name);
                });
            }
        }
    }
}

fn fix(ast: &mut Ast, composable: KtFunction, property: KtProperty, view_model_factory_name: &str) {
    let variable_name = property.name(ast).unwrap_or_else(|| "null".to_owned());
    let Some(call_expression) = find_direct_first_child_by_class::<KtCallExpression>(ast, property.node()) else { return };
    let Some(argument_list) = call_expression.value_argument_list(ast) else { return };
    if !call_expression.value_arguments(ast).is_empty() {
        return;
    }
    let view_model_type_reference = match property.type_reference(ast) {
        Some(type_reference) => type_reference.node(),
        None => match call_expression.type_arguments(ast).as_slice() {
            [single] => *single,
            _ => return,
        },
    };

    let raw_view_model_type = ast.text(view_model_type_reference);
    let raw_argument_list = ast.text(argument_list.node());
    let value_parameters = composable.value_parameters(ast);
    let last_parameters = &value_parameters[value_parameters.len().saturating_sub(2)..];
    let Some(parameter_list) = composable.value_parameter_list(ast) else { return };

    let new_code = format!("{variable_name}: {raw_view_model_type} = {view_model_factory_name}{raw_argument_list}");
    let new_param = kt_psi_factory::create_parameter(ast, &new_code);
    let new_param_text = new_param.text(ast);

    let last_is_function_type = last_parameters
        .last()
        .and_then(|it| it.type_reference(ast))
        .and_then(|it| it.type_element(ast))
        .is_some_and(|it| KtFunctionType::is(ast, it.node()));
    if last_parameters.is_empty() {
        let last_token = leaf(ast, last_child_leaf_or_self(ast, parameter_list.node()));
        ast.raw_replace_with_text(last_token, &format!("{new_param_text})"));
    } else if last_is_function_type {
        if last_parameters.len() == 1 {
            let first_token = leaf(ast, first_child_leaf_or_self(ast, parameter_list.node()));
            ast.raw_replace_with_text(first_token, &format!("({new_code}, "));
        } else {
            let comma = next_code_sibling(ast, last_parameters[0].node()).expect("NullPointerException: nextCodeSibling()!!");
            let last_token = leaf(ast, last_child_leaf_or_self(ast, comma));
            let text = format!("{} {new_code},", ast.text(last_token));
            ast.raw_replace_with_text(last_token, &text);
        }
    } else {
        let last_parameter = value_parameters.last().expect("NoSuchElementException: List is empty.");
        let has_trailing_comma = next_code_sibling(ast, last_parameter.node()).is_some_and(|it| ast.text(it) == ",");
        let pre_comma_if_needed = if has_trailing_comma { "" } else { "," };
        let trailing_comma_if_needed = if has_trailing_comma { "," } else { "" };
        let last_token = leaf(ast, last_child_leaf_or_self(ast, parameter_list.node()));
        ast.raw_replace_with_text(last_token, &format!("{pre_comma_if_needed}{new_param_text}{trailing_comma_if_needed})"));
    }

    if let Some(previous) = ast.tree_prev(property.node()).filter(|&it| ast.element_type(it) == WHITE_SPACE) {
        psi::delete(ast, previous);
    }
    psi::delete(ast, property.node());
}

/// `node as LeafPsiElement`.
fn leaf(ast: &Ast, node: NodeId) -> NodeId {
    assert!(ast.is_leaf_element(node), "ClassCastException: {:?} cannot be cast to LeafPsiElement", ast.element_type(node));
    node
}

fn is_navigation(ast: &Ast, call: KtCallExpression, stop_at: NodeId) -> bool {
    ast.parents(call.node())
        .take_while(|&it| it != stop_at)
        .filter_map(|it| KtCallExpression::cast(ast, it))
        .any(|it| it.callee_expression(ast).is_some_and(|c| KNOWN_NAVIGATION_CALL_EXPRESSIONS.contains(&ast.text(c).as_str())))
}

const KNOWN_NAVIGATION_CALL_EXPRESSIONS: &[&str] = &["composable", "NavHost"];

const DEFAULT_KNOWN_VIEW_MODEL_FACTORIES: &[&str] = &[
    "viewModel",
    "weaverViewModel",
    "hiltViewModel",
    "injectedViewModel",
    "mavericksViewModel",
    "tangleViewModel",
    "metroViewModel",
    "anvilViewModel",
];

pub fn error_message(factory_name: &str) -> String {
    format!(
        "Implicit dependencies of composables should be made explicit.\n\
         Usages of {factory_name} to acquire a ViewModel should be done in composable default parameters, so that it is more testable and flexible.\n\
         See https://mrmans0n.github.io/compose-rules/rules/#viewmodels for more information."
    )
}
