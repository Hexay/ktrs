//! Member and container accessors of declarations: `getDeclarations`, `getSecondaryConstructors`,
//! `getSuperTypeListEntries`, `getTypeParameters`, `isInterface`, `isObjectLiteral`, `isTopLevel`, `isLocal`,
//! `containingClassOrObject`, `getOwnerFunction`, and `KtFile`'s directive lookups.

use ktrs_syntax::SyntaxKind::*;

use super::file::{FqName, KtFile};
use crate::element::PsiElement;
use crate::types::*;

/// Kotlin `isKtFile(parent)`.
fn parent_is_file(e: &PsiElement) -> bool {
    e.parent().is_some_and(|p| p.is_file())
}

macro_rules! class_or_object {
    ($($t:ident),*) => {$(impl $t {
        /// `getDeclarations()`: the class body's.
        pub fn declarations(&self) -> Vec<KtDeclaration> {
            self.body().map(|b| b.declarations()).unwrap_or_default()
        }

        /// `getSecondaryConstructors()`.
        pub fn secondary_constructors(&self) -> Vec<KtSecondaryConstructor> {
            self.body().map(|b| b.secondary_constructors()).unwrap_or_default()
        }

        /// `getSuperTypeListEntries()`; a `KtEnumEntry`'s are its initializers.
        pub fn super_type_list_entries(&self) -> Vec<KtSuperTypeListEntry> {
            if self.kind() == ENUM_ENTRY {
                let initializer_list = self.get_stub_or_psi_child::<KtInitializerList>(INITIALIZER_LIST);
                return initializer_list.map(|l| l.initializers()).unwrap_or_default();
            }
            self.super_type_list().map(|l| l.entries()).unwrap_or_default()
        }

        /// `isTopLevel()`.
        pub fn is_top_level(&self) -> bool {
            parent_is_file(self)
        }

        /// psiUtil `KtClassOrObject.isObjectLiteral()`.
        pub fn is_object_literal(&self) -> bool {
            self.kind() == OBJECT_DECLARATION && self.parent().is_some_and(|p| p.is::<KtObjectLiteralExpression>())
        }
    })*};
}

class_or_object!(KtClassOrObject, KtClass, KtObjectDeclaration, KtEnumEntry);

macro_rules! class {
    ($($t:ident),*) => {$(impl $t {
        /// `isInterface()`.
        pub fn is_interface(&self) -> bool {
            self.find_child_by_type::<PsiElement>(INTERFACE_KEYWORD).is_some()
        }
    })*};
}

class!(KtClass, KtEnumEntry);

macro_rules! type_parameter_list_owner {
    ($($t:ident),*) => {$(impl $t {
        /// `KtTypeParameterListOwner.getTypeParameters()`.
        pub fn type_parameters(&self) -> Vec<KtTypeParameter> {
            self.type_parameter_list().map(|l| l.parameters()).unwrap_or_default()
        }
    })*};
}

type_parameter_list_owner!(
    KtTypeParameterListOwner, KtCallableDeclaration, KtFunction, KtClassOrObject, KtConstructor, KtClass,
    KtObjectDeclaration, KtEnumEntry, KtNamedFunction, KtProperty, KtTypeAlias, KtDestructuringDeclarationEntry,
    KtPrimaryConstructor, KtSecondaryConstructor, KtParameter, KtFunctionLiteral
);

impl KtClassBody {
    /// `getDeclarations()`: `PsiTreeUtil.getChildrenOfTypeAsList(this, KtDeclaration)`.
    pub fn declarations(&self) -> Vec<KtDeclaration> {
        self.get_children_of_type()
    }

    /// `secondaryConstructors`.
    pub fn secondary_constructors(&self) -> Vec<KtSecondaryConstructor> {
        self.get_stub_or_psi_children(SECONDARY_CONSTRUCTOR)
    }

