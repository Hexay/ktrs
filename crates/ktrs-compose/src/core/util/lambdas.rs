//! Port of `core/util/Lambdas.kt`.

use ktrs_ast::psi::{KtClass, KtFunctionType, KtTypeAlias, KtTypeElement, KtTypeReference};
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::FUN_KEYWORD;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::util::kt_annotateds::is_composable;
use crate::core::util::psi_elements::find_all_children;

fn referenced_name_in(ast: &Ast, element: KtTypeElement, types: &[String]) -> bool {
    element.as_user_type(ast).and_then(|u| u.referenced_name(ast)).is_some_and(|name| types.contains(&name))
}

/// `KtTypeElement.isLambda(treatAsLambdaTypes)`.
pub fn type_element_is_lambda(ast: &Ast, element: KtTypeElement, treat_as_lambda_types: &[String]) -> bool {
    if element.as_function_type(ast).is_some() {
        return true;
    }
    if let Some(nullable) = element.as_nullable_type(ast) {
        return nullable.inner_type(ast).is_some_and(|inner| type_element_is_lambda(ast, inner, treat_as_lambda_types));
    }
    referenced_name_in(ast, element, treat_as_lambda_types)
}

/// `KtTypeReference.isLambda(treatAsLambdaTypes)`.
pub fn is_lambda(ast: &Ast, type_reference: KtTypeReference, treat_as_lambda_types: &[String]) -> bool {
    type_reference.type_element(ast).is_some_and(|e| type_element_is_lambda(ast, e, treat_as_lambda_types))
}

/// `KtTypeReference.isComposableLambda(treatAsLambdaTypes, treatAsComposableLambdaTypes)`.
pub fn is_composable_lambda(
    ast: &Ast,
    type_reference: KtTypeReference,
    treat_as_lambda_types: &[String],
    treat_as_composable_lambda_types: &[String],
) -> bool {
    composable_lambda_check(ast, type_reference, treat_as_lambda_types, treat_as_composable_lambda_types, false)
}

/// `KtTypeReference.isComposableUiEmitterLambda(...)`: like [`is_composable_lambda`], but a function type must
/// also return `Unit`.
pub fn is_composable_ui_emitter_lambda(
    ast: &Ast,
    type_reference: KtTypeReference,
    treat_as_lambda_types: &[String],
    treat_as_composable_lambda_types: &[String],
) -> bool {
    composable_lambda_check(ast, type_reference, treat_as_lambda_types, treat_as_composable_lambda_types, true)
}

/// The shared `when (val element = typeElement)` of the two functions above.
fn composable_lambda_check(
    ast: &Ast,
    type_reference: KtTypeReference,
    treat_as_lambda_types: &[String],
    treat_as_composable_lambda_types: &[String],
    function_type_returns_unit: bool,
) -> bool {
    let Some(element) = type_reference.type_element(ast) else { return false };
    let composable = || is_composable(ast, type_reference.node());
    if let Some(function_type) = element.as_function_type(ast) {
        return composable() && (!function_type_returns_unit || function_type_returns_unit_check(ast, function_type));
    }
    if let Some(nullable) = element.as_nullable_type(ast) {
        return (composable() && type_element_is_lambda(ast, element, treat_as_lambda_types))
            || nullable.inner_type(ast).is_some_and(|inner| referenced_name_in(ast, inner, treat_as_composable_lambda_types));
    }
    if element.as_user_type(ast).is_some() {
        return (composable() && referenced_name_in(ast, element, treat_as_lambda_types))
            || referenced_name_in(ast, element, treat_as_composable_lambda_types);
    }
    false
}

fn function_type_returns_unit_check(ast: &Ast, function_type: KtFunctionType) -> bool {
    function_type.return_type_reference(ast).is_some_and(|t| t.text(ast) == "Unit")
}

/// `KtTypeElement.returnsUnit`.
pub fn type_element_returns_unit(ast: &Ast, element: KtTypeElement) -> bool {
    if let Some(function_type) = element.as_function_type(ast) {
        return function_type_returns_unit_check(ast, function_type);
    }
    element.as_nullable_type(ast).and_then(|n| n.inner_type(ast)).is_some_and(|inner| type_element_returns_unit(ast, inner))
}

/// `KtTypeReference.returnsUnit`.
pub fn returns_unit(ast: &Ast, type_reference: KtTypeReference) -> bool {
    let Some(element) = type_reference.type_element(ast) else { return false };
    if let Some(function_type) = element.as_function_type(ast) {
        return function_type_returns_unit_check(ast, function_type);
    }
    element.as_nullable_type(ast).and_then(|n| n.inner_type(ast)).is_some_and(|inner| type_element_returns_unit(ast, inner))
}

fn fun_interfaces(ast: &Ast, file: NodeId) -> Vec<KtClass> {
    find_all_children::<KtClass>(ast, file)
        .into_iter()
        .filter(|it| it.is_interface(ast) && it.has_modifier(ast, FUN_KEYWORD))
        .collect()
}

fn add_all(set: &mut Vec<String>, items: impl IntoIterator<Item = String>) {
    for item in items {
        if !set.contains(&item) {
            set.push(item);
        }
    }
}

/// `KtFile.lambdaTypes(config)`: configured types, fun interfaces, then typealiases of lambdas (last, so that
/// `isLambda` sees the others).
pub fn lambda_types(ast: &Ast, file: NodeId, config: &dyn ComposeKtConfig) -> Vec<String> {
    let mut set = Vec::new();
    add_all(&mut set, config.get_set("treatAsLambda", &[]));
    add_all(&mut set, fun_interfaces(ast, file).into_iter().filter_map(|it| it.name(ast)));
    for alias in find_all_children::<KtTypeAlias>(ast, file) {
        if alias.type_reference(ast).is_some_and(|t| is_lambda(ast, t, &set)) {
            add_all(&mut set, alias.name(ast));
        }
    }
    set
}

/// `KtFile.composableLambdaTypes(config)`.
pub fn composable_lambda_types(ast: &Ast, file: NodeId, config: &dyn ComposeKtConfig) -> Vec<String> {
    let mut set = Vec::new();
    add_all(&mut set, config.get_set("treatAsComposableLambda", &[]));
    let composable_sams = fun_interfaces(ast, file).into_iter().filter(|fun_interface| {
        fun_interface
            .body(ast)
            .and_then(|body| body.functions(ast).into_iter().find(|it| !it.has_body(ast)))
            .is_some_and(|sam| is_composable(ast, sam.node()))
    });
    add_all(&mut set, composable_sams.filter_map(|it| it.name(ast)).collect::<Vec<_>>());
    for alias in find_all_children::<KtTypeAlias>(ast, file) {
        if is_composable_lambda_alias(ast, alias, &set) {
            add_all(&mut set, alias.name(ast));
        }
    }
    set
}

/// The typealias filter of `composableLambdaTypes`, against the types found so far.
fn is_composable_lambda_alias(ast: &Ast, alias: KtTypeAlias, found: &[String]) -> bool {
    let Some(type_reference) = alias.type_reference(ast) else { return false };
    let Some(type_element) = type_reference.type_element(ast) else { return false };
    if type_element.as_function_type(ast).is_some() {
        return is_composable(ast, type_reference.node());
    }
    if let Some(nullable) = type_element.as_nullable_type(ast) {
        // `typeElement.innerType?.name` is `PsiElementBase.getName()`: always null, never in the set.
        return is_composable(ast, type_reference.node())
            && nullable.inner_type(ast).is_some_and(|inner| inner.as_function_type(ast).is_some());
    }
    referenced_name_in(ast, type_element, found)
}
