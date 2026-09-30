//! `KotlinInputAstVisitor.kt` lines 475-764: qualified expression chains and `visitCallExpression`.

use std::collections::VecDeque;
use std::ops::ControlFlow::{Break, Continue};

use ktrs_psi::*;

use crate::doc::{FillMode, Indent};
use crate::format::psi_utils::is_lambda;

use super::KotlinInputAstVisitor;

/// Extra data to help [KotlinInputAstVisitor::emit_qualified_expression] know when to open and close a group.
#[derive(Clone, Default)]
struct GroupingInfo {
    group_open_count: usize,
    should_close_group: bool,
}

/// Kotlin `Char.isUpperCase()` on the first UTF-16 unit of `element.text` (a lone surrogate is not
/// uppercase). Reads only the first char: the element can be a long call chain or lambda.
fn first_unit_is_upper_case(element: &PsiElement) -> bool {
    let first = element.try_for_each_text_chunk(|chunk| chunk.chars().next().map_or(Continue(()), Break));
    first.break_value().is_some_and(|c| (c as u32) <= 0xFFFF && c.is_uppercase())
}

/// `element.text.length < limit`, reading no further than `limit` UTF-16 units.
fn text_shorter_than(element: &PsiElement, limit: i32) -> bool {
    let mut length = 0;
    limit > 0 && element.text_slice().chars().all(|c| {
        length += c.len_utf16() as i32;
        length < limit
    })
}

