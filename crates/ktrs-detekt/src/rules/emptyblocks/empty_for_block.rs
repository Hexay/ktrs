//! `EmptyForBlock.kt`.

use ktrs_psi::{KtForExpression, kt_visitor_void};

use super::empty_rule::EmptyRule;

empty_rule!(EmptyForBlock);

crate::detekt_visitor! {
    impl EmptyForBlock {
        fn visit_for_expression(&mut self, expression: &KtForExpression) {
            kt_visitor_void::visit_for_expression(self, expression);
            if let Some(body) = expression.body() {
                self.add_finding_if_block_expr_is_empty(&body);
            }
        }
    }
}
