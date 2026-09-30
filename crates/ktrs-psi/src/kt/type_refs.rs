//! Type references and type elements.

use ktrs_syntax::SyntaxKind::*;

use crate::element::{AstNode, PsiElement};
use crate::tokens::TYPE_ELEMENT_TYPES;
use crate::tree_util::get_trailing_comma_by_closing_element;
use crate::types::*;

impl KtTypeReference {
    pub fn type_element(&self) -> Option<KtTypeElement> {
        self.get_stub_or_psi_children_set(TYPE_ELEMENT_TYPES).into_iter().next()
    }

    pub fn has_parentheses(&self) -> bool {
        self.find_child_by_type::<PsiElement>(LPAR).is_some() && self.find_child_by_type::<PsiElement>(RPAR).is_some()
    }
}

impl KtNullableType {
    pub fn question_mark_node(&self) -> Option<AstNode> {
        self.node().find_child_by_type(QUEST)
    }

    pub fn inner_type(&self) -> Option<KtTypeElement> {
        self.get_stub_or_psi_children_set(TYPE_ELEMENT_TYPES).into_iter().next()
    }
}

impl KtUserType {
    pub fn type_argument_list(&self) -> Option<KtTypeArgumentList> {
        self.get_stub_or_psi_child(TYPE_ARGUMENT_LIST)
    }

    pub fn type_arguments(&self) -> Vec<KtTypeProjection> {
        self.type_argument_list().map(|l| l.arguments()).unwrap_or_default()
    }

    pub fn reference_expression(&self) -> Option<KtSimpleNameExpression> {
        self.get_stub_or_psi_child(REFERENCE_EXPRESSION)
            .or_else(|| self.get_stub_or_psi_child(ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION))
    }

    pub fn qualifier(&self) -> Option<KtUserType> {
        self.get_stub_or_psi_child(USER_TYPE)
    }

    pub fn referenced_name(&self) -> Option<String> {
        Some(self.reference_expression()?.referenced_name())
    }
}

impl KtIntersectionType {
    pub fn left_type_ref(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_children(TYPE_REFERENCE).into_iter().next()
    }

    pub fn right_type_ref(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_children(TYPE_REFERENCE).into_iter().nth(1)
    }
}

impl KtTypeArgumentList {
    pub fn arguments(&self) -> Vec<KtTypeProjection> {
        self.get_stub_or_psi_children(TYPE_PROJECTION)
    }

    pub fn trailing_comma(&self) -> Option<PsiElement> {
        get_trailing_comma_by_closing_element(self.find_child_by_type::<PsiElement>(GT).as_ref())
    }
}

/// `KtProjectionKind`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KtProjectionKind {
    In,
    Out,
    Star,
    None,
}

impl KtProjectionKind {
    /// Upstream enum constant name (`IN`, `OUT`, `STAR`, `NONE`).
    pub fn name(self) -> &'static str {
        match self {
            KtProjectionKind::In => "IN",
            KtProjectionKind::Out => "OUT",
            KtProjectionKind::Star => "STAR",
            KtProjectionKind::None => "NONE",
        }
    }
}

impl KtTypeProjection {
    pub fn projection_kind(&self) -> KtProjectionKind {
        match self.projection_token().map(|t| t.kind()) {
            Some(IN_KEYWORD) => KtProjectionKind::In,
            Some(OUT_KEYWORD) => KtProjectionKind::Out,
            Some(MUL) => KtProjectionKind::Star,
            _ => KtProjectionKind::None,
        }
    }

    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }

    pub fn projection_token(&self) -> Option<PsiElement> {
        if let Some(star) = self.find_child_by_type(MUL) {
            return Some(star);
        }
        let modifier_list = self.modifier_list()?;
        modifier_list.find_child_by_type(IN_KEYWORD).or_else(|| modifier_list.find_child_by_type(OUT_KEYWORD))
    }
}

impl KtFunctionType {
    pub fn parameter_list(&self) -> Option<KtParameterList> {
        self.get_stub_or_psi_child(VALUE_PARAMETER_LIST)
    }

    pub fn parameters(&self) -> Vec<KtParameter> {
        self.parameter_list().map(|l| l.parameters()).unwrap_or_default()
    }

    pub fn receiver(&self) -> Option<KtFunctionTypeReceiver> {
        self.get_stub_or_psi_child(FUNCTION_TYPE_RECEIVER)
    }

    pub fn receiver_type_reference(&self) -> Option<KtTypeReference> {
        self.receiver()?.type_reference()
    }

    pub fn context_receiver_list(&self) -> Option<KtContextReceiverList> {
        self.get_stub_or_psi_child(CONTEXT_PARAMETER_LIST)
    }

    pub fn context_parameter_list(&self) -> Option<KtContextParameterList> {
        self.get_stub_or_psi_child(CONTEXT_PARAMETER_LIST)
    }

    pub fn return_type_reference(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }
}

impl KtFunctionTypeReceiver {
    /// `getTypeReference()`: upstream requires it (throws when absent; None here).
    pub fn type_reference(&self) -> Option<KtTypeReference> {
        self.get_stub_or_psi_child(TYPE_REFERENCE)
    }
}
