//! Type references and type elements: `KtTypeReference`, `KtUserType`, `KtNullableType`, `KtFunctionType`.

use ktrs_psi::TYPE_ELEMENT_TYPES;
use ktrs_syntax::SyntaxKind::*;

use super::classes::*;
use crate::arena::{Ast, NodeId};

fn first_type_element(ast: &Ast, n: NodeId) -> Option<KtTypeElement> {
    ast.children(n).find(|&c| TYPE_ELEMENT_TYPES.contains(ast.element_type(c))).map(KtTypeElement)
}

impl KtTypeReference {
    pub fn type_element(self, ast: &Ast) -> Option<KtTypeElement> {
        first_type_element(ast, self.0)
    }
}

impl KtNullableType {
    pub fn inner_type(self, ast: &Ast) -> Option<KtTypeElement> {
        first_type_element(ast, self.0)
    }
}

impl KtUserType {
    pub fn reference_expression(self, ast: &Ast) -> Option<KtSimpleNameExpression> {
        let found = ast.find_child_by_type(self.0, REFERENCE_EXPRESSION)
            .or_else(|| ast.find_child_by_type(self.0, ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION))?;
        KtSimpleNameExpression::cast(ast, found)
    }

    /// `getReferencedName()`: the last segment's name (`b` for `a.b<T>`).
    pub fn referenced_name(self, ast: &Ast) -> Option<String> {
        Some(self.reference_expression(ast)?.referenced_name(ast))
    }

    pub fn qualifier(self, ast: &Ast) -> Option<KtUserType> {
        KtUserType::cast(ast, ast.find_child_by_type(self.0, USER_TYPE)?)
    }
}

impl KtFunctionType {
    /// `getReturnTypeReference()`: the type reference child (receiver and parameter types are nested deeper).
    pub fn return_type_reference(self, ast: &Ast) -> Option<KtTypeReference> {
        KtTypeReference::cast(ast, ast.find_child_by_type(self.0, TYPE_REFERENCE)?)
    }
}

impl KtTypeElement {
    /// The element as its concrete class, for `when (typeElement) { is KtFunctionType -> ... }`.
    pub fn as_function_type(self, ast: &Ast) -> Option<KtFunctionType> {
        KtFunctionType::cast(ast, self.0)
    }

    pub fn as_nullable_type(self, ast: &Ast) -> Option<KtNullableType> {
        KtNullableType::cast(ast, self.0)
    }

    pub fn as_user_type(self, ast: &Ast) -> Option<KtUserType> {
        KtUserType::cast(ast, self.0)
    }
}
