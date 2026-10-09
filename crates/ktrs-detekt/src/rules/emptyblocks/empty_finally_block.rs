//! `EmptyFinallyBlock.kt`.

use ktrs_psi::{KtFinallySection, kt_visitor_void};

use super::empty_rule::EmptyRule;

empty_rule!(EmptyFinallyBlock);

crate::detekt_visitor! {
    impl EmptyFinallyBlock {
        fn visit_finally_section(&mut self, finally_section: &KtFinallySection) {
            kt_visitor_void::visit_finally_section(self, finally_section);
            if let Some(final_expression) = finally_section.final_expression() {
                self.add_finding_if_block_expr_is_empty(&final_expression);
            }
        }
    }
}
