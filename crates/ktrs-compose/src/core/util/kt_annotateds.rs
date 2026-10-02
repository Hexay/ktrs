//! Port of `core/util/KtAnnotateds.kt`. `KtAnnotated` receivers are bare nodes (a function, class, type reference,
//! file, ...); `annotationEntries` is `KtAnnotated::of(ast, n).annotation_entries(ast)`.

use ktrs_ast::psi::{KtAnnotated, KtAnnotationEntry, KtLiteralStringTemplateEntry, KtStringTemplateExpression};
use ktrs_ast::{Ast, NodeId};

fn callee_text(ast: &Ast, entry: KtAnnotationEntry) -> Option<String> {
    entry.callee_expression(ast).map(|c| ast.text(c))
}

/// `KtAnnotated.isComposable`.
pub fn is_composable(ast: &Ast, annotated: NodeId) -> bool {
    KtAnnotated::of(ast, annotated).annotation_entries(ast).into_iter().any(|it| callee_text(ast, it).as_deref() == Some("Composable"))
}

/// `KtElement.isSuppressed(suppression)`: a `@Suppress("<suppression>")` on the element or an annotated ancestor.
pub fn is_suppressed(ast: &Ast, element: NodeId, suppression: &str) -> bool {
    ast.parents_with_self(element)
        .filter_map(|it| KtAnnotated::cast(ast, it))
        .flat_map(|it| it.annotation_entries(ast))
        .filter(|it| callee_text(ast, *it).as_deref() == Some("Suppress"))
        .flat_map(|it| it.value_arguments(ast))
        .filter_map(|it| it.argument_expression(ast))
        .filter_map(|it| KtStringTemplateExpression::cast(ast, it))
        .flat_map(|it| it.entries(ast))
        .filter(|&it| KtLiteralStringTemplateEntry::is(ast, it))
        .any(|it| ast.text(it) == suppression)
}

/// `KtAnnotated.isAnnotatedWith(annotations)`.
pub fn is_annotated_with(ast: &Ast, annotated: NodeId, annotations: &[String]) -> bool {
    KtAnnotated::of(ast, annotated)
        .annotation_entries(ast)
        .into_iter()
        .any(|it| callee_text(ast, it).is_some_and(|text| annotations.contains(&text)))
}
