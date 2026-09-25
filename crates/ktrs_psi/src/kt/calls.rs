//! Qualified expressions, calls, value arguments, lambdas and simple names.

use ktrs_syntax::SyntaxKind::*;

use crate::element::{AstNode, PsiElement};
use crate::tokens::{KtSingleValueToken, NAME_REFERENCE_EXPRESSIONS, OPERATIONS, OPERATION_TOKENS, single_value};
use crate::tree_util::get_trailing_comma_by_closing_element;
use crate::types::*;

macro_rules! qualified_expression {
    ($($t:ident),*) => {$(impl $t {
        /// `getReceiverExpression()`: upstream throws when absent (None here).
        pub fn receiver_expression(&self) -> Option<KtExpression> {
            self.get_expression(false)
        }

        pub fn selector_expression(&self) -> Option<KtExpression> {
            self.get_expression(true)
        }

        /// `getOperationTokenNode()`: upstream throws when absent (None here).
        pub fn operation_token_node(&self) -> Option<AstNode> {
            self.node().find_child_by_type_set(OPERATIONS)
        }

        /// `getOperationSign()`: upstream throws when absent (None here).
        pub fn operation_sign(&self) -> Option<KtSingleValueToken> {
            Some(KtSingleValueToken(self.operation_token_node()?.element_type()))
        }

        fn get_expression(&self, after_operation: bool) -> Option<KtExpression> {
            self.operation_token_node()?.psi().siblings(after_operation, false).find_map(|s| s.cast())
        }
    })*};
}

qualified_expression!(KtQualifiedExpression, KtDotQualifiedExpression, KtSafeQualifiedExpression);

impl KtCallExpression {
    pub fn callee_expression(&self) -> Option<KtExpression> {
        self.get_stub_or_psi_child(REFERENCE_EXPRESSION).or_else(|| self.find_child_by_class())
    }

    pub fn value_argument_list(&self) -> Option<KtValueArgumentList> {
        self.get_stub_or_psi_child(VALUE_ARGUMENT_LIST)
    }

    pub fn type_argument_list(&self) -> Option<KtTypeArgumentList> {
        self.get_stub_or_psi_child(TYPE_ARGUMENT_LIST)
    }

    pub fn lambda_arguments(&self) -> Vec<KtLambdaArgument> {
        self.get_stub_or_psi_children(LAMBDA_ARGUMENT)
    }

    pub fn value_arguments(&self) -> Vec<KtValueArgument> {
        let mut arguments = self.value_argument_list().map(|l| l.arguments()).unwrap_or_default();
        arguments.extend(self.lambda_arguments().into_iter().map(|a| a.upcast()));
        arguments
    }
}

/// `KtCallElement` methods, dispatched to the implementing class.
impl KtCallElement {
    pub fn callee_expression(&self) -> Option<KtExpression> {
        match self.kind() {
            CALL_EXPRESSION => self.upcast::<KtCallExpression>().callee_expression(),
            ANNOTATION_ENTRY => self.upcast::<KtAnnotationEntry>().callee_expression().map(|c| c.upcast()),
            SUPER_TYPE_CALL_ENTRY => self.upcast::<KtSuperTypeCallEntry>().callee_expression().map(|c| c.upcast()),
            _ => self.upcast::<KtConstructorDelegationCall>().callee_expression().map(|c| c.upcast()),
        }
    }

    pub fn value_argument_list(&self) -> Option<KtValueArgumentList> {
        match self.kind() {
            CALL_EXPRESSION => self.upcast::<KtCallExpression>().value_argument_list(),
            ANNOTATION_ENTRY => self.upcast::<KtAnnotationEntry>().value_argument_list(),
            SUPER_TYPE_CALL_ENTRY => self.upcast::<KtSuperTypeCallEntry>().value_argument_list(),
            _ => self.upcast::<KtConstructorDelegationCall>().value_argument_list(),
        }
    }

    pub fn type_argument_list(&self) -> Option<KtTypeArgumentList> {
        match self.kind() {
            CALL_EXPRESSION => self.upcast::<KtCallExpression>().type_argument_list(),
            ANNOTATION_ENTRY => self.upcast::<KtAnnotationEntry>().type_argument_list(),
            SUPER_TYPE_CALL_ENTRY => self.upcast::<KtSuperTypeCallEntry>().type_argument_list(),
            _ => None,
        }
    }

    pub fn lambda_arguments(&self) -> Vec<KtLambdaArgument> {
        match self.kind() {
            CALL_EXPRESSION => self.upcast::<KtCallExpression>().lambda_arguments(),
            _ => Vec::new(),
        }
    }
}

impl KtValueArgumentList {
    pub fn arguments(&self) -> Vec<KtValueArgument> {
        self.get_stub_or_psi_children(VALUE_ARGUMENT)
    }

    pub fn right_parenthesis(&self) -> Option<PsiElement> {
        self.find_child_by_type(RPAR)
    }

    pub fn left_parenthesis(&self) -> Option<PsiElement> {
        self.find_child_by_type(LPAR)
    }

    pub fn trailing_comma(&self) -> Option<PsiElement> {
        get_trailing_comma_by_closing_element(self.right_parenthesis().as_ref())
    }
}

