//! Port of `core/util/Modifiers.kt`.

use ktrs_ast::psi::{
    KtBlockExpression, KtCallExpression, KtCallableDeclaration, KtDotQualifiedExpression, KtFunction, KtParameter,
    KtProperty, KtReferenceExpression, KtValueArgument,
};
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CALL_EXPRESSION, VALUE_ARGUMENT_NAME};

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::util::kt_dot_qualified_expressions::root_expression;
use crate::core::util::psi_elements::find_all_children;

fn insert(set: &mut Vec<String>, name: String) {
    if !set.contains(&name) {
        set.push(name);
    }
}

/// `KtBlockExpression.obtainAllModifierNames(initialName)`: the modifier's name and every alias reassigned from
/// it, until stable.
pub fn obtain_all_modifier_names(ast: &Ast, block: KtBlockExpression, initial_name: &str) -> Vec<String> {
    let root_block = block;
    let mut last_size = 0;
    let mut temp_modifier_names = vec![initial_name.to_owned()];
    while last_size < temp_modifier_names.len() {
        last_size = temp_modifier_names.len();
        let found = find_modifier_manipulations(ast, block, |it| temp_modifier_names.iter().any(|n| n == it));
        found.into_iter().for_each(|n| insert(&mut temp_modifier_names, n));
        // `+= sequence` adds lazily: a later block already sees the names found in earlier ones.
        for child_block in find_all_children::<KtBlockExpression>(ast, block.node()) {
            let shadowed = shadowed_modifier_names(ast, child_block, &temp_modifier_names, root_block);
            let accessible: Vec<String> = temp_modifier_names.iter().filter(|n| !shadowed.contains(n)).cloned().collect();
            if !accessible.is_empty() {
                let found = find_modifier_manipulations(ast, child_block, |it| accessible.iter().any(|n| n == it));
                found.into_iter().for_each(|n| insert(&mut temp_modifier_names, n));
            }
        }
    }
    temp_modifier_names
}

/// `KtBlockExpression.shadowedModifierNames(modifierNames, stopAt)`: modifier names redeclared by a parameter of a
/// function (lambda or nested) between this block and `stop_at`.
fn shadowed_modifier_names(ast: &Ast, block: KtBlockExpression, modifier_names: &[String], stop_at: KtBlockExpression) -> Vec<String> {
    let mut set = Vec::new();
    for function in ast.parents(block.node()).take_while(|&it| it != stop_at.node()).filter_map(|it| KtFunction::cast(ast, it)) {
        for param in function.value_parameters(ast) {
            for name in shadowing_names(ast, param, modifier_names) {
                insert(&mut set, name);
            }
        }
    }
    set
}

/// A parameter's names (its own, or a destructured one's entries) that are modifier names.
pub(crate) fn shadowing_names(ast: &Ast, param: KtParameter, modifier_names: &[String]) -> Vec<String> {
    let is_modifier_name = |name: &String| modifier_names.contains(name);
    if let Some(name) = param.name(ast) {
        return Some(name).filter(is_modifier_name).into_iter().collect();
    }
    param
        .destructuring_declaration(ast)
        .map(|d| d.entries(ast).into_iter().filter_map(|it| it.name(ast).filter(is_modifier_name)).collect())
        .unwrap_or_default()
}

/// `KtBlockExpression.findModifierManipulations(contains)`: names of the block's properties whose initializer
/// references a modifier name (outside a call's callee or an argument name).
fn find_modifier_manipulations(ast: &Ast, block: KtBlockExpression, contains: impl Fn(&str) -> bool) -> Vec<String> {
    block
        .statements(ast)
        .into_iter()
        .filter_map(|it| KtProperty::cast(ast, it))
        .flat_map(|property| {
            find_all_children::<KtReferenceExpression>(ast, property.node())
                .into_iter()
                .filter(|reference| {
                    let parent = ast.tree_parent(reference.node()).map(|p| ast.element_type(p));
                    parent != Some(CALL_EXPRESSION) && parent != Some(VALUE_ARGUMENT_NAME) && contains(&reference.text(ast))
                })
                .map(move |_| property)
                .collect::<Vec<_>>()
        })
        .filter_map(|it| it.name_identifier(ast).map(|n| ast.text(n)))
        .collect()
}

