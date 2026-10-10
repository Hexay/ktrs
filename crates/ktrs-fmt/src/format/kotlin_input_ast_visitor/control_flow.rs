//! `ControlFlowExpressionFormatter.kt` (`when` and its conditions, `if`), `DeclarationFormatter.kt` (class
//! bodies, blocks), `CallFormatter.kt` (array access).

use ktrs_psi::*;

use crate::doc::{BlankLineWanted, FillMode, Indent, RealOrImaginary};
use crate::format::enum_entry_list::EnumEntryList;

use super::KotlinInputAstVisitor;
use super::comma_separated::{EachCommaSeparated, psi_list};

impl KotlinInputAstVisitor<'_, '_, '_> {
    pub(super) fn visit_when_expression(&mut self, expression: &KtWhenExpression) {
        self.sync(expression);
        let block_indent = self.block_indent();
        let ebi = self.expression_break_indent();
        self.block(Indent::ZERO, |v| {
            v.emit_keyword_with_condition("when", expression.subject_expression().as_ref(), true);

            v.builder.space();
            v.builder.token("{", RealOrImaginary::Real, block_indent.clone(), Some(block_indent.clone()));

            for (index, when_entry) in expression.entries().iter().enumerate() {
                v.block(block_indent.clone(), |v| {
                    if index != 0 {
                        // preserve new line if there's one
                        v.builder.blank_line_wanted(BlankLineWanted::PRESERVE);
                    }
                    v.builder.forced_break();
                    v.block(Indent::ZERO, |v| {
                        if when_entry.else_keyword().is_some() {
                            v.token("else");
                        } else {
                            let conditions = when_entry.conditions();
                            for (index, condition) in conditions.iter().enumerate() {
                                v.visit(Some(condition));
                                v.builder.guess_token(",");
                                if index != conditions.len() - 1 {
                                    v.builder.forced_break();
                                }
                            }
                        }
                        if let Some(guard) = when_entry.guard() {
                            v.builder.space();
                            v.emit_keyword_with_condition("if", guard.expression().as_ref(), false);
                        }
                    });
                    let when_expression = when_entry.expression();
                    if when_entry.trailing_comma().is_some() {
                        v.builder.forced_break();
                    } else {
                        v.builder.space();
                    }
                    v.token("->");
                    if when_expression.as_ref().is_some_and(|e| e.is::<KtBlockExpression>() || e.is::<KtLambdaExpression>()) {
                        v.builder.space();
                        v.visit(when_expression.as_ref());
                    } else {
                        v.block(ebi.clone(), |v| {
                            v.builder.break_op(FillMode::Independent, " ", Indent::ZERO);
                            v.visit(when_expression.as_ref());
                        });
                    }
                    v.builder.guess_token(";");
                });
                v.builder.forced_break();
            }
            v.token("}");
        });
    }

    pub(super) fn visit_class_body(&mut self, body: &KtClassBody) {
        self.sync(body);
        self.emit_braced_block(body, |v, children| {
            let enum_entry_list = EnumEntryList::extract_child_list(body);
            let members: Vec<&PsiElement> = children.iter().filter(|c| !c.is::<KtEnumEntry>()).collect();

            if let Some(enum_entry_list) = enum_entry_list {
                v.block(Indent::ZERO, |v| {
                    v.builder.break_op(FillMode::Unified, "", Indent::ZERO);
                    for value in &enum_entry_list.enum_entries {
                        v.visit(Some(value));
                        if v.builder.peek_token() == Some(",") {
                            v.token(",");
                            v.builder.forced_break();
                        }
                    }
                });
                v.builder.guess_token(";");

                if !members.is_empty() {
                    v.builder.forced_break();
                    v.builder.blank_line_wanted(BlankLineWanted::YES);
                }
            } else {
                let parent = body.parent();
                if parent.and_then(|p| p.cast::<KtClass>()).is_some_and(|p| p.is_enum()) && !children.is_empty() {
                    v.token(";");
                    v.builder.forced_break();
                }
            }

            let mut prev: Option<&PsiElement> = None;
            for curr in members {
                let blank_line_between_members = match prev.map(|p| p.cast::<KtProperty>()) {
                    None => BlankLineWanted::PRESERVE,
                    Some(None) => BlankLineWanted::YES,
                    Some(Some(prev)) if prev.getter().is_some() || prev.setter().is_some() => BlankLineWanted::YES,
                    Some(Some(_)) if curr.is::<KtProperty>() => BlankLineWanted::PRESERVE,
                    Some(Some(_)) => BlankLineWanted::YES,
                };
                v.builder.blank_line_wanted(blank_line_between_members);

                v.mark_for_partial_format();
                v.block(Indent::ZERO, |v| v.visit(Some(curr)));
                v.mark_for_partial_format();
                v.builder.guess_token(";");
                v.builder.forced_break();

                prev = Some(curr);
            }
        });
    }

    pub(super) fn visit_block_expression(&mut self, expression: &KtBlockExpression) {
        self.sync(expression);
        self.emit_braced_block(expression, |v, children| v.visit_statements(&children));
    }

    pub(super) fn visit_when_condition_with_expression(&mut self, condition: &KtWhenConditionWithExpression) {
        self.sync(condition);
        self.visit(condition.expression().as_ref());
    }

    pub(super) fn visit_when_condition_is_pattern(&mut self, condition: &KtWhenConditionIsPattern) {
        self.sync(condition);
        self.token(if condition.is_negated() { "!is" } else { "is" });
        self.builder.space();
        self.visit(condition.type_reference().as_ref());
    }

    /// Example `in 1..2` as part of a when expression
    pub(super) fn visit_when_condition_in_range(&mut self, condition: &KtWhenConditionInRange) {
        self.sync(condition);
        self.token(if condition.is_negated() { "!in" } else { "in" });
        self.builder.space();
        self.visit(condition.range_expression().as_ref());
    }

    pub(super) fn visit_if_expression(&mut self, expression: &KtIfExpression) {
        self.sync(expression);
        let ebi = self.expression_break_indent();
        self.block(Indent::ZERO, |v| {
            v.emit_keyword_with_condition("if", expression.condition().as_ref(), true);

            let then = expression.then();
            let then_is_block = then.as_ref().is_some_and(|t| t.is::<KtBlockExpression>());
            if then_is_block {
                v.builder.space();
                v.block(Indent::ZERO, |v| v.visit(then.as_ref()));
            } else {
                v.builder.break_op(FillMode::Independent, " ", ebi.clone());
                v.block(ebi.clone(), |v| {
                    v.fence_comments();
                    v.visit(then.as_ref());
                });
            }

            if expression.else_keyword().is_some() {
                if then_is_block {
                    v.builder.space();
                } else {
                    v.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
                }

                v.block(Indent::ZERO, |v| {
                    v.token("else");
                    let else_ = expression.r#else();
                    if else_.as_ref().is_some_and(|e| e.is::<KtBlockExpression>() || e.is::<KtIfExpression>()) {
                        v.builder.space();
                        v.block(Indent::ZERO, |v| v.visit(else_.as_ref()));
                    } else {
                        v.builder.break_op(FillMode::Independent, " ", ebi.clone());
                        v.block(ebi.clone(), |v| v.visit(else_.as_ref()));
                    }
                });
            }
        });
    }

    /// Example `a[3]`, `b["a", 5]` or `a.b.c[4]`
    pub(super) fn visit_array_access_expression(&mut self, expression: &KtArrayAccessExpression) {
        self.sync(expression);
        if expression.array_expression().is_some_and(|a| a.is::<KtQualifiedExpression>()) {
            self.emit_qualified_expression(&expression.upcast());
        } else {
            self.visit(expression.array_expression().as_ref());
            self.visit_array_access_brackets(expression);
        }
    }

    /// Example `[3]` in `a[3]` or `a[3].b`; shared by top-level array expressions and qualified chains.
    pub(super) fn visit_array_access_brackets(&mut self, expression: &KtArrayAccessExpression) {
        self.block(self.expression_break_indent(), |v| {
            v.visit_each_comma_separated(
                &psi_list(expression.index_expressions()),
                EachCommaSeparated {
                    has_trailing_comma: expression.trailing_comma().is_some(),
                    wrap_in_block: true,
                    prefix: Some("["),
                    postfix: Some("]"),
                    break_before_postfix: false,
                    ..v.comma_separated()
                },
            );
        });
    }
}
