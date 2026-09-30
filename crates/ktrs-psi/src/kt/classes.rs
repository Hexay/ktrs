//! Class-like declarations, constructors, super type lists, initializers, type aliases, type parameters.

use ktrs_syntax::SyntaxKind::{self, *};

use crate::element::PsiElement;
use crate::tokens::{CLASS_INTERFACE_OBJECT, SUPER_TYPE_LIST_ENTRIES};
use crate::tree_util::get_trailing_comma_by_closing_element;
use crate::types::*;

/// `KtModifierListOwner.hasModifier(token)` on the AST path.
fn has_modifier(owner: &PsiElement, token: SyntaxKind) -> bool {
    owner
        .find_child_by_type::<PsiElement>(MODIFIER_LIST)
        .is_some_and(|list| list.find_child_by_type::<PsiElement>(token).is_some())
}

macro_rules! class_or_object {
    ($($t:ident),*) => {$(impl $t {
        pub fn colon(&self) -> Option<PsiElement> {
            self.find_child_by_type(COLON)
        }

        pub fn super_type_list(&self) -> Option<KtSuperTypeList> {
            self.get_stub_or_psi_child(SUPER_TYPE_LIST)
        }

        pub fn body(&self) -> Option<KtClassBody> {
            self.get_stub_or_psi_child(CLASS_BODY)
        }

        pub fn primary_constructor(&self) -> Option<KtPrimaryConstructor> {
            self.get_stub_or_psi_child(PRIMARY_CONSTRUCTOR)
        }

        /// `class`, `interface` or `object` keyword.
        pub fn declaration_keyword(&self) -> Option<PsiElement> {
            self.find_child_by_type_set(CLASS_INTERFACE_OBJECT)
        }
    })*};
}

class_or_object!(KtClassOrObject, KtClass, KtObjectDeclaration, KtEnumEntry);

macro_rules! class {
    ($($t:ident),*) => {$(impl $t {
        pub fn is_enum(&self) -> bool {
            has_modifier(self, ENUM_KEYWORD)
        }
    })*};
}

class!(KtClass, KtEnumEntry);

impl KtObjectDeclaration {
    pub fn is_companion(&self) -> bool {
        has_modifier(self, COMPANION_KEYWORD)
    }

    pub fn object_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type(OBJECT_KEYWORD)
    }
}

impl KtEnumEntry {
    pub fn initializer_list(&self) -> Option<KtInitializerList> {
        self.get_stub_or_psi_child(INITIALIZER_LIST)
    }
}

impl KtInitializerList {
    pub fn initializers(&self) -> Vec<KtSuperTypeListEntry> {
        self.get_stub_or_psi_children_set(SUPER_TYPE_LIST_ENTRIES)
    }
}

impl KtClassBody {
    pub fn enum_entries(&self) -> Vec<KtEnumEntry> {
        self.get_stub_or_psi_children(ENUM_ENTRY)
    }
}

impl KtSuperTypeList {
    pub fn entries(&self) -> Vec<KtSuperTypeListEntry> {
        self.get_stub_or_psi_children_set(SUPER_TYPE_LIST_ENTRIES)
    }
}

macro_rules! super_type_list_entry {
    ($($t:ident),*) => {$(impl $t {
        /// `KtSuperTypeListEntry.getTypeReference()` (`KtSuperTypeCallEntry` reads it from its callee).
        pub fn type_reference(&self) -> Option<KtTypeReference> {
            if self.kind() == SUPER_TYPE_CALL_ENTRY {
                return self.get_stub_or_psi_child::<KtConstructorCalleeExpression>(CONSTRUCTOR_CALLEE)?.type_reference();
            }
            self.get_stub_or_psi_child(TYPE_REFERENCE)
        }

        pub fn type_as_user_type(&self) -> Option<KtUserType> {
            self.type_reference()?.type_element()?.cast()
        }
    })*};
}

super_type_list_entry!(KtSuperTypeListEntry, KtDelegatedSuperTypeEntry, KtSuperTypeCallEntry, KtSuperTypeEntry);

impl KtDelegatedSuperTypeEntry {
    pub fn delegate_expression(&self) -> Option<KtExpression> {
        self.find_child_by_class()
    }
}

