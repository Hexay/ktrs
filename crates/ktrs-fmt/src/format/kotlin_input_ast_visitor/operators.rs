//! `KotlinInputAstVisitor.kt` lines 1230-1361: binary, postfix, prefix and labeled expressions.

use std::collections::VecDeque;

use ktrs_psi::*;
use ktrs_syntax::SyntaxKind;

use crate::doc::{FillMode, Indent};

use super::KotlinInputAstVisitor;

impl KotlinInputAstVisitor<'_, '_, '_> {
    /// For example `a + b`, `a + b + c` or `a..b`. The AST parses `a + b + c` as `a + (b + c)`; drill
    /// to the leftmost operand so the chain is formatted as `(a + b) + c`.
    pub(super) fn visit_binary_expression(&mut self, expression: &KtBinaryExpression) {
        self.sync(expression);
        let op = expression.operation_token();

        if op.is_some_and(|op| ALL_ASSIGNMENTS.contains(op)) && self.is_lambda_or_scoping_function(expression.right().as_ref()) {
            // Assignments are statements in Kotlin; we don't have to worry about compound assignment.
            let Some(operation_reference) = expression.operation_reference() else { return self.fail() };
            self.visit(expression.left().as_ref());
            self.builder.space();
            self.token(operation_reference.text_slice());
            self.visit_lambda_or_scoping_function(expression.right().as_deref(), true);
            return;
        }

        let mut parts = VecDeque::new();
        let mut current: Option<KtExpression> = Some(expression.upcast());
        while let Some(binary) = current.as_ref().and_then(|c| c.cast::<KtBinaryExpression>()) {
            if binary.operation_token() != op {
                break;
            }
            current = binary.left();
            parts.push_front(binary);
        }

        let left_most_expression = parts[0].clone();
        self.visit(left_most_expression.left().as_ref());
        for left_expression in &parts {
            let is_first = *left_expression == left_most_expression;
            let Some(operation_reference) = left_expression.operation_reference() else { return self.fail() };

            match left_expression.operation_token() {
                Some(SyntaxKind::RANGE | SyntaxKind::RANGE_UNTIL) => {
                    if is_first {
                        self.builder.open(self.expression_break_indent());
                    }
                    self.token(operation_reference.text_slice());
                }
                Some(SyntaxKind::ELVIS) => {
                    if is_first {
                        self.builder.open(self.expression_break_indent());
                    }
                    self.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
                    self.token(operation_reference.text_slice());
                    self.builder.space();
                }
                _ => {
                    self.builder.space();
                    if is_first {
                        self.builder.open(self.expression_break_indent());
                    }
                    self.token(operation_reference.text_slice());
                    let fill_mode = if self.has_line_breaking_comment_before(&operation_reference) {
                        FillMode::Independent
                    } else {
                        FillMode::Unified
                    };
                    self.builder.break_op(fill_mode, " ", Indent::ZERO);
                }
            }
            self.visit(left_expression.right().as_ref());
        }
        self.builder.close();
    }

    /// Whether a line-breaking comment precedes `element`: any `//` comment, or a block comment on
    /// its own line (inline ones like `x /*tag*/ ||` don't force a break).
    fn has_line_breaking_comment_before(&self, element: &PsiElement) -> bool {
        let mut prev = element.prev_sibling();
        while let Some(p) = prev.as_ref().filter(|p| p.is::<PsiWhiteSpace>()) {
            prev = p.prev_sibling();
        }
        let Some(prev) = prev.filter(|p| p.is::<PsiComment>()) else { return false };

        // Line comments always force a line break
        if prev.text_slice().starts_with("//") {
            return true;
        }

        // Block comments force a break only if on their own line
        prev.prev_sibling().is_some_and(|before_comment| before_comment.is::<PsiWhiteSpace>() && before_comment.text_slice().contains('\n'))
    }

    pub(super) fn visit_postfix_expression(&mut self, expression: &KtPostfixExpression) {
        self.sync(expression);
        let Some(operation_reference) = expression.operation_reference() else { return self.fail() };
        self.block(Indent::ZERO, |v| {
            let base_expression = expression.base_expression();
            let operator = operation_reference.text();

            v.visit(base_expression.as_ref());
            if base_expression
                .as_ref()
                .and_then(|b| b.cast::<KtPostfixExpression>())
                .and_then(|b| b.operation_reference())
                .is_some_and(|r| r.text_slice().chars().last() == operator.chars().next())
            {
                v.builder.space();
            }
            v.token(&operator);
        });
    }

    pub(super) fn visit_prefix_expression(&mut self, expression: &KtPrefixExpression) {
        self.sync(expression);
        let Some(operation_reference) = expression.operation_reference() else { return self.fail() };
        self.block(Indent::ZERO, |v| {
            let base_expression = expression.base_expression();
            let operator = operation_reference.text();

            v.token(&operator);
            if base_expression
                .as_ref()
                .and_then(|b| b.cast::<KtPrefixExpression>())
                .and_then(|b| b.operation_reference())
                .is_some_and(|r| operator.chars().last() == r.text_slice().chars().next())
            {
                v.builder.space();
            }
            v.visit(base_expression.as_ref());
        });
    }

    pub(super) fn visit_labeled_expression(&mut self, expression: &KtLabeledExpression) {
        self.sync(expression);
        self.visit(expression.label_qualifier().as_ref());
        if !expression.base_expression().is_some_and(|b| b.is::<KtLambdaExpression>()) {
            self.builder.space();
        }
        self.visit(expression.base_expression().as_ref());
    }
}
