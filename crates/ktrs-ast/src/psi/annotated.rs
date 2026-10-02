//! `KtAnnotated` / `KtModifierListOwner` and the psiUtil helpers over them (`visibilityModifierType`,
//! `isPublic`), `KtModifierList`, `KtAnnotation`, `KtAnnotationEntry`, `KtFileAnnotationList`.

use ktrs_parser::kt_tokens::VISIBILITY_MODIFIERS;
use ktrs_syntax::SyntaxKind::{self, *};

use super::classes::*;
use super::local::is_local;
use crate::arena::{Ast, NodeId};

/// `KtModifierListOwner.getModifierList()` (the `MODIFIER_LIST` child; `KtTypeProjection`, `KtTypeReference` and
/// the declarations all look it up the same way).
pub(crate) fn modifier_list(ast: &Ast, owner: NodeId) -> Option<KtModifierList> {
    KtModifierList::cast(ast, ast.find_child_by_type(owner, MODIFIER_LIST)?)
}

/// ktPsiUtil `collectAnnotationEntriesFromPsi`: entries among the children, and those of `@[...]` annotations.
fn collect_annotation_entries_from_psi(ast: &Ast, container: NodeId) -> Vec<KtAnnotationEntry> {
    let mut entries = Vec::new();
    for child in ast.children(container) {
        match ast.element_type(child) {
            ANNOTATION_ENTRY => entries.push(KtAnnotationEntry(child)),
            ANNOTATION => entries.extend(KtAnnotation(child).entries(ast)),
            _ => {}
        }
    }
    entries
}

/// `KtAnnotated.getAnnotationEntries()`, dispatched to the implementing class.
fn annotation_entries(ast: &Ast, annotated: NodeId) -> Vec<KtAnnotationEntry> {
    match ast.element_type(annotated) {
        FILE => KtFile(annotated).file_annotation_list(ast).map(|l| l.annotation_entries(ast)).unwrap_or_default(),
        ANNOTATED_EXPRESSION | TYPE_CONSTRAINT => collect_annotation_entries_from_psi(ast, annotated),
        _ => modifier_list(ast, annotated).map(|l| l.annotation_entries(ast)).unwrap_or_default(),
    }
}

macro_rules! annotated {
    ($($t:ident),*) => {$(impl $t {
        /// `KtAnnotated.getAnnotationEntries()`.
        pub fn annotation_entries(self, ast: &Ast) -> Vec<KtAnnotationEntry> {
            annotation_entries(ast, self.0)
        }
    })*};
}

annotated!(
    KtAnnotated, KtModifierListOwner, KtDeclaration, KtNamedDeclaration, KtCallableDeclaration, KtDeclarationWithBody,
    KtDeclarationWithInitializer, KtFunction, KtNamedFunction, KtClassOrObject, KtClass, KtObjectDeclaration, KtEnumEntry,
    KtTypeAlias, KtProperty, KtPropertyAccessor, KtParameter, KtDestructuringDeclaration, KtPrimaryConstructor,
    KtFunctionLiteral, KtTypeReference, KtAnnotatedExpression, KtFile
);

macro_rules! modifier_list_owner {
    ($($t:ident),*) => {$(impl $t {
        pub fn modifier_list(self, ast: &Ast) -> Option<KtModifierList> {
            modifier_list(ast, self.0)
        }

        /// `hasModifier(modifier)`.
        pub fn has_modifier(self, ast: &Ast, modifier: SyntaxKind) -> bool {
            modifier_list(ast, self.0).is_some_and(|l| l.has_modifier(ast, modifier))
        }

        /// psiUtil `visibilityModifier()`: `modifierList?.modifierFromTokenSet(VISIBILITY_MODIFIERS)`.
        pub fn visibility_modifier(self, ast: &Ast) -> Option<NodeId> {
            let list = modifier_list(ast, self.0)?;
            VISIBILITY_MODIFIERS.types().find_map(|t| list.modifier(ast, t))
        }

        /// psiUtil `visibilityModifierType()`.
        pub fn visibility_modifier_type(self, ast: &Ast) -> Option<SyntaxKind> {
            self.visibility_modifier(ast).map(|m| ast.element_type(m))
        }

        /// psiUtil `isPublic`: not local, and no visibility modifier or `public`.
        pub fn is_public(self, ast: &Ast) -> bool {
            if KtDeclaration::is(ast, self.0) && is_local(ast, self.0) {
                return false;
            }
            self.visibility_modifier_type(ast).is_none_or(|v| v == PUBLIC_KEYWORD)
        }
    })*};
}