impl KotlinInputAstVisitor<'_, '_, '_> {
    /// Example: "com.facebook.bla.bla" in imports or "a.b.c.d" in expressions. Imports stay on one
    /// line; other chains go to the leftmost descendant so indentation starts at the first break.
    pub(super) fn visit_qualified_expression(&mut self, expression: &KtQualifiedExpression) {
        self.sync(expression);
        let Some(receiver) = expression.receiver_expression() else { return self.fail() };
        if self.in_import {
            self.visit(Some(&receiver));
            if let Some(selector_expression) = expression.selector_expression() {
                self.token(".");
                self.visit(Some(&selector_expression));
            }
        } else if receiver.is::<KtStringTemplateExpression>() {
            let Some(operation_sign) = expression.operation_sign() else { return self.fail() };
            self.block(self.expression_break_indent(), |v| {
                v.visit(Some(&receiver));
                v.builder.break_op(FillMode::Unified, "", Indent::ZERO);
                v.token(operation_sign.value());
                v.visit(expression.selector_expression().as_ref());
            });
        } else if receiver.is::<KtWhenExpression>() {
            let Some(operation_sign) = expression.operation_sign() else { return self.fail() };
            self.block(Indent::ZERO, |v| {
                v.visit(Some(&receiver));
                v.token(operation_sign.value());
                v.visit(expression.selector_expression().as_ref());
            });
        } else if self.is_chained_scoping_function(expression)
            && self.is_multiline_scoping_function(&self.chain_root(expression))
            && self.chained_selectors_have_no_value_arguments(expression)
        {
            self.visit_chained_scoping_function(expression, false);
        } else {
            self.emit_qualified_expression(&expression.upcast());
        }
    }

    /// Handles a chain of qualified expressions, i.e. `a[5].b!!.c()[4].f()`: breaks it into the
    /// steps it executes in, computes which parts form groups, then emits part by part.
    pub(super) fn emit_qualified_expression(&mut self, expression: &KtExpression) {
        let parts = self.break_into_parts(expression);
        // whether we want to make a lambda look like a block, this make Kotlin DSLs look as expected
        let use_block_like_lambda_style =
            is_lambda(parts.last().unwrap()) && parts.iter().filter(|it| is_lambda(it)).count() == 1;
        let Some(grouping_infos) = self.compute_grouping_info(&parts, use_block_like_lambda_style) else {
            return self.fail();
        };
        let ebi = self.expression_break_indent();
        let negative = self.expression_break_negative_indent();
        self.block(ebi.clone(), |v| {
            let name_tag = v.gen_sym(); // allows adjusting arguments indentation if a break will be made
            for (index, kt_expression) in parts.iter().enumerate() {
                if kt_expression.is::<KtQualifiedExpression>() {
                    v.builder.break_op_tagged(FillMode::Unified, "", Indent::ZERO, Some(name_tag.clone()));
                }
                for _ in 0..grouping_infos[index].group_open_count {
                    v.builder.open(Indent::ZERO);
                }
                if let Some(qualified) = kt_expression.cast::<KtQualifiedExpression>() {
                    let Some(operation_sign) = qualified.operation_sign() else { return v.fail() };
                    v.token(operation_sign.value());
                    let selector_expression = qualified.selector_expression();
                    match selector_expression.as_ref().and_then(|s| s.cast::<KtCallExpression>()) {
                        None => {
                            // selector is a simple field access
                            v.visit(selector_expression.as_ref());
                            if grouping_infos[index].should_close_group {
                                v.builder.close();
                            }
                        }
                        Some(selector_expression) => {
                            // emit `doIt` from `doIt(1, 2) { it }`, closing groups after the name
                            v.visit(selector_expression.callee_expression().as_ref());
                            if grouping_infos[index].should_close_group {
                                v.builder.close();
                            }
                            // close group due to last lambda to allow block-like style in `as.forEach { ... }`
                            let is_trailing_lambda = use_block_like_lambda_style && index == parts.len() - 1;
                            if is_trailing_lambda {
                                v.builder.close();
                            }
                            let args_indent_else = if index == parts.len() - 1 { Indent::ZERO } else { ebi.clone() };
                            let lambda_indent_else = if is_trailing_lambda { negative.clone() } else { Indent::ZERO };
                            let negative_lambda_indent_else = if is_trailing_lambda { ebi.clone() } else { Indent::ZERO };

                            // emit `(1, 2) { it }` from `doIt(1, 2) { it }`
                            v.visit_call_element(
                                None,
                                selector_expression.type_argument_list().as_ref(),
                                selector_expression.value_argument_list().as_ref(),
                                &selector_expression.lambda_arguments(),
                                Indent::make_if(&name_tag, ebi.clone(), args_indent_else),
                                Indent::make_if(&name_tag, Indent::ZERO, lambda_indent_else),
                                Indent::make_if(&name_tag, Indent::ZERO, negative_lambda_indent_else),
                            );
                        }
                    }
                } else if let Some(array_access) = kt_expression.cast::<KtArrayAccessExpression>() {
                    v.visit_array_access_brackets(&array_access);
                    v.builder.close();
                } else if let Some(postfix) = kt_expression.cast::<KtPostfixExpression>() {
                    let Some(operation_reference) = postfix.operation_reference() else { return v.fail() };
                    v.token(operation_reference.text_slice());
                    v.builder.close();
                } else {
                    if index != 0 {
                        return v.throw_runtime("Check failed.");
                    }
                    v.visit(Some(kt_expression));
                }
            }
        });
    }

    /// Decomposes a qualified expression into parts, so `rainbow.red.orange.yellow` becomes
    /// `[rainbow, rainbow.red, rainbow.red.orange, rainbow.orange.yellow]`.
    pub(super) fn break_into_parts(&self, expression: &KtExpression) -> Vec<KtExpression> {
        let mut parts = VecDeque::new();

        // add elements to the beginning so the innermost expression comes first
        let mut node = Some(expression.clone());
        while let Some(n) = node {
            node = if let Some(q) = n.cast::<KtQualifiedExpression>() {
                q.receiver_expression()
            } else if let Some(a) = n.cast::<KtArrayAccessExpression>() {
                a.array_expression()
            } else if let Some(p) = n.cast::<KtPostfixExpression>() {
                p.base_expression()
            } else {
                None
            };
            parts.push_front(n);
        }

        parts.into()
    }

    /// Generates the [GroupingInfo]s for the parts: `a.b[2].c.d()` is emitted as `{{a.b}[2]}.{c.d}()`.
    /// Field accesses group with the start of the chain until the first part that shouldn't;
    /// array accesses and postfixes group with the last named part and always close. `None` where
    /// upstream's `checkNotNull` throws.
    fn compute_grouping_info(&self, parts: &[KtExpression], use_block_like_lambda_style: bool) -> Option<Vec<GroupingInfo>> {
        let mut grouping_infos = vec![GroupingInfo::default(); parts.len()];
        let mut last_index_to_open = 0;
        for (index, part) in parts.iter().enumerate() {
            if let Some(qualified) = part.cast::<KtQualifiedExpression>() {
                let receiver_expression = qualified.receiver_expression()?;
                let previous = receiver_expression
                    .cast::<KtQualifiedExpression>()
                    .and_then(|r| r.selector_expression())
                    .unwrap_or(receiver_expression);
                let current = qualified.selector_expression()?;
                if last_index_to_open == 0 && self.should_group_part_with_previous(parts, part, index, &previous, &current) {
                    // this and the previous items should be grouped for better style
                    grouping_infos[0].group_open_count += 1;
                    // we don't always close a group when emitting this node, so mark it
                    grouping_infos[index].should_close_group = true;
                } else {
                    // open future groups for arrays and postfixes here; stop grouping field access
                    last_index_to_open = index;
                }
            } else if part.is::<KtArrayAccessExpression>() || part.is::<KtPostfixExpression>() {
                // we group these with the last item with a name, and we always close them
                grouping_infos[last_index_to_open].group_open_count += 1;
            }
        }
        if use_block_like_lambda_style {
            // a trailing lambda adds a group that we stop before emitting the lambda
            grouping_infos[0].group_open_count += 1;
        }
        Some(grouping_infos)
    }

    /// Decide whether a [KtQualifiedExpression] part should be grouped with the previous part.
    fn should_group_part_with_previous(
        &self,
        parts: &[KtExpression],
        part: &KtExpression,
        index: usize,
        previous: &KtExpression,
        current: &KtExpression,
    ) -> bool {
        // this is the second, and the first is short, avoid `.` "hanging in air"
        if index == 1 && text_shorter_than(previous, self.options.continuation_indent) {
            return true;
        }
        // the previous part is `this` or `super`
        if previous.is::<KtSuperExpression>() || previous.is::<KtThisExpression>() {
            return true;
        }
        // this and the previous part are a package name, type name, or property
        if previous.is::<KtSimpleNameExpression>()
            && current.is::<KtSimpleNameExpression>()
            && part.is::<KtDotQualifiedExpression>()
        {
            return true;
        }
        // this is `Foo` in `com.facebook.Foo`, so everything before it is a package name
        if first_unit_is_upper_case(current)
            && current.is::<KtSimpleNameExpression>()
            && part.is::<KtDotQualifiedExpression>()
        {
            return true;
        }
        // this is the `foo()` in `com.facebook.Foo.foo()` or in `Foo.foo()`
        if current.is::<KtCallExpression>() && !previous.is::<KtCallExpression>() && first_unit_is_upper_case(previous) {
            return true;
        }
        // an invocation as the last item after a non-call, i.e. `a.b.c()`: keep `b.c` together
        current.is::<KtCallExpression>() && !previous.is::<KtCallExpression>() && index == parts.len() - 1
    }

    pub(super) fn visit_call_expression(&mut self, call_expression: &KtCallExpression) {
        self.sync(call_expression);
        self.visit_call_element(
            call_expression.callee_expression().as_ref(),
            call_expression.type_argument_list().as_ref(),
            call_expression.value_argument_list().as_ref(),
            &call_expression.lambda_arguments(),
            self.expression_break_indent(),
            Indent::ZERO,
            Indent::ZERO,
        );
    }
}
