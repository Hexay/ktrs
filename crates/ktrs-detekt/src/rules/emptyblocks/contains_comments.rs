//! `internal/ContainsComments.kt`.

use ktrs_psi::{KtClassOrObject, PsiComment, PsiElement};

/// `KtClassOrObject.hasCommentInside()`.
pub(super) fn class_has_comment_inside(class_or_object: &KtClassOrObject) -> bool {
    class_or_object.body().is_some_and(|body| has_comment_inside(&body))
}

/// `PsiElement.hasCommentInside()`: a comment among the direct children.
pub(super) fn has_comment_inside(element: &PsiElement) -> bool {
    element.get_child_of_type::<PsiComment>().is_some()
}
