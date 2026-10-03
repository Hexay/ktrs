//! Port of `core/util/Previews.kt`.

use ktrs_ast::psi::{KtAnnotated, KtAnnotationEntry};
use ktrs_ast::{Ast, NodeId};

/// `KtAnnotated.isPreview`.
pub fn is_preview(ast: &Ast, annotated: NodeId) -> bool {
    KtAnnotated::of(ast, annotated).annotation_entries(ast).into_iter().any(|it| is_preview_annotation(ast, it))
}

/// `KtAnnotated.hasPreviewWrapper`.
pub fn has_preview_wrapper(ast: &Ast, annotated: NodeId) -> bool {
    KtAnnotated::of(ast, annotated).annotation_entries(ast).into_iter().any(|it| is_preview_wrapper_annotation(ast, it))
}

/// `KtAnnotationEntry.isPreviewAnnotation`: the callee text contains `Preview`.
pub fn is_preview_annotation(ast: &Ast, entry: KtAnnotationEntry) -> bool {
    entry.callee_expression(ast).is_some_and(|c| ast.text(c).contains("Preview"))
}

/// `KtAnnotationEntry.isPreviewWrapperAnnotation`.
pub fn is_preview_wrapper_annotation(ast: &Ast, entry: KtAnnotationEntry) -> bool {
    entry.callee_expression(ast).is_some_and(|c| ast.text(c).contains("PreviewWrapper"))
}
