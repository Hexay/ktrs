//! `util/ExcludeClass.kt`.

use ktrs_psi::{KtClass, PsiElement};

use crate::kotlin::Regex;

// TODO: isContainingExcludedClassOrObject (rules not ported yet)

/// `KtDeclaration.isContainingExcludedClass(pattern)`.
pub(super) fn is_containing_excluded_class(declaration: &PsiElement, pattern: &Regex) -> bool {
    declaration.get_parent_of_type::<KtClass>(true).and_then(|class| class.name()).is_some_and(|name| pattern.matches(&name))
}
