//! Port of `core/util/KtFunctions.kt`. `KtModifierListOwner` receivers are bare nodes.

use ktrs_ast::psi::{EmbeddedKotlin, KtClass, KtFile, KtFunction, KtModifierListOwner, KtNamedFunction};
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{self, *};

/// `KtFunction.returnsValue`: a declared return type other than `Unit`.
pub fn returns_value(ast: &Ast, function: KtFunction) -> bool {
    function.type_reference(ast).is_some_and(|t| t.text(ast) != "Unit")
}

pub fn has_receiver_type(ast: &Ast, function: KtFunction) -> bool {
    function.receiver_type_reference(ast).is_some()
}

fn visibility_modifier_type(ast: &Ast, owner: NodeId) -> Option<SyntaxKind> {
    KtModifierListOwner::of(ast, owner).visibility_modifier_type(ast)
}

pub fn is_private(ast: &Ast, owner: NodeId) -> bool {
    visibility_modifier_type(ast, owner) == Some(PRIVATE_KEYWORD)
}

pub fn is_protected(ast: &Ast, owner: NodeId) -> bool {
    visibility_modifier_type(ast, owner) == Some(PROTECTED_KEYWORD)
}

pub fn is_internal(ast: &Ast, owner: NodeId) -> bool {
    visibility_modifier_type(ast, owner) == Some(INTERNAL_KEYWORD)
}

pub fn is_override(ast: &Ast, function: KtFunction) -> bool {
    function.has_modifier(ast, OVERRIDE_KEYWORD)
}

pub fn is_actual(ast: &Ast, function: KtFunction) -> bool {
    function.has_modifier(ast, ACTUAL_KEYWORD)
}

pub fn is_expect(ast: &Ast, function: KtFunction) -> bool {
    function.has_modifier(ast, EXPECT_KEYWORD)
}

pub fn is_abstract(ast: &Ast, function: KtFunction) -> bool {
    function.has_modifier(ast, ABSTRACT_KEYWORD)
}

pub fn is_open(ast: &Ast, function: KtFunction) -> bool {
    function.has_modifier(ast, OPEN_KEYWORD)
}

pub fn is_operator(ast: &Ast, function: KtFunction) -> bool {
    function.has_modifier(ast, OPERATOR_KEYWORD)
}

/// `KtFunction.definedInInterface`: directly in the body of an interface.
pub fn defined_in_interface(ast: &Ast, function: KtFunction) -> bool {
    ast.tree_parent(function.node())
        .filter(|&p| ast.element_type(p) == CLASS_BODY)
        .and_then(|body| ast.tree_parent(body))
        .and_then(|class| KtClass::cast(ast, class))
        .is_some_and(|class| class.is_interface(ast))
}

/// `KtNamedFunction.isNested`: a named function encloses it (below the file).
pub fn is_nested(ast: &Ast, function: KtNamedFunction) -> bool {
    ast.parents(function.node()).take_while(|&it| !KtFile::is(ast, it)).any(|it| KtNamedFunction::is(ast, it))
}

/// `KtNamedFunction.hasAnyContextArguments`: context receivers, context parameters, or text starting with
/// `context(`.
// Gotcha: upstream finds `getContextParameters` by reflection; ktlint 1.8.0's Kotlin 2.2.21 has no such method.
pub fn has_any_context_arguments(ast: &Ast, function: KtNamedFunction, kotlin: EmbeddedKotlin) -> bool {
    !function.context_receivers(ast).is_empty()
        || !context_parameters_or_empty(ast, function, kotlin).is_empty()
        || function.text(ast).trim_start().starts_with("context(")
}

fn context_parameters_or_empty(ast: &Ast, function: KtNamedFunction, kotlin: EmbeddedKotlin) -> Vec<NodeId> {
    match kotlin {
        EmbeddedKotlin::V2_2_21 => Vec::new(),
        EmbeddedKotlin::V2_4_10 => function.context_parameters(ast).into_iter().map(|p| p.node()).collect(),
    }
}