impl KtSuperTypeCallEntry {
    /// `getCalleeExpression()`: upstream requires it (throws when absent; None here).
    pub fn callee_expression(&self) -> Option<KtConstructorCalleeExpression> {
        self.get_stub_or_psi_child(CONSTRUCTOR_CALLEE)
    }

    pub fn value_argument_list(&self) -> Option<KtValueArgumentList> {
        self.get_stub_or_psi_child(VALUE_ARGUMENT_LIST)
    }

    pub fn lambda_arguments(&self) -> Vec<KtLambdaArgument> {
        Vec::new()
    }

    pub fn type_argument_list(&self) -> Option<KtTypeArgumentList> {
        self.type_as_user_type()?.type_argument_list()
    }
}

impl KtConstructorCalleeExpression {
    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }

    pub fn constructor_reference_expression(&self) -> Option<KtSimpleNameExpression> {
        self.type_reference()?.type_element()?.cast::<KtUserType>()?.reference_expression()
    }
}

macro_rules! constructor {
    ($($t:ident),*) => {$(impl $t {
        pub fn constructor_keyword(&self) -> Option<PsiElement> {
            self.find_child_by_type(CONSTRUCTOR_KEYWORD)
        }

        pub fn has_constructor_keyword(&self) -> bool {
            self.constructor_keyword().is_some()
        }
    })*};
}

constructor!(KtConstructor, KtPrimaryConstructor, KtSecondaryConstructor);

impl KtSecondaryConstructor {
    /// `getDelegationCall()`: upstream requires it (throws when absent; None here).
    pub fn delegation_call(&self) -> Option<KtConstructorDelegationCall> {
        self.find_child_by_class()
    }
}

impl KtConstructorDelegationCall {
    pub fn callee_expression(&self) -> Option<KtConstructorDelegationReferenceExpression> {
        self.find_child_by_class()
    }

    pub fn value_argument_list(&self) -> Option<KtValueArgumentList> {
        self.find_child_by_type(VALUE_ARGUMENT_LIST)
    }

    pub fn lambda_arguments(&self) -> Vec<KtLambdaArgument> {
        Vec::new()
    }

    pub fn type_argument_list(&self) -> Option<KtTypeArgumentList> {
        None
    }

    pub fn is_implicit(&self) -> bool {
        self.callee_expression().is_some_and(|callee| callee.first_child().is_none())
    }

    pub fn is_call_to_this(&self) -> bool {
        self.callee_expression().is_some_and(|callee| callee.is_this())
    }
}

impl KtConstructorDelegationReferenceExpression {
    pub fn is_this(&self) -> bool {
        self.find_child_by_type::<PsiElement>(THIS_KEYWORD).is_some()
    }
}

macro_rules! anonymous_initializer {
    ($($t:ident),*) => {$(impl $t {
        pub fn body(&self) -> Option<KtExpression> {
            self.find_child_by_class()
        }
    })*};
}

anonymous_initializer!(KtAnonymousInitializer, KtClassInitializer, KtScriptInitializer);

impl KtTypeAlias {
    pub fn type_alias_keyword(&self) -> Option<PsiElement> {
        self.find_child_by_type(TYPE_ALIAS_KEYWORD)
    }

    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }
}

impl KtTypeParameterList {
    pub fn parameters(&self) -> Vec<KtTypeParameter> {
        self.get_stub_or_psi_children(TYPE_PARAMETER)
    }

    pub fn trailing_comma(&self) -> Option<PsiElement> {
        get_trailing_comma_by_closing_element(self.find_child_by_type::<PsiElement>(GT).as_ref())
    }
}

impl KtTypeParameter {
    pub fn extends_bound(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }
}

impl KtTypeConstraintList {
    pub fn constraints(&self) -> Vec<KtTypeConstraint> {
        self.get_stub_or_psi_children(TYPE_CONSTRAINT)
    }
}

impl KtTypeConstraint {
    pub fn subject_type_parameter_name(&self) -> Option<KtSimpleNameExpression> {
        self.get_stub_or_psi_child(REFERENCE_EXPRESSION)
    }

    pub fn bound_type_reference(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }
}
