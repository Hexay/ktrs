//! Methods of the declaration interfaces (`KtModifierListOwner`, `PsiNameIdentifierOwner`,
//! `KtTypeParameterListOwner`, `KtCallableDeclaration`, `KtDeclarationWithBody`), each resolving the
//! upstream override by element type.

use ktrs_syntax::SyntaxKind::*;

use crate::element::PsiElement;
use crate::tree_util::get_next_sibling_of_type;
use crate::types::*;

macro_rules! modifier_list_owner {
    ($($t:ident),*) => {$(impl $t {
        /// `KtModifierListOwner.getModifierList()`.
        pub fn modifier_list(&self) -> Option<KtModifierList> {
            self.get_stub_or_psi_child(MODIFIER_LIST)
        }
    })*};
}

modifier_list_owner!(
    KtModifierListOwner, KtDeclaration, KtNamedDeclaration, KtCallableDeclaration, KtDeclarationWithBody, KtFunction,
    KtClassOrObject, KtTypeParameterListOwner, KtValVarKeywordOwner, KtDeclarationWithInitializer, KtConstructor,
    KtAnonymousInitializer, KtClass, KtObjectDeclaration, KtEnumEntry, KtNamedFunction, KtProperty, KtPropertyAccessor,
    KtBackingField, KtTypeAlias, KtDestructuringDeclaration, KtDestructuringDeclarationEntry, KtClassInitializer,
    KtScriptInitializer, KtPrimaryConstructor, KtSecondaryConstructor, KtParameter, KtTypeParameter, KtScript,
    KtFunctionLiteral, KtTypeReference, KtPackageDirective, KtTypeProjection, KtNullableType
);

macro_rules! name_identifier_owner {
    ($($t:ident),*) => {$(impl $t {
        /// `PsiNameIdentifierOwner.getNameIdentifier()`.
        pub fn name_identifier(&self) -> Option<PsiElement> {
            name_identifier(self)
        }
    })*};
}

name_identifier_owner!(
    KtNamedDeclaration, KtCallableDeclaration, KtFunction, KtClassOrObject, KtTypeParameterListOwner, KtConstructor,
    KtClass, KtObjectDeclaration, KtEnumEntry, KtNamedFunction, KtProperty, KtTypeAlias, KtDestructuringDeclarationEntry,
    KtPrimaryConstructor, KtSecondaryConstructor, KtParameter, KtTypeParameter, KtScript, KtFunctionLiteral,
    KtImportAlias, KtLabeledExpression
);

fn name_identifier(e: &PsiElement) -> Option<PsiElement> {
    match e.kind() {
        FUNCTION_LITERAL | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR => None,
        LABELED_EXPRESSION => e.upcast::<KtLabeledExpression>().target_label()?.identifier(),
        // KtNamedDeclarationStub / KtNamedDeclarationNotStubbed / KtImportAlias
        _ => e.find_child_by_type(IDENTIFIER),
    }
}

macro_rules! type_parameter_list_owner {
    ($($t:ident),*) => {$(impl $t {
        /// `KtTypeParameterListOwner.getTypeParameterList()`.
        pub fn type_parameter_list(&self) -> Option<KtTypeParameterList> {
            match self.kind() {
                VALUE_PARAMETER | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR | DESTRUCTURING_DECLARATION_ENTRY => None,
                _ => self.get_stub_or_psi_child(TYPE_PARAMETER_LIST),
            }
        }

        /// `KtTypeParameterListOwner.getTypeConstraintList()`.
        pub fn type_constraint_list(&self) -> Option<KtTypeConstraintList> {
            match self.kind() {
                VALUE_PARAMETER | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR | DESTRUCTURING_DECLARATION_ENTRY => None,
                _ => self.get_stub_or_psi_child(TYPE_CONSTRAINT_LIST),
            }
        }
    })*};
}

type_parameter_list_owner!(
    KtTypeParameterListOwner, KtCallableDeclaration, KtFunction, KtClassOrObject, KtConstructor, KtClass,
    KtObjectDeclaration, KtEnumEntry, KtNamedFunction, KtProperty, KtTypeAlias, KtDestructuringDeclarationEntry,
    KtPrimaryConstructor, KtSecondaryConstructor, KtParameter, KtFunctionLiteral
);

macro_rules! callable_declaration {
    ($($t:ident),*) => {$(impl $t {
        /// `KtCallableDeclaration.getReceiverTypeReference()`.
        pub fn receiver_type_reference(&self) -> Option<KtTypeReference> {
            receiver_type_reference(self)
        }

        /// `KtCallableDeclaration.getTypeReference()`.
        pub fn type_reference(&self) -> Option<KtTypeReference> {
            callable_type_reference(self)
        }

        /// `KtCallableDeclaration.getValueParameterList()`.
        pub fn value_parameter_list(&self) -> Option<KtParameterList> {
            match self.kind() {
                PROPERTY | VALUE_PARAMETER | DESTRUCTURING_DECLARATION_ENTRY => None,
                _ => self.get_stub_or_psi_child(VALUE_PARAMETER_LIST),
            }
        }
    })*};
}

