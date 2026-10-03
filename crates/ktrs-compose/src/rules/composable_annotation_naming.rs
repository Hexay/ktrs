//! Port of `rules/ComposableAnnotationNaming.kt`.

use ktrs_ast::Ast;
use ktrs_ast::psi::KtClass;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;

pub struct ComposableAnnotationNaming;

impl ComposeKtVisitor for ComposableAnnotationNaming {
    fn visit_class(&self, ast: &mut Ast, clazz: KtClass, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        if !clazz.is_annotation(ast) {
            return;
        }
        if !is_composable_target_marker_annotation(ast, clazz) {
            return;
        }
        let name = clazz.name_as_safe_name(ast);
        if !name.ends_with("Composable") {
            emitter.report(ast, clazz.node(), COMPOSABLE_ANNOTATION_DOES_NOT_END_WITH_COMPOSABLE, false);
        }
    }
}

/// `KtAnnotated.isComposableTargetMarkerAnnotation`.
fn is_composable_target_marker_annotation(ast: &Ast, clazz: KtClass) -> bool {
    clazz
        .annotation_entries(ast)
        .into_iter()
        .any(|it| it.callee_expression(ast).is_some_and(|c| ast.text(c).contains("ComposableTargetMarker")))
}

pub const COMPOSABLE_ANNOTATION_DOES_NOT_END_WITH_COMPOSABLE: &str = "\
Composable annotations (e.g. tagged with `@ComposableTargetMarker`) should have the `Composable` suffix.
See https://mrmans0n.github.io/compose-rules/rules/#naming-composable-annotations-properly for more information.";
