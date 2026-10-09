//! `EmptyDoWhileBlock.kt`.

use ktrs_psi::{KtDoWhileExpression, kt_visitor_void};

use super::empty_rule::EmptyRule;

empty_rule!(EmptyDoWhileBlock);

crate::detekt_visitor! {
    impl EmptyDoWhileBlock {
        fn visit_do_while_expression(&mut self, expression: &KtDoWhileExpression) {
            kt_visitor_void::visit_do_while_expression(self, expression);
            if let Some(body) = expression.body() {
                self.add_finding_if_block_expr_is_empty(&body);
            }
        }
    }
}
