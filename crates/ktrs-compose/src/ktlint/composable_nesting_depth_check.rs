//! Port of `ktlint/ComposableNestingDepthCheck.kt`: opt-in through `compose_composable_nesting_depth_enabled`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::ktlint::editor_config_properties::{
    COMPOSABLE_NESTING_DEPTH_ENABLED, COMPOSABLE_NESTING_DEPTH_THRESHOLD, CONTENT_EMITTERS_DENYLIST,
    CONTENT_EMITTERS_PROPERTY, ComposeProperty,
};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::composable_nesting_depth::ComposableNestingDepth;

pub struct ComposableNestingDepthCheck {
    visitor: ComposableNestingDepth,
}

impl ComposeKtVisitor for ComposableNestingDepthCheck {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if config.get_boolean("composableNestingDepthEnabled", false) {
            self.visitor.visit_composable(ast, function, emitter, config);
        }
    }
}

pub const CHECK: Option<fn() -> KtlintRule> = Some(composable_nesting_depth_check);

pub fn composable_nesting_depth_check() -> KtlintRule {
    KtlintRule::new(
        "compose:composable-nesting-depth-check",
        vec![
            ComposeProperty::Boolean(&COMPOSABLE_NESTING_DEPTH_ENABLED),
            ComposeProperty::Int(&COMPOSABLE_NESTING_DEPTH_THRESHOLD),
            ComposeProperty::String(&CONTENT_EMITTERS_PROPERTY),
            ComposeProperty::String(&CONTENT_EMITTERS_DENYLIST),
        ],
        Box::new(ComposableNestingDepthCheck { visitor: ComposableNestingDepth }),
    )
}
