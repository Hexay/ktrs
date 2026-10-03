//! Port of `rules/PreviewNaming.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::previews::{has_preview_wrapper, is_preview};

pub struct PreviewNaming;

enum PreviewNamingType {
    Suffix,
    Prefix,
    Anywhere,
}

impl ComposeKtVisitor for PreviewNaming {
    fn is_opt_in(&self) -> bool {
        true
    }

    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, config: &dyn ComposeKtConfig) {
        if !is_preview(ast, function.node()) || has_preview_wrapper(ast, function.node()) {
            return;
        }
        let strategy = match config.get_string("previewNamingStrategy", Some("suffix")).as_deref() {
            Some("prefix") => PreviewNamingType::Prefix,
            Some("anywhere") => PreviewNamingType::Anywhere,
            _ => PreviewNamingType::Suffix,
        };
        let name = function.name_as_safe_name(ast);
        let kotlin = config.embedded_kotlin();
        match strategy {
            PreviewNamingType::Suffix => {
                if !name.ends_with("Preview") {
                    emitter.report(ast, function.node(), PREVIEW_DOES_NOT_END_WITH_PREVIEW, true).if_fix(|| {
                        function.set_name(ast, &format!("{name}Preview"), kotlin);
                    });
                }
            }
            PreviewNamingType::Prefix => {
                if !name.starts_with("Preview") {
                    emitter.report(ast, function.node(), PREVIEW_DOES_NOT_START_WITH_PREVIEW, true).if_fix(|| {
                        function.set_name(ast, &format!("Preview{name}"), kotlin);
                    });
                }
            }
            PreviewNamingType::Anywhere => {
                if !name.contains("Preview") {
                    emitter.report(ast, function.node(), PREVIEW_DOES_NOT_CONTAIN_PREVIEW, true).if_fix(|| {
                        function.set_name(ast, &format!("{name}Preview"), kotlin);
                    });
                }
            }
        }
    }
}

pub const PREVIEW_DOES_NOT_START_WITH_PREVIEW: &str = "\
Preview functions should have `Preview` as prefix, per your project's configuration.
See https://mrmans0n.github.io/compose-rules/rules/#naming-previews-properly for more information.";

pub const PREVIEW_DOES_NOT_END_WITH_PREVIEW: &str = "\
Preview functions should have `Preview` as suffix, per your project's configuration.
See https://mrmans0n.github.io/compose-rules/rules/#naming-previews-properly for more information.";

pub const PREVIEW_DOES_NOT_CONTAIN_PREVIEW: &str = "\
Preview functions should contain `Preview` in their names, per your project's configuration.
See https://mrmans0n.github.io/compose-rules/rules/#naming-previews-properly for more information.";
