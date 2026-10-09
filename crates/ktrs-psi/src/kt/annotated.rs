//! `KtAnnotated.getAnnotationEntries()` per implementing class, and the `KtModifierListOwner` helpers of
//! `KtModifierListOwnerStub` / psiUtils.kt / ktPsiUtil.kt (`hasModifier`, `visibilityModifier[Type]`, `isPrivate`,
//! `isProtected`, `hasExpectModifier`, `hasActualModifier`).

use ktrs_parser::kt_tokens::VISIBILITY_MODIFIERS;
use ktrs_syntax::SyntaxKind::{self, *};

use super::file::KtFile;
use super::operators::collect_annotation_entries_from_psi;
use crate::element::PsiElement;
use crate::types::*;

fn annotation_entries(annotated: &PsiElement) -> Vec<KtAnnotationEntry> {
    if annotated.is_file() {
        // KtCommonFile: `fileAnnotationList?.annotationEntries ?: emptyList()`
        let list = annotated.find_child_by_type::<KtFileAnnotationList>(FILE_ANNOTATION_LIST);
        return list.map(|l| l.annotation_entries()).unwrap_or_default();
    }
    match annotated.kind() {
        ANNOTATED_EXPRESSION | TYPE_CONSTRAINT => collect_annotation_entries_from_psi(annotated),
        _ => modifier_list(annotated).map(|l| l.annotation_entries()).unwrap_or_default(),
    }
}

fn modifier_list(owner: &PsiElement) -> Option<KtModifierList> {
    owner.find_child_by_type(MODIFIER_LIST)
}

fn has_modifier(owner: &PsiElement, modifier: SyntaxKind) -> bool {
    modifier_list(owner).is_some_and(|l| l.has_modifier(modifier))
}

/// ktPsiUtil `modifierFromTokenSet(VISIBILITY_MODIFIERS)`: the first set member (in set order) the list has.
fn visibility_modifier(owner: &PsiElement) -> Option<PsiElement> {
    let list = modifier_list(owner)?;
    VISIBILITY_MODIFIERS.types().find_map(|t| list.modifier(t))
}

macro_rules! annotated {
    ($($t:ident),*) => {$(impl $t {
        /// `KtAnnotated.getAnnotationEntries()`.
        pub fn annotation_entries(&self) -> Vec<KtAnnotationEntry> {
            annotation_entries(self)
        }
    })*};
}

annotated!(
    KtAnnotated, KtModifierListOwner, KtDeclaration, KtNamedDeclaration, KtCallableDeclaration, KtDeclarationWithBody,
    KtFunction, KtClassOrObject, KtTypeParameterListOwner, KtConstructor, KtClass, KtObjectDeclaration, KtEnumEntry,
    KtNamedFunction, KtProperty, KtPropertyAccessor, KtTypeAlias, KtDestructuringDeclaration,
    KtDestructuringDeclarationEntry, KtPrimaryConstructor, KtSecondaryConstructor, KtParameter, KtTypeParameter,
    KtFunctionLiteral, KtTypeReference, KtPackageDirective, KtTypeProjection, KtFile
);

macro_rules! modifier_list_owner {
    ($($t:ident),*) => {$(impl $t {
        /// `KtModifierListOwner.hasModifier(modifier)`.
        pub fn has_modifier(&self, modifier: SyntaxKind) -> bool {
            has_modifier(self, modifier)
        }

        /// psiUtil `visibilityModifier()`.
        pub fn visibility_modifier(&self) -> Option<PsiElement> {
            visibility_modifier(self)
        }

        /// psiUtil `visibilityModifierType()`.
        pub fn visibility_modifier_type(&self) -> Option<SyntaxKind> {
            visibility_modifier(self).map(|m| m.kind())
        }

        /// psiUtil `isPrivate()`.
        pub fn is_private(&self) -> bool {
            has_modifier(self, PRIVATE_KEYWORD)
        }

        /// psiUtil `isProtected()`.
        pub fn is_protected(&self) -> bool {
            has_modifier(self, PROTECTED_KEYWORD)
        }

        /// psiUtil `hasExpectModifier()`.
        pub fn has_expect_modifier(&self) -> bool {
            has_modifier(self, EXPECT_KEYWORD)
        }

        /// psiUtil `hasActualModifier()`.
        pub fn has_actual_modifier(&self) -> bool {
            has_modifier(self, ACTUAL_KEYWORD)
        }
    })*};
}

modifier_list_owner!(
    KtModifierListOwner, KtDeclaration, KtNamedDeclaration, KtCallableDeclaration, KtDeclarationWithBody, KtFunction,
    KtClassOrObject, KtTypeParameterListOwner, KtConstructor, KtClass, KtObjectDeclaration, KtEnumEntry, KtNamedFunction,
    KtProperty, KtPropertyAccessor, KtTypeAlias, KtDestructuringDeclaration, KtDestructuringDeclarationEntry,
    KtPrimaryConstructor, KtSecondaryConstructor, KtParameter, KtTypeParameter, KtFunctionLiteral, KtTypeReference,
    KtPackageDirective, KtTypeProjection
);

impl KtFile {
    /// `getFileAnnotationList()`.
    pub fn file_annotation_list(&self) -> Option<KtFileAnnotationList> {
        self.find_child_by_type(FILE_ANNOTATION_LIST)
    }
}
