//! `EmptyWhenBlock.kt`.

use ktrs_psi::{KtWhenExpression, kt_visitor_void};

use crate::api::{Entity, Finding, Rule};

empty_rule!(EmptyWhenBlock);

crate::detekt_visitor! {
    impl EmptyWhenBlock {
        fn visit_when_expression(&mut self, expression: &KtWhenExpression) {
            kt_visitor_void::visit_when_expression(self, expression);
            if expression.entries().is_empty() {
                self.report(Finding::new(Entity::from(expression), "This when block is empty."));
            }
        }
    }
}