    /// psiUtil `KtClassBody.containingClassOrObject`.
    pub fn containing_class_or_object(&self) -> Option<KtClassOrObject> {
        let parent = self.parent()?;
        match parent.kind() {
            COMPANION_BLOCK => parent.parent()?.cast::<KtClassBody>()?.containing_class_or_object(),
            _ => parent.cast(),
        }
    }
}

impl KtFile {
    /// `getDeclarations()`.
    pub fn declarations(&self) -> Vec<KtDeclaration> {
        self.get_children_of_type()
    }

    /// `getPackageDirective()`.
    pub fn package_directive(&self) -> Option<KtPackageDirective> {
        self.find_child_by_type(PACKAGE_DIRECTIVE)
    }

    /// `getPackageFqName()`.
    pub fn package_fq_name(&self) -> FqName {
        self.package_directive().map_or_else(FqName::root, |d| d.fq_name())
    }
}

impl KtNamedFunction {
    /// `isTopLevel`.
    pub fn is_top_level(&self) -> bool {
        parent_is_file(self)
    }

    /// `hasBody()`.
    pub fn has_body(&self) -> bool {
        self.body_expression().is_some()
    }

    /// `isLocal()`: not directly in a file, class body or script block.
    pub fn is_local(&self) -> bool {
        let Some(parent) = self.parent() else { return false };
        if parent.is_file() || parent.is::<KtClassBody>() {
            return false;
        }
        !parent.parent().is_some_and(|g| g.is::<KtScript>())
    }
}

impl KtProperty {
    /// `isTopLevel()`.
    pub fn is_top_level(&self) -> bool {
        parent_is_file(self)
    }

    /// `isMember()`: psiUtil `containingClassOrScript != null`.
    pub fn is_member(&self) -> bool {
        let Some(parent) = self.parent() else { return false };
        parent.is::<KtClassOrObject>()
            || parent.is::<KtClassBody>()
            || (parent.is::<KtBlockExpression>() && parent.parent().is_some_and(|g| g.is::<KtScript>()))
    }

    /// `isLocal()`.
    pub fn is_local(&self) -> bool {
        !self.is_top_level() && !self.is_member()
    }
}

macro_rules! declaration {
    ($($t:ident),*) => {$(impl $t {
        /// psiUtil `KtDeclaration.containingClassOrObject`.
        pub fn containing_class_or_object(&self) -> Option<KtClassOrObject> {
            let parent = self.parent()?;
            match parent.kind() {
                _ if parent.is_file() => None,
                CLASS_BODY => parent.upcast::<KtClassBody>().containing_class_or_object(),
                CLASS | ENUM_ENTRY | OBJECT_DECLARATION => parent.cast(),
                VALUE_PARAMETER_LIST => parent.parent()?.cast::<KtPrimaryConstructor>()?.parent()?.cast(),
                DESTRUCTURING_DECLARATION if self.kind() == DESTRUCTURING_DECLARATION_ENTRY => {
                    parent.upcast::<KtDeclaration>().containing_class_or_object()
                }
                _ => None,
            }
        }

        /// psiUtil `KtElement.containingClass()`.
        pub fn containing_class(&self) -> Option<KtClass> {
            self.get_parent_of_type(true)
        }
    })*};
}

declaration!(
    KtDeclaration, KtNamedDeclaration, KtCallableDeclaration, KtFunction, KtClassOrObject, KtConstructor, KtClass,
    KtObjectDeclaration, KtNamedFunction, KtProperty, KtPrimaryConstructor, KtSecondaryConstructor, KtParameter
);

impl KtParameterList {
    /// `getOwnerFunction()`.
    pub fn owner_function(&self) -> Option<KtDeclarationWithBody> {
        self.parent()?.cast()
    }
}

impl KtParameter {
    /// `getOwnerFunction()`.
    pub fn owner_function(&self) -> Option<KtDeclarationWithBody> {
        self.parent()?.cast::<KtParameterList>()?.owner_function()
    }
}
