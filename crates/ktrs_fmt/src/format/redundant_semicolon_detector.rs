//! Port of `RedundantSemicolonDetector.kt` (lines 32-116).

use ktrs_psi::{
    KtClassBody, KtContainerNodeForControlStructureBody, KtDotQualifiedExpression, KtEnumEntry, KtIfExpression,
    KtLambdaExpression, KtObjectDeclaration, KtStringTemplateEntry, KtStringTemplateExpression, KtWhileExpression,
    LeafPsiElement, PsiElement,
};
use ktrs_syntax::SyntaxKind;

use super::enum_entry_list::EnumEntryList;

#[derive(Default)]
pub struct RedundantSemicolonDetector {
    extra_semicolons: Vec<PsiElement>,
}

impl RedundantSemicolonDetector {
    pub fn get_redundant_semicolon_elements(&self) -> &[PsiElement] {
        &self.extra_semicolons
    }

    pub fn take_element(&mut self, element: &PsiElement) {
        if Self::is_extra_semicolon(element) {
            self.extra_semicolons.push(element.clone());
        }
    }

    fn is_last_concrete_child(element: &PsiElement) -> bool {
        match element.get_next_sibling_ignoring_whitespace_and_comments(false) {
            None => true,
            Some(next_sibling) => next_sibling.is::<LeafPsiElement>() && next_sibling.kind() == SyntaxKind::RBRACE,
        }
    }

    /// Returns true if this element was an extra semicolon.
    fn is_extra_semicolon(element: &PsiElement) -> bool {
        if !element.is::<LeafPsiElement>() || element.kind() != SyntaxKind::SEMICOLON {
            return false;
        }

        let Some(parent) = element.parent() else { return true };
        if parent.is::<KtStringTemplateExpression>() || parent.is::<KtStringTemplateEntry>() {
            return false;
        }

        if parent.is::<KtEnumEntry>() {
            let class_body = parent.parent().expect("enum entry in a class body");
            // Terminating semicolon with no other class members.
            return class_body.children().last() == Some(&parent);
        }

        let prev_concrete_sibling = element.get_prev_sibling_ignoring_whitespace_and_comments(false);
        if let Some(class_body) = parent.cast::<KtClassBody>() {
            if prev_concrete_sibling
                .as_ref()
                .and_then(|p| p.cast::<KtObjectDeclaration>())
                .is_some_and(|o| o.is_companion() && o.name_identifier().is_none())
                && !Self::is_last_concrete_child(element)
            {
                // Example: `class Foo { companion object ; init { } }`
                return false;
            }

            let Some(enum_entry_list) = EnumEntryList::extract_child_list(&class_body) else { return true };
            // Is not terminating semicolon or is terminating with no members.
            return Some(element) != enum_entry_list.terminating_semicolon.as_ref() || class_body.children().is_empty();
        }

        let prev_leaf = element.prev_leaf(false);
        if prev_concrete_sibling.as_ref().is_some_and(|p| p.is::<KtIfExpression>() || p.is::<KtWhileExpression>())
            && prev_leaf.as_ref().is_some_and(|l| l.is::<KtContainerNodeForControlStructureBody>() && l.text().is_empty())
        {
            return false;
        }

        let next_concrete_sibling = element.get_next_sibling_ignoring_whitespace_and_comments(false);

        // Semicolons followed by lambdas (`foo(0) ; { dead -> lambda }.bar()`) are assumed meaningful.
        let next_sibling_is_lambda = next_concrete_sibling.as_ref().is_some_and(|n| {
            n.is::<KtLambdaExpression>()
                || n.cast::<KtDotQualifiedExpression>()
                    .and_then(|d| d.receiver_expression())
                    .is_some_and(|r| r.is::<KtLambdaExpression>())
        });

        !next_sibling_is_lambda
    }
}
