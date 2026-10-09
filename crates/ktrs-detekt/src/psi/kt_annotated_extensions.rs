//! `KtAnnotatedExtensions.kt`.

use ktrs_psi::{KtAnnotated, KtUserType, PsiElement};

/// `KtAnnotated.hasAnnotation(vararg annotationNames)`: by the annotation type's short name.
pub fn has_annotation(annotated: &PsiElement, annotation_names: &[&str]) -> bool {
    annotated.upcast::<KtAnnotated>().annotation_entries().iter().any(|entry| {
        let referenced_name = entry
            .type_reference()
            .and_then(|type_reference| type_reference.type_element())
            .and_then(|type_element| type_element.cast::<KtUserType>())
            .and_then(|user_type| user_type.referenced_name());
        referenced_name.is_some_and(|name| annotation_names.contains(&name.as_str()))
    })
}
