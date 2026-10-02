//! Port of `ktlint/Material2Check.kt`: opt-in through `compose_disallow_material2`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFile;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::ktlint::editor_config_properties::{ALLOWED_FROM_M2, ComposeProperty, DISALLOW_MATERIAL2};
use crate::ktlint::ktlint_rule::KtlintRule;
use crate::rules::material2::Material2;

pub struct Material2Check {
    visitor: Material2,
}

impl ComposeKtVisitor for Material2Check {
    fn visit_file(&self, ast: &mut Ast, file: KtFile, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if config.get_boolean("disallowMaterial2", false) {
            self.visitor.visit_file(ast, file, emitter, config);
        }
    }
}

pub const CHECK: Option<fn() -> KtlintRule> = Some(material2_check);

pub fn material2_check() -> KtlintRule {
    KtlintRule::new(
        "compose:material-two",
        vec![ComposeProperty::String(&ALLOWED_FROM_M2), ComposeProperty::Boolean(&DISALLOW_MATERIAL2)],
        Box::new(Material2Check { visitor: Material2 }),
    )
}