modifier_list_owner!(
    KtModifierListOwner, KtDeclaration, KtNamedDeclaration, KtCallableDeclaration, KtDeclarationWithBody,
    KtDeclarationWithInitializer, KtFunction, KtNamedFunction, KtClassOrObject, KtClass, KtObjectDeclaration, KtEnumEntry,
    KtTypeAlias, KtProperty, KtPropertyAccessor, KtParameter, KtDestructuringDeclaration, KtPrimaryConstructor,
    KtFunctionLiteral, KtTypeReference
);

impl KtModifierList {
    pub fn annotation_entries(self, ast: &Ast) -> Vec<KtAnnotationEntry> {
        collect_annotation_entries_from_psi(ast, self.0)
    }

    pub fn annotations(self, ast: &Ast) -> Vec<KtAnnotation> {
        ast.children(self.0).filter_map(|c| KtAnnotation::cast(ast, c)).collect()
    }

    pub fn has_modifier(self, ast: &Ast, modifier: SyntaxKind) -> bool {
        self.modifier(ast, modifier).is_some()
    }

    /// `getModifier(tokenType)`.
    pub fn modifier(self, ast: &Ast, modifier: SyntaxKind) -> Option<NodeId> {
        ast.find_child_by_type(self.0, modifier)
    }

    /// `getContextParameterList()` (2.2.21: `getContextReceiverList()`, the same node).
    pub fn context_parameter_list(self, ast: &Ast) -> Option<NodeId> {
        ast.find_child_by_type(self.0, CONTEXT_PARAMETER_LIST)
    }
}

impl KtAnnotation {
    pub fn entries(self, ast: &Ast) -> Vec<KtAnnotationEntry> {
        ast.children(self.0).filter_map(|c| KtAnnotationEntry::cast(ast, c)).collect()
    }
}

impl KtFileAnnotationList {
    pub fn annotation_entries(self, ast: &Ast) -> Vec<KtAnnotationEntry> {
        collect_annotation_entries_from_psi(ast, self.0)
    }
}

impl KtAnnotationEntry {
    /// `getCalleeExpression()`: the `CONSTRUCTOR_CALLEE` child (a `KtConstructorCalleeExpression`).
    pub fn callee_expression(self, ast: &Ast) -> Option<NodeId> {
        ast.find_child_by_type(self.0, CONSTRUCTOR_CALLEE)
    }

    pub fn value_argument_list(self, ast: &Ast) -> Option<KtValueArgumentList> {
        KtValueArgumentList::cast(ast, ast.find_child_by_type(self.0, VALUE_ARGUMENT_LIST)?)
    }

    /// `KtCallElement.getValueArguments()`: an annotation entry has no lambda arguments.
    pub fn value_arguments(self, ast: &Ast) -> Vec<KtValueArgument> {
        self.value_argument_list(ast).map(|l| l.arguments(ast)).unwrap_or_default()
    }

    /// `getTypeReference()`: the callee's type reference.
    pub fn type_reference(self, ast: &Ast) -> Option<KtTypeReference> {
        KtTypeReference::cast(ast, ast.find_child_by_type(self.callee_expression(ast)?, TYPE_REFERENCE)?)
    }
}

impl KtFile {
    pub fn file_annotation_list(self, ast: &Ast) -> Option<KtFileAnnotationList> {
        KtFileAnnotationList::cast(ast, ast.find_child_by_type(self.0, FILE_ANNOTATION_LIST)?)
    }
}
