//! Modifier lists, annotations, use-site targets and context parameter lists.

use ktrs_syntax::SyntaxKind::{self, *};

use super::operators::collect_annotation_entries_from_psi;
use crate::element::PsiElement;
use crate::types::*;

macro_rules! modifier_list {
    ($($t:ident),*) => {$(impl $t {
        pub fn annotations(&self) -> Vec<KtAnnotation> {
            self.get_stub_or_psi_children(ANNOTATION)
        }

        pub fn annotation_entries(&self) -> Vec<KtAnnotationEntry> {
            collect_annotation_entries_from_psi(self)
        }

        pub fn context_parameter_list(&self) -> Option<KtContextParameterList> {
            self.get_stub_or_psi_child(CONTEXT_PARAMETER_LIST)
        }

        pub fn context_receiver_list(&self) -> Option<KtContextReceiverList> {
            self.get_stub_or_psi_child(CONTEXT_PARAMETER_LIST)
        }

        pub fn has_modifier(&self, token: SyntaxKind) -> bool {
            self.modifier(token).is_some()
        }

        pub fn modifier(&self, token: SyntaxKind) -> Option<PsiElement> {
            self.find_child_by_type(token)
        }
    })*};
}

modifier_list!(KtModifierList, KtDeclarationModifierList);

impl KtFileAnnotationList {
    pub fn annotations(&self) -> Vec<KtAnnotation> {
        self.get_stub_or_psi_children(ANNOTATION)
    }

    pub fn annotation_entries(&self) -> Vec<KtAnnotationEntry> {
        collect_annotation_entries_from_psi(self)
    }
}

impl KtAnnotation {
    pub fn entries(&self) -> Vec<KtAnnotationEntry> {
        self.get_stub_or_psi_children(ANNOTATION_ENTRY)
    }

    pub fn use_site_target(&self) -> Option<KtAnnotationUseSiteTarget> {
        self.get_stub_or_psi_child(ANNOTATION_TARGET)
    }
}

impl KtAnnotationEntry {
    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.callee_expression()?.type_reference()
    }

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
        self.type_reference()?.type_element()?.cast::<KtUserType>()?.type_argument_list()
    }

    pub fn at_symbol(&self) -> Option<PsiElement> {
        self.find_child_by_type(AT)
    }

    /// Own target, else the enclosing `@target:[...]` annotation's.
    pub fn use_site_target(&self) -> Option<KtAnnotationUseSiteTarget> {
        self.get_stub_or_psi_child(ANNOTATION_TARGET)
            .or_else(|| self.parent()?.cast::<KtAnnotation>()?.use_site_target())
    }
}

/// `org.jetbrains.kotlin.descriptors.annotations.AnnotationUseSiteTarget`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnnotationUseSiteTarget {
    All,
    Field,
    File,
    Property,
    PropertyGetter,
    PropertySetter,
    Receiver,
    ConstructorParameter,
    SetterParameter,
    PropertyDelegateField,
}

impl AnnotationUseSiteTarget {
    pub fn render_name(self) -> &'static str {
        match self {
            AnnotationUseSiteTarget::All => "all",
            AnnotationUseSiteTarget::Field => "field",
            AnnotationUseSiteTarget::File => "file",
            AnnotationUseSiteTarget::Property => "property",
            AnnotationUseSiteTarget::PropertyGetter => "get",
            AnnotationUseSiteTarget::PropertySetter => "set",
            AnnotationUseSiteTarget::Receiver => "receiver",
            AnnotationUseSiteTarget::ConstructorParameter => "param",
            AnnotationUseSiteTarget::SetterParameter => "setparam",
            AnnotationUseSiteTarget::PropertyDelegateField => "delegate",
        }
    }
}

impl KtAnnotationUseSiteTarget {
    /// `getAnnotationUseSiteTarget()`: from the first child's token; upstream throws on unknown ones (None).
    pub fn annotation_use_site_target(&self) -> Option<AnnotationUseSiteTarget> {
        Some(match self.first_child()?.kind() {
            ALL_KEYWORD => AnnotationUseSiteTarget::All,
            FIELD_KEYWORD => AnnotationUseSiteTarget::Field,
            FILE_KEYWORD => AnnotationUseSiteTarget::File,
            PROPERTY_KEYWORD => AnnotationUseSiteTarget::Property,
            GET_KEYWORD => AnnotationUseSiteTarget::PropertyGetter,
            SET_KEYWORD => AnnotationUseSiteTarget::PropertySetter,
            RECEIVER_KEYWORD => AnnotationUseSiteTarget::Receiver,
            PARAM_KEYWORD => AnnotationUseSiteTarget::ConstructorParameter,
            SETPARAM_KEYWORD => AnnotationUseSiteTarget::SetterParameter,
            DELEGATE_KEYWORD => AnnotationUseSiteTarget::PropertyDelegateField,
            _ => return None,
        })
    }
}

macro_rules! context_parameter_list {
    ($($t:ident),*) => {$(impl $t {
        pub fn context_parameters(&self) -> Vec<KtParameter> {
            self.get_stub_or_psi_children(VALUE_PARAMETER)
        }

        pub fn context_receivers(&self) -> Vec<KtContextReceiver> {
            self.get_stub_or_psi_children(CONTEXT_RECEIVER)
        }

        pub fn type_references(&self) -> Vec<KtTypeReference> {
            self.context_receivers().iter().filter_map(KtContextReceiver::type_reference).collect()
        }
    })*};
}

context_parameter_list!(KtContextParameterList, KtContextReceiverList);

impl KtContextReceiver {
    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }
}
