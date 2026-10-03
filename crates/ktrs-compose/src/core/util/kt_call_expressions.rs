//! Port of `core/util/KtCallExpressions.kt`.

use ktrs_ast::psi::{
    KtCallExpression, KtCallableDeclaration, KtDeclarationWithInitializer, KtDestructuringDeclaration,
    KtDotQualifiedExpression, KtReferenceExpression,
};
use ktrs_ast::{Ast, NodeId};

use crate::core::util::kt_dot_qualified_expressions::root_expression;
use crate::core::util::psi_elements::walk_backwards;

pub const DEFAULT_MODIFIER_TYPE_NAMES: &[&str] = &["Modifier", "GlanceModifier"];

fn contains(names: &[String], name: &str) -> bool {
    names.iter().any(|n| n == name)
}

fn insert(set: &mut Vec<String>, name: String) {
    if !set.contains(&name) {
        set.push(name);
    }
}

/// `KtCallExpression.parametersBeingUsedFrom(parameterNames, modifierTypeNames)`.
pub fn parameters_being_used_from(
    ast: &Ast,
    call: KtCallExpression,
    parameter_names: &[String],
    modifier_type_names: &[String],
) -> Vec<String> {
    let mut used = Vec::new();
    for argument in call.value_arguments(ast) {
        let Some(expression) = argument.argument_expression(ast) else { continue };
        if KtReferenceExpression::is(ast, expression) {
            let text = ast.text(expression);
            if contains(parameter_names, &text) {
                insert(&mut used, text);
            }
        } else if let Some(dot) = KtDotQualifiedExpression::cast(ast, expression) {
            for name in parameter_names_used_in(ast, dot, parameter_names, modifier_type_names) {
                insert(&mut used, name);
            }
        }
    }
    used
}

/// `KtDotQualifiedExpression.parameterNamesUsedIn(parameterNames, modifierTypeNames)`.
fn parameter_names_used_in(
    ast: &Ast,
    expression: KtDotQualifiedExpression,
    parameter_names: &[String],
    modifier_type_names: &[String],
) -> Vec<String> {
    let mut set = Vec::new();
    let root_text = ast.text(root_expression(ast, expression));
    if contains(parameter_names, &root_text) {
        insert(&mut set, root_text.clone());
    }
    let should_scan_then_args = contains(parameter_names, &root_text) || contains(modifier_type_names, &root_text);
    if should_scan_then_args {
        let mut current = Some(expression);
        while let Some(dot) = current {
            let selector = dot.selector_expression(ast).and_then(|s| KtCallExpression::cast(ast, s));
            if let Some(selector) = selector.filter(|s| callee_text(ast, *s).as_deref() == Some("then")) {
                for arg in selector.value_arguments(ast) {
                    let Some(expr) = arg.argument_expression(ast) else { continue };
                    if KtReferenceExpression::is(ast, expr) {
                        let text = ast.text(expr);
                        if contains(parameter_names, &text) {
                            insert(&mut set, text);
                        }
                    } else if let Some(nested) = KtDotQualifiedExpression::cast(ast, expr) {
                        let root = ast.text(root_expression(ast, nested));
                        if contains(parameter_names, &root) {
                            insert(&mut set, root);
                        }
                    }
                }
            }
            current = dot.receiver_expression(ast).and_then(|r| KtDotQualifiedExpression::cast(ast, r));
        }
    }
    set
}

fn callee_text(ast: &Ast, call: KtCallExpression) -> Option<String> {
    call.callee_expression(ast).map(|c| ast.text(c))
}

/// The names a parameter declares: its own, or a destructured parameter's entry names.
fn parameter_declared_names(ast: &Ast, parameter: ktrs_ast::psi::KtParameter) -> Vec<String> {
    if let Some(name) = parameter.name(ast) {
        return vec![name];
    }
    parameter
        .destructuring_declaration(ast)
        .map(|d| d.entries(ast).into_iter().filter_map(|it| it.name(ast)).collect())
        .unwrap_or_default()
}

/// `KtCallExpression.ancestorsParameterNamesSequence(stopAt)`.
fn ancestors_parameter_names_sequence(ast: &Ast, call: KtCallExpression, stop_at: NodeId) -> Vec<String> {
    ast.parents(call.node())
        .take_while(|&it| it != stop_at)
        .filter_map(|it| KtCallableDeclaration::cast(ast, it))
        .flat_map(|it| it.value_parameters(ast))
        .flat_map(|parameter| parameter_declared_names(ast, parameter))
        .collect()
}

/// `KtCallExpression.walkbackDeclarationsUntil(stopAt)`: (name, declaration) of every declaration with an
/// initializer before the call in its scopes.
fn walkback_declarations_until(ast: &Ast, call: KtCallExpression, stop_at: NodeId) -> Vec<(String, KtDeclarationWithInitializer)> {
    walk_backwards(ast, call.node(), Some(stop_at))
        .into_iter()
        .filter_map(|it| KtDeclarationWithInitializer::cast(ast, it))
        .flat_map(|declaration| {
            let names = match ktrs_ast::psi::name(ast, declaration.node()) {
                Some(name) => vec![name],
                None => match KtDestructuringDeclaration::cast(ast, declaration.node()) {
                    Some(destructuring) => destructuring.entries(ast).into_iter().filter_map(|it| it.name(ast)).collect(),
                    None => Vec::new(),
                },
            };
            names.into_iter().map(move |name| (name, declaration))
        })
        .collect()
}

/// `KtCallExpression.findShadowingRedeclarations(parameterName, stopAt)`.
pub fn find_shadowing_redeclarations(
    ast: &Ast,
    call: KtCallExpression,
    parameter_name: &str,
    stop_at: NodeId,
) -> Vec<KtDeclarationWithInitializer> {
    walkback_declarations_until(ast, call, stop_at).into_iter().filter(|(name, _)| name == parameter_name).map(|(_, d)| d).collect()
}

/// `KtCallExpression.isFullyShadowed(parameterNames, origin, modifierTypeNames)`: every parameter name the call
/// uses is redeclared by a parameter of an enclosing callable below `origin`.
pub fn is_fully_shadowed(
    ast: &Ast,
    call: KtCallExpression,
    parameter_names: &[String],
    origin: NodeId,
    modifier_type_names: &[String],
) -> bool {
    let current_names = parameters_being_used_from(ast, call, parameter_names, modifier_type_names);
    if current_names.is_empty() {
        return false;
    }
    let ancestor_names = ancestors_parameter_names_sequence(ast, call, origin);
    current_names.iter().all(|it| ancestor_names.contains(it))
}

/// [`DEFAULT_MODIFIER_TYPE_NAMES`] as the `Set<String>` the default arguments pass.
pub fn default_modifier_type_names() -> Vec<String> {
    DEFAULT_MODIFIER_TYPE_NAMES.iter().map(|s| s.to_string()).collect()
}
