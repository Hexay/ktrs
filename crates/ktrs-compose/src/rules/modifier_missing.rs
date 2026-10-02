//! Port of `rules/ModifierMissing.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::composables::function_emits_content;
use crate::core::util::kt_annotateds::is_annotated_with;
use crate::core::util::kt_functions::{defined_in_interface, is_internal, is_override, returns_value};
use crate::core::util::modifiers::{is_modifier_receiver, modifier_parameter};
use crate::core::util::previews::is_preview;

pub struct ModifierMissing;

impl ComposeKtVisitor for ModifierMissing {
    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if returns_value(ast, function)
            || is_override(ast, function)
            || defined_in_interface(ast, function)
            || is_preview(ast, function.node())
            || is_modifier_receiver(ast, function.node(), config)
        {
            return;
        }
        if is_annotated_with(ast, function.node(), &config.get_set("modifierMissingIgnoreAnnotated", &[])) {
            return;
        }
        let should_check = match config.get_string("checkModifiersForVisibility", Some("only_public")).as_deref() {
            Some("public_and_internal") => function.is_public(ast) || is_internal(ast, function.node()),
            Some("all") => true,
            _ => function.is_public(ast),
        };
        if !should_check || modifier_parameter(ast, function, config).is_some() {
            return;
        }
        if function_emits_content(ast, function, config) {
            emitter.report(ast, function.node(), MISSING_MODIFIER_CONTENT_COMPOSABLE, false);
        }
    }
}

pub const MISSING_MODIFIER_CONTENT_COMPOSABLE: &str = "\
This @Composable function emits content but doesn't have a modifier parameter.
See https://mrmans0n.github.io/compose-rules/rules/#when-should-i-expose-modifier-parameters for more information.";
