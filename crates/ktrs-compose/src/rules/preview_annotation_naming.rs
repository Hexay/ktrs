//! Port of `rules/PreviewAnnotationNaming.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtClass;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::previews::is_preview;

pub struct PreviewAnnotationNaming;

impl ComposeKtVisitor for PreviewAnnotationNaming {
    fn visit_class(&self, ast: &mut Ast, clazz: KtClass, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        if !clazz.is_annotation(ast) {
            return;
        }
        if !is_preview(ast, clazz.node()) {
            return;
        }
        if !clazz.name_as_safe_name(ast).starts_with("Preview") {
            emitter.report(ast, clazz.node(), PREVIEW_ANNOTATION_DOES_NOT_START_WITH_PREVIEW, false);
        }
    }
}

pub const PREVIEW_ANNOTATION_DOES_NOT_START_WITH_PREVIEW: &str = "\
MultiPreview annotations should start with `Preview` as prefix.
See https://mrmans0n.github.io/compose-rules/rules/#naming-multipreview-annotations-properly for more information.";