macro_rules! value_argument {
    ($($t:ident),*) => {$(impl $t {
        pub fn argument_expression(&self) -> Option<KtExpression> {
            self.find_child_by_class()
        }

        pub fn argument_name(&self) -> Option<KtValueArgumentName> {
            self.get_stub_or_psi_child(VALUE_ARGUMENT_NAME)
        }

        pub fn equals_token(&self) -> Option<PsiElement> {
            self.find_child_by_type(EQ)
        }

        pub fn is_named(&self) -> bool {
            self.argument_name().is_some()
        }

        pub fn spread_element(&self) -> Option<LeafPsiElement> {
            Some(self.node().find_child_by_type(MUL)?.psi().upcast())
        }

        pub fn is_spread(&self) -> bool {
            self.spread_element().is_some()
        }
    })*};
}

value_argument!(KtValueArgument, KtLambdaArgument);

impl KtValueArgumentName {
    pub fn reference_expression(&self) -> Option<KtSimpleNameExpression> {
        self.get_stub_or_psi_child(REFERENCE_EXPRESSION)
    }
}

impl KtLambdaExpression {
    /// `getFunctionLiteral()`: upstream throws when absent (None here).
    pub fn function_literal(&self) -> Option<KtFunctionLiteral> {
        self.find_child_by_type(FUNCTION_LITERAL)
    }

    pub fn value_parameters(&self) -> Vec<KtParameter> {
        self.function_literal().map(|f| f.value_parameters()).unwrap_or_default()
    }

    pub fn body_expression(&self) -> Option<KtBlockExpression> {
        self.function_literal()?.body_expression()?.cast()
    }

    pub fn left_curly_brace(&self) -> Option<AstNode> {
        self.function_literal()?.node().find_child_by_type(LBRACE)
    }

    pub fn right_curly_brace(&self) -> Option<AstNode> {
        self.function_literal()?.node().find_child_by_type(RBRACE)
    }
}

impl KtFunctionLiteral {
    pub fn has_parameter_specification(&self) -> bool {
        self.arrow().is_some()
    }

    pub fn l_brace(&self) -> Option<PsiElement> {
        self.find_child_by_type(LBRACE)
    }

    pub fn r_brace(&self) -> Option<PsiElement> {
        self.find_child_by_type(RBRACE)
    }

    pub fn arrow(&self) -> Option<PsiElement> {
        self.find_child_by_type(ARROW)
    }
}

macro_rules! simple_name_expression {
    ($($t:ident),*) => {$(impl $t {
        pub fn identifier(&self) -> Option<PsiElement> {
            match self.kind() {
                ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION => enum_superclass_referenced_element(self)?.name_identifier(),
                _ => self.find_child_by_type(IDENTIFIER),
            }
        }

        /// `getReferencedNameElement()`; `None` only where upstream throws (enum superclass reference
        /// outside an enum class).
        pub fn referenced_name_element(&self) -> Option<PsiElement> {
            match self.kind() {
                REFERENCE_EXPRESSION => {
                    Some(self.find_child_by_type_set(NAME_REFERENCE_EXPRESSIONS).unwrap_or_else(|| PsiElement::clone(self)))
                }
                OPERATION_REFERENCE => Some(self.find_child_by_type_set(OPERATION_TOKENS).unwrap_or_else(|| PsiElement::clone(self))),
                LABEL => Some(self.identifier().unwrap_or_else(|| PsiElement::clone(self))),
                _ => enum_superclass_referenced_element(self).map(PsiElement::from),
            }
        }

        /// `getReferencedName()`; empty only where upstream throws.
        pub fn referenced_name(&self) -> String {
            if self.kind() == ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION {
                let Some(class) = enum_superclass_referenced_element(self) else { return String::new() };
                let name = class.name_identifier().map(|i| unquote_identifier(&i.text()));
                return unquote_identifier_or_field_reference(name.as_deref().unwrap_or("<no name provided>"));
            }
            self.referenced_name_element().map(|e| unquote_identifier_or_field_reference(&e.text())).unwrap_or_default()
        }

        pub fn referenced_name_element_type(&self) -> Option<ktrs_syntax::SyntaxKind> {
            Some(self.referenced_name_element()?.kind())
        }
    })*};
}

simple_name_expression!(
    KtSimpleNameExpression, KtNameReferenceExpression, KtOperationReferenceExpression, KtLabelReferenceExpression,
    KtEnumEntrySuperclassReferenceExpression
);

/// `KtEnumEntrySuperclassReferenceExpression.referencedElement`.
fn enum_superclass_referenced_element(e: &PsiElement) -> Option<KtClass> {
    e.get_parent_of_type::<KtEnumEntry>(true)?.parent()?.parent()?.cast()
}

impl KtOperationReferenceExpression {
    pub fn operation_sign_token_type(&self) -> Option<KtSingleValueToken> {
        let first = self.first_child()?;
        single_value(first.kind()).map(|_| KtSingleValueToken(first.kind()))
    }
}

/// `KtPsiUtil.unquoteIdentifier`.
pub fn unquote_identifier(quoted: &str) -> String {
    if !quoted.contains('`') {
        return quoted.to_owned();
    }
    if quoted.starts_with('`') && quoted.ends_with('`') && quoted.len() >= 2 {
        quoted[1..quoted.len() - 1].to_owned()
    } else {
        quoted.to_owned()
    }
}

/// `KtPsiUtil.unquoteIdentifierOrFieldReference`.
pub fn unquote_identifier_or_field_reference(quoted: &str) -> String {
    if !quoted.contains('`') {
        return quoted.to_owned();
    }
    match quoted.strip_prefix('$') {
        Some(rest) => format!("${}", unquote_identifier(rest)),
        None => unquote_identifier(quoted),
    }
}
