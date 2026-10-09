//! `EmptySecondaryConstructor.kt`.

use ktrs_psi::KtSecondaryConstructor;

use super::empty_rule::EmptyRule;

empty_rule!(EmptySecondaryConstructor);

crate::detekt_visitor! {
    impl EmptySecondaryConstructor {
        fn visit_secondary_constructor(&mut self, constructor: &KtSecondaryConstructor) {
            if let Some(body_expression) = constructor.body_expression() {
                self.add_finding_if_block_expr_is_empty(&body_expression);
            }
        }
    }
}
