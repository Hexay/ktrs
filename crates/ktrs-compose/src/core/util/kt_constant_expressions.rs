//! Port of `core/util/KtConstantExpressions.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtConstantExpression;
use ktrs_syntax::SyntaxKind::{FLOAT_CONSTANT, INTEGER_CONSTANT};

pub fn is_int(ast: &Ast, expression: KtConstantExpression) -> bool {
    is_integer_constant(ast, expression) && !is_long(ast, expression) && !is_unsigned_integer(ast, expression)
}

pub fn is_long(ast: &Ast, expression: KtConstantExpression) -> bool {
    is_integer_constant(ast, expression) && expression.text(ast).ends_with('L') && !is_unsigned_integer(ast, expression)
}

pub fn is_double(ast: &Ast, expression: KtConstantExpression) -> bool {
    is_float_constant(ast, expression) && !is_float(ast, expression)
}

pub fn is_float(ast: &Ast, expression: KtConstantExpression) -> bool {
    is_float_constant(ast, expression) && expression.text(ast).ends_with('f')
}

fn is_integer_constant(ast: &Ast, expression: KtConstantExpression) -> bool {
    ast.element_type(expression.node()) == INTEGER_CONSTANT
}

fn is_float_constant(ast: &Ast, expression: KtConstantExpression) -> bool {
    ast.element_type(expression.node()) == FLOAT_CONSTANT
}

/// Ends with `U` or `UL`, ignoring case.
fn is_unsigned_integer(ast: &Ast, expression: KtConstantExpression) -> bool {
    let text = expression.text(ast).to_ascii_lowercase();
    text.ends_with('u') || text.ends_with("ul")
}
