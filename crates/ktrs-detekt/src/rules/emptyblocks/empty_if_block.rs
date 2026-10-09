//! `EmptyIfBlock.kt`.

use ktrs_psi::{KtIfExpression, kt_visitor_void};

use super::empty_rule::EmptyRule;
use crate::api::{Entity, Finding, Rule};

empty_rule!(EmptyIfBlock);

crate::detekt_visitor! {
    impl EmptyIfBlock {
        fn visit_if_expression(&mut self, expression: &KtIfExpression) {
            kt_visitor_void::visit_if_expression(self, expression);
            match expression.then() {
                Some(then) => self.add_finding_if_block_expr_is_empty(&then),
                None => self.check_then_body_for_lone_semicolon(expression, |rule, it| {
                    rule.report(Finding::new(Entity::from(it), "This if block is empty and can be removed."));
                }),
            }
        }
    }
}
