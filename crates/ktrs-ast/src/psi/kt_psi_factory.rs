//! `KtPsiFactory`: each `create*` parses a snippet as `dummy.kt` into a new detached file element of the arena
//! and returns the requested node inside it (a `replaceChild` then moves it into the tree, as on the JVM). The
//! factory's project/context (`KtPsiFactory.contextual(e)`) only matters for analysis, so it is not modelled.

use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::SyntaxKind::*;

use super::classes::*;
use super::named::name_identifier;
use crate::arena::{Ast, NodeId};

/// `createFile(text)`: the new file element.
pub fn create_file(ast: &mut Ast, text: &str) -> KtFile {
    let parse = parse_file(text, FileKind::Source);
    KtFile(ast.seed(&parse))
}

/// `createDeclaration(text)`: the file's only declaration; upstream fails `checkWithAttachment` otherwise.
pub fn create_declaration(ast: &mut Ast, text: &str) -> NodeId {
    let file = create_file(ast, text);
    let declarations: Vec<NodeId> = ast.children(file.node()).filter(|&c| KtDeclaration::is(ast, c)).collect();
    assert!(
        declarations.len() == 1,
        "KotlinExceptionWithAttachments: unexpected {} declarations",
        declarations.len()
    );
    declarations[0]
}

pub fn create_class(ast: &mut Ast, text: &str) -> KtClass {
    let declaration = create_declaration(ast, text);
    KtClass::of(ast, declaration)
}

pub fn create_property(ast: &mut Ast, text: &str) -> KtProperty {
    let declaration = create_declaration(ast, text);
    KtProperty::of(ast, declaration)
}

/// `createProperty(modifiers, name, type, isVar, initializer)`. Gotcha: a null `modifiers` renders as `"null "`
/// (`modifiers.let { "$it " }`), so the snippet starts with an error element; the declaration count is still one.
pub fn create_property_from_parts(
    ast: &mut Ast,
    modifiers: Option<&str>,
    name: &str,
    type_: Option<&str>,
    is_var: bool,
    initializer: Option<&str>,
) -> KtProperty {
    let text = format!(
        "{} {}{name}{}{}",
        modifiers.unwrap_or("null"),
        if is_var { " var " } else { " val " },
        type_.map(|t| format!(":{t}")).unwrap_or_default(),
        initializer.map(|i| format!(" = {i}")).unwrap_or_default()
    );
    create_property(ast, &text)
}

/// `createNameIdentifierIfPossible(name)`: `createProperty(name, null, false).nameIdentifier`.
pub fn create_name_identifier_if_possible(ast: &mut Ast, name: &str) -> Option<NodeId> {
    let property = create_property_from_parts(ast, None, name, None, false, None);
    name_identifier(ast, property.node())
}

/// `createNameIdentifier(name)`.
pub fn create_name_identifier(ast: &mut Ast, name: &str) -> NodeId {
    create_name_identifier_if_possible(ast, name).expect("NullPointerException: createNameIdentifier")
}

/// `createParameter(text)`: the first primary constructor parameter of `class A(<text>)`.
pub fn create_parameter(ast: &mut Ast, text: &str) -> KtParameter {
    let class = create_class(ast, &format!("class A({text})"));
    class
        .primary_constructor_parameters(ast)
        .into_iter()
        .next()
        .expect("NoSuchElementException: List is empty.")
}

/// `createParameterList(text)`: the value parameter list of `fun foo<text>{}`.
pub fn create_parameter_list(ast: &mut Ast, text: &str) -> KtParameterList {
    let declaration = create_declaration(ast, &format!("fun foo{text}{{}}"));
    let function = KtNamedFunction::of(ast, declaration);
    KtParameterList::of(ast, ast.find_child_by_type(function.node(), VALUE_PARAMETER_LIST).expect("NullPointerException"))
}
