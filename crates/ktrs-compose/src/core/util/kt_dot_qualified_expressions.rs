//! Port of `core/util/KtDotQualifiedExpressions.kt`.

use ktrs_ast::psi::KtDotQualifiedExpression;
use ktrs_ast::{Ast, NodeId};

/// `KtDotQualifiedExpression.rootExpression`: the innermost receiver of the chain.
pub fn root_expression(ast: &Ast, expression: KtDotQualifiedExpression) -> NodeId {
    let mut current = expression.receiver_expression(ast).expect("NullPointerException: receiverExpression");
    while let Some(dot) = KtDotQualifiedExpression::cast(ast, current) {
        current = dot.receiver_expression(ast).expect("NullPointerException: receiverExpression");
    }
    current
}

/// `Sequence<KtDotQualifiedExpression>.dedupUsingOutermost()`: each one's outermost, without duplicates.
pub fn dedup_using_outermost(ast: &Ast, expressions: &[KtDotQualifiedExpression]) -> Vec<KtDotQualifiedExpression> {
    let mut out: Vec<KtDotQualifiedExpression> = Vec::new();
    for outer in expressions.iter().map(|&it| outermost(ast, it)) {
        if !out.contains(&outer) {
            out.push(outer);
        }
    }
    out
}

/// `KtDotQualifiedExpression.outermost`: the last of the `KtDotQualifiedExpression` ancestors-or-self run.
pub fn outermost(ast: &Ast, expression: KtDotQualifiedExpression) -> KtDotQualifiedExpression {
    let last = ast.parents_with_self(expression.node()).take_while(|&it| KtDotQualifiedExpression::is(ast, it)).last();
    KtDotQualifiedExpression::of(ast, last.expect("the expression itself is one"))
}
