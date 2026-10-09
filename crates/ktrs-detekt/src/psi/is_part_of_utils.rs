//! `IsPartOfUtils.kt`.

use ktrs_psi::{PsiElement, PsiType};

/// Tests if this element is part of given PsiElement.
pub fn is_part_of<T: PsiType>(element: &PsiElement) -> bool {
    element.get_parent_of_type::<T>(false).is_some()
}
