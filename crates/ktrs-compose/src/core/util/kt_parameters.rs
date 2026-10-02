//! Port of `core/util/KtParameters.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtParameter;

/// `KtParameter.isTypeNullable`: the type element is a `KtNullableType`.
pub fn is_type_nullable(ast: &Ast, parameter: KtParameter) -> bool {
    parameter.type_reference(ast).and_then(|t| t.type_element(ast)).is_some_and(|e| e.as_nullable_type(ast).is_some())
}