pub fn is_using_modifiers(ast: &Ast, call: KtCallExpression, modifier_names: &[String], modifier_type_names: &[String]) -> bool {
    !arguments_using_modifiers(ast, call, modifier_names, modifier_type_names).is_empty()
}

/// `KtCallExpression.argumentsUsingModifiers(modifierNames, modifierTypeNames)`.
pub fn arguments_using_modifiers(
    ast: &Ast,
    call: KtCallExpression,
    modifier_names: &[String],
    modifier_type_names: &[String],
) -> Vec<KtValueArgument> {
    call.value_arguments(ast)
        .into_iter()
        .filter(|argument| {
            let Some(expression) = argument.argument_expression(ast) else { return false };
            if KtReferenceExpression::is(ast, expression) {
                return modifier_names.contains(&ast.text(expression));
            }
            let Some(dot) = KtDotQualifiedExpression::cast(ast, expression) else { return false };
            let root_text = ast.text(root_expression(ast, dot));
            modifier_names.contains(&root_text)
                || (modifier_type_names.contains(&root_text) && has_modifier_as_chain_argument(ast, dot, modifier_names))
        })
        .collect()
}

/// `KtDotQualifiedExpression.hasModifierAsChainArgument(modifierNames)`: a modifier name is a direct argument of a
/// `.then()` anywhere in the chain.
fn has_modifier_as_chain_argument(ast: &Ast, expression: KtDotQualifiedExpression, modifier_names: &[String]) -> bool {
    let mut current = Some(expression);
    while let Some(dot) = current {
        let selector = dot.selector_expression(ast).and_then(|s| KtCallExpression::cast(ast, s));
        if let Some(selector) = selector.filter(|s| s.callee_expression(ast).is_some_and(|c| ast.text(c) == "then")) {
            for arg in selector.value_arguments(ast) {
                let Some(expr) = arg.argument_expression(ast) else { continue };
                if KtReferenceExpression::is(ast, expr) {
                    if modifier_names.contains(&ast.text(expr)) {
                        return true;
                    }
                } else if let Some(nested) = KtDotQualifiedExpression::cast(ast, expr)
                    && modifier_names.contains(&ast.text(root_expression(ast, nested)))
                {
                    return true;
                }
            }
        }
        current = dot.receiver_expression(ast).and_then(|r| KtDotQualifiedExpression::cast(ast, r));
    }
    false
}

pub const MODIFIER_NAMES: &[&str] = &["Modifier", "GlanceModifier"];

/// `modifierTypeNames(config)`: `Modifier`, `GlanceModifier` and the configured custom modifiers.
pub fn modifier_type_names(config: &dyn ComposeKtConfig) -> Vec<String> {
    let mut names: Vec<String> = MODIFIER_NAMES.iter().map(|s| s.to_string()).collect();
    for name in config.get_set("customModifiers", &[]) {
        insert(&mut names, name);
    }
    names
}

/// `KtCallableDeclaration.isModifier(config)`: the declared type is a modifier type name.
pub fn is_modifier(ast: &Ast, callable: NodeId, config: &dyn ComposeKtConfig) -> bool {
    KtCallableDeclaration::of(ast, callable)
        .type_reference(ast)
        .is_some_and(|t| modifier_type_names(config).contains(&t.text(ast)))
}

/// `KtCallableDeclaration.isModifierReceiver(config)`.
pub fn is_modifier_receiver(ast: &Ast, callable: NodeId, config: &dyn ComposeKtConfig) -> bool {
    KtCallableDeclaration::of(ast, callable)
        .receiver_type_reference(ast)
        .is_some_and(|t| modifier_type_names(config).contains(&t.text(ast)))
}

/// `KtFunction.modifierParameter(config)`: the modifier parameter named `modifier`, else the first one.
pub fn modifier_parameter(ast: &Ast, function: KtFunction, config: &dyn ComposeKtConfig) -> Option<KtParameter> {
    let modifiers = modifier_parameters(ast, function, config);
    modifiers.iter().copied().find(|it| it.name(ast).as_deref() == Some("modifier")).or_else(|| modifiers.first().copied())
}

pub fn modifier_parameters(ast: &Ast, function: KtFunction, config: &dyn ComposeKtConfig) -> Vec<KtParameter> {
    function.value_parameters(ast).into_iter().filter(|it| is_modifier(ast, it.node(), config)).collect()
}
