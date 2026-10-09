//! `KtModifierList.kt`: the `KtModifierListOwner` predicates.

use ktrs_psi::{KtModifierListOwner, PsiElement};
use ktrs_syntax::SyntaxKind::{self, *};

fn has_modifier(owner: &PsiElement, modifier: SyntaxKind) -> bool {
    owner.cast::<KtModifierListOwner>().is_some_and(|owner| owner.has_modifier(modifier))
}

// TODO: isPublicNotOverridden (needs psiUtil `isPublic`, hence `KtPsiUtil.isLocal`)

pub fn is_abstract(owner: &PsiElement) -> bool {
    has_modifier(owner, ABSTRACT_KEYWORD)
}

pub fn is_override(owner: &PsiElement) -> bool {
    has_modifier(owner, OVERRIDE_KEYWORD)
}

pub fn is_open(owner: &PsiElement) -> bool {
    has_modifier(owner, OPEN_KEYWORD)
}

pub fn is_external(owner: &PsiElement) -> bool {
    has_modifier(owner, EXTERNAL_KEYWORD)
}

pub fn is_operator(owner: &PsiElement) -> bool {
    has_modifier(owner, OPERATOR_KEYWORD)
}

pub fn is_constant(owner: &PsiElement) -> bool {
    has_modifier(owner, CONST_KEYWORD)
}

pub fn is_internal(owner: &PsiElement) -> bool {
    has_modifier(owner, INTERNAL_KEYWORD)
}

pub fn is_lateinit(owner: &PsiElement) -> bool {
    has_modifier(owner, LATEINIT_KEYWORD)
}

pub fn is_inline(owner: &PsiElement) -> bool {
    has_modifier(owner, INLINE_KEYWORD)
}

pub fn is_expect(owner: &PsiElement) -> bool {
    has_modifier(owner, EXPECT_KEYWORD)
}

pub fn is_actual(owner: &PsiElement) -> bool {
    has_modifier(owner, ACTUAL_KEYWORD)
}

pub fn is_protected(owner: &PsiElement) -> bool {
    has_modifier(owner, PROTECTED_KEYWORD)
}
