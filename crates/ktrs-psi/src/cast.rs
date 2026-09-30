//! Typed views: every PSI class is a newtype over [`PsiElement`] with a kind-based `instanceof`.

use crate::element::PsiElement;

/// A PSI class (concrete or abstract/interface). `can_cast` is Java's `instanceof`.
pub trait PsiType: Clone {
    fn can_cast(e: &PsiElement) -> bool;
    /// Wraps without checking; callers guarantee `can_cast` (e.g. a child found by its element type).
    fn cast_unchecked(e: PsiElement) -> Self;
    fn psi(&self) -> &PsiElement;
}

impl PsiType for PsiElement {
    fn can_cast(_: &PsiElement) -> bool {
        true
    }

    fn cast_unchecked(e: PsiElement) -> Self {
        e
    }

    fn psi(&self) -> &PsiElement {
        self
    }
}

impl PsiElement {
    /// Java's implicit upcast (or a cast the caller knows holds); panics if `self` is not a `T`.
    pub fn upcast<T: PsiType>(&self) -> T {
        self.cast::<T>().unwrap_or_else(|| panic!("{self:?} is not a {}", std::any::type_name::<T>()))
    }
}

/// Declares PSI newtypes: `Name(e) => predicate over e;`.
macro_rules! psi_types {
    ($( $(#[$doc:meta])* $name:ident($e:ident) => $test:expr; )*) => {$(
        $(#[$doc])*
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct $name($crate::element::PsiElement);

        impl $crate::cast::PsiType for $name {
            fn can_cast($e: &$crate::element::PsiElement) -> bool {
                $test
            }

            fn cast_unchecked(e: $crate::element::PsiElement) -> Self {
                $name(e)
            }

            fn psi(&self) -> &$crate::element::PsiElement {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = $crate::element::PsiElement;

            fn deref(&self) -> &$crate::element::PsiElement {
                &self.0
            }
        }

        impl From<$name> for $crate::element::PsiElement {
            fn from(value: $name) -> $crate::element::PsiElement {
                value.0
            }
        }
    )*};
}

pub(crate) use psi_types;
