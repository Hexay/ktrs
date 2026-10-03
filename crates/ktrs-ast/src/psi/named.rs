//! `PsiNameIdentifierOwner` / `KtNamedDeclaration`: `getName`, `getNameIdentifier`, `getNameAsSafeName`,
//! `setName` (`KtNamedDeclarationStub`, `KtNamedDeclarationNotStubbed`, `KtConstructor`, `KtFunctionLiteral`,
//! `KtObjectDeclaration` overrides).

use ktrs_psi::classes::is_named_declaration;
use ktrs_psi::unquote_identifier;
use ktrs_syntax::SyntaxKind::*;

use super::EmbeddedKotlin;
use super::classes::*;
use super::kt_psi_factory;
use super::kt_psi_util::quote_if_needed;
use crate::arena::{Ast, NodeId};

/// `SpecialNames.NO_NAME_PROVIDED`.
pub const NO_NAME_PROVIDED: &str = "<no name provided>";
/// `SpecialNames.ANONYMOUS_STRING`.
pub const ANONYMOUS_STRING: &str = "<anonymous>";

/// `element is PsiNameIdentifierOwner`: the named declarations, `KtImportAlias`, `KtLabeledExpression`.
pub fn is_name_identifier_owner(ast: &Ast, element: NodeId) -> bool {
    !ast.is_leaf_element(element)
        && (is_named_declaration(ast.element_type(element)) || matches!(ast.element_type(element), IMPORT_ALIAS | LABELED_EXPRESSION))
}

/// `PsiNameIdentifierOwner.getNameIdentifier()`, dispatched to the implementing class.
pub fn name_identifier(ast: &Ast, owner: NodeId) -> Option<NodeId> {
    match ast.element_type(owner) {
        FUNCTION_LITERAL | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR | SCRIPT => None,
        LABELED_EXPRESSION => ast.find_child_by_type(ast.find_child_by_type(owner, LABEL_QUALIFIER)?, LABEL)
            .and_then(|label| ast.find_child_by_type(label, IDENTIFIER)),
        _ => ast.find_child_by_type(owner, IDENTIFIER),
    }
}

/// `getName()`, dispatched to the implementing class; `PsiElementBase.getName()` (null) for the others.
pub fn name(ast: &Ast, element: NodeId) -> Option<String> {
    match ast.element_type(element) {
        FUNCTION_LITERAL => Some(ANONYMOUS_STRING.to_owned()),
        PRIMARY_CONSTRUCTOR => name(ast, ast.tree_parent(element)?),
        SECONDARY_CONSTRUCTOR => name(ast, ast.tree_parent(ast.tree_parent(element)?)?),
        OBJECT_DECLARATION => identifier_name(ast, element).or_else(|| {
            let object = KtObjectDeclaration(element);
            (object.is_companion(ast) && !object.is_top_level(ast)).then(|| "Companion".to_owned())
        }),
        kind if is_named_declaration(kind) && kind != SCRIPT => identifier_name(ast, element),
        _ => None,
    }
}

fn identifier_name(ast: &Ast, element: NodeId) -> Option<String> {
    Some(unquote_identifier(&ast.text(name_identifier(ast, element)?)))
}

/// `getNameAsSafeName().asString()`: the name, or `<no name provided>`.
pub fn name_as_safe_name(ast: &Ast, element: NodeId) -> String {
    name(ast, element).unwrap_or_else(|| NO_NAME_PROVIDED.to_owned())
}

/// `KtNamedDeclarationStub.setName(name)` as the jars run it (2.4.20 moved it behind `KtPsiMutationService`):
/// replaces the name identifier with one parsed from `quoteIfNeeded(name)`, or deletes it when none parses.
/// Returns `None` (upstream `null`) without a name identifier.
// TODO: the `operator` modifier is not dropped when the new name is no operator convention name
// (`shouldDropOperatorKeyword`); no compose-rules fix renames an operator function.
pub fn set_name(ast: &mut Ast, declaration: NodeId, new_name: &str, kotlin: EmbeddedKotlin) -> Option<NodeId> {
    if matches!(ast.element_type(declaration), PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR) {
        panic!("IncorrectOperationException: setName to constructor");
    }
    let identifier = name_identifier(ast, declaration)?;
    match kt_psi_factory::create_name_identifier_if_possible(ast, &quote_if_needed(new_name, kotlin)) {
        Some(new_identifier) => {
            let parent = ast.tree_parent(identifier).expect("a name identifier has a parent");
            ast.replace_child(parent, identifier, new_identifier);
        }
        None => super::delete(ast, identifier),
    }
    Some(declaration)
}

macro_rules! named {
    ($($t:ident),*) => {$(impl $t {
        /// `getName()`.
        pub fn name(self, ast: &Ast) -> Option<String> {
            name(ast, self.0)
        }

        /// `getNameIdentifier()`.
        pub fn name_identifier(self, ast: &Ast) -> Option<NodeId> {
            name_identifier(ast, self.0)
        }

        /// `getNameAsSafeName().asString()`.
        pub fn name_as_safe_name(self, ast: &Ast) -> String {
            name_as_safe_name(ast, self.0)
        }

        /// `setName(name)`; see [`set_name`].
        pub fn set_name(self, ast: &mut Ast, name: &str, kotlin: EmbeddedKotlin) -> Option<NodeId> {
            set_name(ast, self.0, name, kotlin)
        }
    })*};
}

named!(
    KtNamedDeclaration, KtCallableDeclaration, KtFunction, KtNamedFunction, KtClassOrObject, KtClass, KtObjectDeclaration,
    KtEnumEntry, KtTypeAlias, KtProperty, KtParameter, KtDestructuringDeclarationEntry, KtPrimaryConstructor,
    KtFunctionLiteral
);

impl KtObjectDeclaration {
    pub fn is_companion(self, ast: &Ast) -> bool {
        self.has_modifier(ast, COMPANION_KEYWORD)
    }

    /// `isTopLevel()`: directly in the file.
    pub fn is_top_level(self, ast: &Ast) -> bool {
        ast.tree_parent(self.0).is_some_and(|p| KtFile::is(ast, p))
    }
}
