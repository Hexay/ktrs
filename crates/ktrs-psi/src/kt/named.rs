//! `KtNamedDeclaration.getName()` / `getNameAsSafeName()` per implementing class (`KtNamedDeclarationStub`,
//! `KtNamedDeclarationNotStubbed`, `KtConstructor`, `KtFunctionLiteral`, `KtObjectDeclaration`), the `Name` value
//! they return, and `getTextOffset()`.

use std::fmt;

use ktrs_syntax::SyntaxKind::*;

use super::calls::unquote_identifier;
use crate::classes::is_named_declaration;
use crate::element::PsiElement;
use crate::types::*;

/// `org.jetbrains.kotlin.name.Name`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Name {
    name: String,
    special: bool,
}

impl Name {
    /// `SpecialNames.NO_NAME_PROVIDED`.
    pub const NO_NAME_PROVIDED: &'static str = "<no name provided>";

    pub fn identifier(name: &str) -> Name {
        Name { name: name.to_owned(), special: false }
    }

    pub fn special(name: &str) -> Name {
        Name { name: name.to_owned(), special: true }
    }

    pub fn as_string(&self) -> &str {
        &self.name
    }

    pub fn is_special(&self) -> bool {
        self.special
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

/// `getName()`, dispatched to the implementing class.
fn name(e: &PsiElement) -> Option<String> {
    match e.kind() {
        // SpecialNames.ANONYMOUS_STRING
        FUNCTION_LITERAL => Some("<anonymous>".to_owned()),
        // KtConstructor: `getContainingClassOrObject().name`
        PRIMARY_CONSTRUCTOR => name(&e.parent()?),
        SECONDARY_CONSTRUCTOR => name(&e.parent()?.parent()?),
        OBJECT_DECLARATION => identifier_name(e).or_else(|| {
            let object = e.upcast::<KtObjectDeclaration>();
            (object.is_companion() && !object.is_top_level()).then(|| "Companion".to_owned())
        }),
        // TODO: KtScript.getName() is the script class name derived from the file name
        SCRIPT => None,
        kind if !e.is_leaf() && !e.is_file() && is_named_declaration(kind) => identifier_name(e),
        _ => None,
    }
}

fn identifier_name(e: &PsiElement) -> Option<String> {
    Some(unquote_identifier(e.find_child_by_type::<PsiElement>(IDENTIFIER)?.text_slice()))
}

/// `KtPsiUtil.safeName(getName())`.
fn name_as_safe_name(e: &PsiElement) -> Name {
    match name(e) {
        Some(name) => Name::identifier(&name),
        None => Name::special(Name::NO_NAME_PROVIDED),
    }
}

macro_rules! named {
    ($($t:ident),*) => {$(impl $t {
        /// `getName()`.
        pub fn name(&self) -> Option<String> {
            name(self)
        }

        /// `getNameAsSafeName()`.
        pub fn name_as_safe_name(&self) -> Name {
            name_as_safe_name(self)
        }
    })*};
}

named!(
    KtNamedDeclaration, KtCallableDeclaration, KtFunction, KtClassOrObject, KtTypeParameterListOwner, KtConstructor, KtClass,
    KtObjectDeclaration, KtEnumEntry, KtNamedFunction, KtProperty, KtTypeAlias, KtDestructuringDeclarationEntry,
    KtPrimaryConstructor, KtSecondaryConstructor, KtParameter, KtTypeParameter, KtFunctionLiteral
);

impl PsiElement {
    /// `PsiElement.getTextOffset()`: the name identifier of named declarations (the `object`, `constructor`,
    /// `get`/`set`/`field` keyword of the unnamed ones), else the start offset.
    pub fn text_offset(&self) -> usize {
        if self.is_leaf() || self.is_file() {
            return self.start_offset();
        }
        let start_of = |e: Option<PsiElement>| e.map(|e| e.start_offset());
        let offset = match self.kind() {
            OBJECT_DECLARATION => start_of(self.find_child_by_type(IDENTIFIER)).or_else(|| start_of(self.find_child_by_type(OBJECT_KEYWORD))),
            PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR => {
                start_of(self.find_child_by_type(CONSTRUCTOR_KEYWORD)).or_else(|| start_of(self.find_child_by_type(VALUE_PARAMETER_LIST)))
            }
            PROPERTY_ACCESSOR => start_of(self.find_child_by_type(GET_KEYWORD)).or_else(|| start_of(self.find_child_by_type(SET_KEYWORD))),
            BACKING_FIELD => start_of(self.find_child_by_type(FIELD_KEYWORD)),
            IMPORT_ALIAS => start_of(self.find_child_by_type(IDENTIFIER)),
            FUNCTION_LITERAL | SCRIPT => None,
            kind if is_named_declaration(kind) => start_of(self.find_child_by_type(IDENTIFIER)),
            _ => None,
        };
        offset.unwrap_or_else(|| self.start_offset())
    }
}
