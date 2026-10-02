//! Port of `ktlint/PreviewNamingCheck.kt`: opt-in through `compose_preview_naming_enabled` (ktlint runs every
//! rule by default).

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::ktlint::editor_config_properties::{COMPOSE_PREVIEW_NAMING_ENABLED, COMPOSE_PREVIEW_NAMING_STRATEGY, ComposeProperty};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::preview_naming::PreviewNaming;

pub struct PreviewNamingCheck {
    visitor: PreviewNaming,
}

impl ComposeKtVisitor for PreviewNamingCheck {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if config.get_boolean("previewNamingEnabled", false) {
            self.visitor.visit_composable(ast, function, emitter, config);
        }
    }
}

pub const CHECK: Option<fn() -> KtlintRule> = Some(preview_naming_check);

pub fn preview_naming_check() -> KtlintRule {
    KtlintRule::new(
        "compose:preview-naming",
        vec![ComposeProperty::Boolean(&COMPOSE_PREVIEW_NAMING_ENABLED), ComposeProperty::String(&COMPOSE_PREVIEW_NAMING_STRATEGY)],
        Box::new(PreviewNamingCheck { visitor: PreviewNaming }),
    )
}
