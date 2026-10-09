//! `EmptyInitBlock.kt`.

use ktrs_psi::KtClassInitializer;

use super::empty_rule::EmptyRule;

empty_rule!(EmptyInitBlock);

crate::detekt_visitor! {
    impl EmptyInitBlock {
        fn visit_class_initializer(&mut self, initializer: &KtClassInitializer) {
            if let Some(body) = initializer.body() {
                self.add_finding_if_block_expr_is_empty(&body);
            }
        }
    }
}
