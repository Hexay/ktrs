//! Port of `ktlint/UnstableCollectionsCheck.kt`: opt-in through `compose_disallow_unstable_collections` (ktlint
//! runs every rule by default).

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::ktlint::editor_config_properties::{ComposeProperty, DISALLOW_UNSTABLE_COLLECTIONS};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::unstable_collections::UnstableCollections;

pub struct UnstableCollectionsCheck {
    visitor: UnstableCollections,
}

impl ComposeKtVisitor for UnstableCollectionsCheck {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if config.get_boolean("disallowUnstableCollections", false) {
            self.visitor.visit_composable(ast, function, emitter, config);
        }
    }
}

pub const CHECK: Option<fn() -> KtlintRule> = Some(unstable_collections_check);

pub fn unstable_collections_check() -> KtlintRule {
    KtlintRule::new(
        "compose:unstable-collections",
        vec![ComposeProperty::Boolean(&DISALLOW_UNSTABLE_COLLECTIONS)],
        Box::new(UnstableCollectionsCheck { visitor: UnstableCollections }),
    )
}
