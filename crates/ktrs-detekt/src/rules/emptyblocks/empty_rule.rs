//! `EmptyRule.kt`: the base of the empty-blocks rules.

use ktrs_psi::{KtBlockExpression, KtIfExpression, PsiElement, single_value};

use super::contains_comments::has_comment_inside;
use crate::api::{Entity, Finding, Rule};

pub(super) const DESCRIPTION: &str = "Empty block of code detected. As they serve no purpose they should be removed.";

/// Rule to detect empty blocks of code.
pub(super) trait EmptyRule: Rule {
    fn finding_message(&self) -> &'static str {
        "This empty block of code can be removed."
    }

    fn add_finding_if_block_expr_is_empty(&mut self, expression: &PsiElement) {
        self.check_block_expr(expression, false);
    }

    fn add_finding_if_block_expr_is_empty_and_not_commented(&mut self, expression: &PsiElement) {
        self.check_block_expr(expression, true);
    }

    /// Calls `report_block` when a `;` token directly follows the `if` expression.
    fn check_then_body_for_lone_semicolon(&mut self, expression: &KtIfExpression, report_block: impl FnOnce(&mut Self, &KtIfExpression))
    where
        Self: Sized,
    {
        let value_of_next_sibling = expression.next_sibling().filter(PsiElement::is_leaf).and_then(|s| single_value(s.kind()));
        if value_of_next_sibling.map(str::trim) == Some(";") {
            report_block(self, expression);
        }
    }

    fn check_block_expr(&mut self, expression: &PsiElement, skip_if_commented: bool) {
        let Some(block) = expression.cast::<KtBlockExpression>() else { return };
        let has_comment = has_comment_inside(&block);
        if skip_if_commented && has_comment {
            return;
        }
        if !block.has_children() && !has_comment {
            let finding_message = self.finding_message();
            self.report(Finding::new(Entity::from(&block), finding_message));
        }
    }
}
