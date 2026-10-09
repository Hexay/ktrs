//! `EmptyTryBlock.kt`.

use ktrs_psi::{KtTryExpression, kt_visitor_void};

use super::empty_rule::EmptyRule;

empty_rule!(EmptyTryBlock);

crate::detekt_visitor! {
    impl EmptyTryBlock {
        fn visit_try_expression(&mut self, expression: &KtTryExpression) {
            kt_visitor_void::visit_try_expression(self, expression);
            if let Some(try_block) = expression.try_block() {
                self.add_finding_if_block_expr_is_empty(&try_block);
            }
        }
    }
}
