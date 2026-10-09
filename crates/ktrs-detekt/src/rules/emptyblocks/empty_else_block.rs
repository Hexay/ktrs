//! `EmptyElseBlock.kt`.

use ktrs_psi::{KtIfExpression, kt_visitor_void};

use super::empty_rule::EmptyRule;
use crate::api::{Entity, Finding, Rule};

empty_rule!(EmptyElseBlock);

crate::detekt_visitor! {
    impl EmptyElseBlock {
        fn visit_if_expression(&mut self, expression: &KtIfExpression) {
            kt_visitor_void::visit_if_expression(self, expression);
            match expression.r#else() {
                Some(r#else) => self.add_finding_if_block_expr_is_empty(&r#else),
                None => self.check_then_body_for_lone_semicolon(expression, |rule, it| {
                    rule.report(Finding::new(Entity::from(it), "This else block is empty and can be removed."));
                }),
            }
        }
    }
}