callable_declaration!(
    KtCallableDeclaration, KtFunction, KtConstructor, KtNamedFunction, KtProperty, KtDestructuringDeclarationEntry,
    KtPrimaryConstructor, KtSecondaryConstructor, KtParameter, KtFunctionLiteral
);

fn receiver_type_reference(e: &PsiElement) -> Option<KtTypeReference> {
    match e.kind() {
        // KtNamedFunction.receiverTypeRefByTree
        FUN => {
            for child in e.all_children() {
                if matches!(child.kind(), LPAR | COLON) {
                    break;
                }
                if let Some(type_reference) = child.cast::<KtTypeReference>() {
                    return Some(type_reference);
                }
            }
            None
        }
        // KtProperty.getReceiverTypeRefByTree
        PROPERTY => {
            for child in e.all_children() {
                if child.kind() == COLON {
                    break;
                }
                if child.kind() == TYPE_REFERENCE {
                    return Some(child.upcast());
                }
            }
            None
        }
        _ => None,
    }
}

fn callable_type_reference(e: &PsiElement) -> Option<KtTypeReference> {
    match e.kind() {
        FUN | PROPERTY | DESTRUCTURING_DECLARATION_ENTRY => type_ref_helpers_get_type_reference(e),
        VALUE_PARAMETER => e.get_stub_or_psi_child(TYPE_REFERENCE),
        _ => None,
    }
}

/// TypeRefHelpers.kt `getTypeReference(declaration)`: the first type reference after the first `:`.
fn type_ref_helpers_get_type_reference(declaration: &PsiElement) -> Option<KtTypeReference> {
    declaration.first_child()?.siblings(true, true).skip_while(|c| c.kind() != COLON).find_map(|c| c.cast())
}

macro_rules! value_parameters_owner {
    ($($t:ident),*) => {$(impl $t {
        /// `KtCallableDeclaration.getValueParameters()` / `KtDeclarationWithBody.getValueParameters()`.
        pub fn value_parameters(&self) -> Vec<KtParameter> {
            value_parameters(self)
        }
    })*};
}

value_parameters_owner!(
    KtCallableDeclaration, KtDeclarationWithBody, KtFunction, KtConstructor, KtNamedFunction, KtProperty,
    KtPropertyAccessor, KtDestructuringDeclarationEntry, KtPrimaryConstructor, KtSecondaryConstructor, KtParameter,
    KtFunctionLiteral
);

fn value_parameters(e: &PsiElement) -> Vec<KtParameter> {
    match e.kind() {
        FUN | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR | FUNCTION_LITERAL => e
            .find_child_by_type::<KtParameterList>(VALUE_PARAMETER_LIST)
            .map(|list| list.parameters())
            .unwrap_or_default(),
        PROPERTY_ACCESSOR => e.upcast::<KtPropertyAccessor>().parameter().into_iter().collect(),
        _ => Vec::new(),
    }
}

macro_rules! declaration_with_body {
    ($($t:ident),*) => {$(impl $t {
        /// `KtDeclarationWithBody.getBodyExpression()`.
        pub fn body_expression(&self) -> Option<KtExpression> {
            body_expression(self)
        }

        /// `KtDeclarationWithBody.getBodyBlockExpression()`.
        pub fn body_block_expression(&self) -> Option<KtBlockExpression> {
            match self.kind() {
                // KtNamedFunction / KtPropertyAccessor overrides; same result as the interface default
                FUN | PROPERTY_ACCESSOR => self.find_child_by_class::<KtExpression>()?.cast(),
                _ => self.body_expression()?.cast(),
            }
        }
    })*};
}

declaration_with_body!(
    KtDeclarationWithBody, KtFunction, KtConstructor, KtNamedFunction, KtPropertyAccessor, KtPrimaryConstructor,
    KtSecondaryConstructor, KtFunctionLiteral
);

fn body_expression(e: &PsiElement) -> Option<KtExpression> {
    match e.kind() {
        PRIMARY_CONSTRUCTOR => None,
        SECONDARY_CONSTRUCTOR => e.find_child_by_class::<KtBlockExpression>().map(|b| b.upcast()),
        // KtNamedFunction, KtPropertyAccessor, KtFunctionNotStubbed
        _ => e.find_child_by_class(),
    }
}

/// `PsiTreeUtil.getNextSiblingOfType(findChildByType(EQ), KtExpression)`: the `getInitializer()` shape
/// shared by properties, accessors, backing fields and destructuring declarations.
pub(super) fn expression_after_eq(e: &PsiElement) -> Option<KtExpression> {
    get_next_sibling_of_type(e.find_child_by_type::<PsiElement>(EQ).as_ref())
}
