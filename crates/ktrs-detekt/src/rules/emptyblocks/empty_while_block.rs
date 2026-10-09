//! `EmptyWhileBlock.kt`.

use ktrs_psi::{KtWhileExpression, kt_visitor_void};

use super::empty_rule::EmptyRule;

empty_rule!(EmptyWhileBlock);

crate::detekt_visitor! {
    impl EmptyWhileBlock {
        fn visit_while_expression(&mut self, expression: &KtWhileExpression) {
            kt_visitor_void::visit_while_expression(self, expression);
            if let Some(body) = expression.body() {
                self.add_finding_if_block_expr_is_empty(&body);
            }
        }
    }
}
