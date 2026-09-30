//! `KotlinInputAstVisitor.kt` lines 2664-2877: `is`/`as`, collection literals, `try`/`catch`/`finally`,
//! `throw`, enum entries, type aliases, `visitElement`, `visitKtFile`, `visitScript`.

use ktrs_psi::*;

use crate::doc::{BlankLineWanted, FillMode, Indent};
use crate::format::kotlin_text::is_kotlin_whitespace;

use super::KotlinInputAstVisitor;
use super::comma_separated::{EachCommaSeparated, psi_list};

impl KotlinInputAstVisitor<'_, '_, '_> {
    /// Example `a is Int` or `b !is Int`
    pub(super) fn visit_is_expression(&mut self, expression: &KtIsExpression) {
        self.sync(expression);
        let ebi = self.expression_break_indent();
        let left_hand_side = expression.left_hand_side();
        let open_group_before_left = !left_hand_side.as_ref().is_some_and(|l| l.is::<KtQualifiedExpression>());
        if open_group_before_left {
            self.builder.open(Indent::ZERO);
        }
        self.visit(left_hand_side.as_ref());
        if !open_group_before_left {
            self.builder.open(Indent::ZERO);
        }
        let parent = expression.parent();
        if parent
            .as_ref()
            .is_some_and(|p| p.is::<KtValueArgument>() || p.is::<KtParenthesizedExpression>() || p.is::<KtContainerNode>())
        {
            self.builder.break_op(FillMode::Unified, " ", ebi.clone());
        } else {
            self.builder.space();
        }
        self.visit(expression.operation_reference().as_ref());
        self.builder.break_op(FillMode::Independent, " ", ebi.clone());
        self.block(ebi, |v| v.visit(expression.type_reference().as_ref()));
        self.builder.close();
    }

    /// Example `a as Int` or `a as? Int`
    pub(super) fn visit_binary_with_type_rhs_expression(&mut self, expression: &KtBinaryExpressionWithTypeRHS) {
        self.sync(expression);
        let ebi = self.expression_break_indent();
        let left = expression.left();
        let open_group_before_left = !left.as_ref().is_some_and(|l| l.is::<KtQualifiedExpression>());
        if open_group_before_left {
            self.builder.open(Indent::ZERO);
        }
        self.visit(left.as_ref());
        if !open_group_before_left {
            self.builder.open(Indent::ZERO);
        }
        self.builder.break_op(FillMode::Unified, " ", ebi.clone());
        self.visit(expression.operation_reference().as_ref());
        self.builder.break_op(FillMode::Independent, " ", ebi.clone());
        self.block(ebi, |v| v.visit(expression.right().as_ref()));
        self.builder.close();
    }

    /// Example: `val a: Array<Int> = [1, 2, 3]`
    pub(super) fn visit_collection_literal_expression(&mut self, expression: &KtCollectionLiteralExpression) {
        self.sync(expression);
        self.block(self.expression_break_indent(), |v| {
            v.visit_each_comma_separated(
                &psi_list(expression.inner_expressions()),
                EachCommaSeparated {
                    has_trailing_comma: expression.trailing_comma().is_some(),
                    prefix: Some("["),
                    postfix: Some("]"),
                    wrap_in_block: !v.options.manage_trailing_commas(),
                    ..v.comma_separated()
                },
            );
        });
    }

    pub(super) fn visit_try_expression(&mut self, expression: &KtTryExpression) {
        self.sync(expression);
        self.token("try");
        self.builder.space();
        self.visit(expression.try_block().as_ref());
        for catch_clause in expression.catch_clauses() {
            self.visit(Some(&catch_clause));
        }
        self.visit(expression.finally_block().as_ref());
    }

    pub(super) fn visit_catch_section(&mut self, catch_clause: &KtCatchClause) {
        self.sync(catch_clause);
        self.builder.space();
        self.token("catch");
        self.builder.space();
        self.block(Indent::ZERO, |v| {
            v.token("(");
            v.block(v.expression_break_indent(), |v| {
                v.builder.break_op(FillMode::Unified, "", Indent::ZERO);
                v.visit(catch_clause.catch_parameter().as_ref());
                v.builder.guess_token(",");
            });
        });
        self.token(")");
        self.builder.space();
        self.visit(catch_clause.catch_body().as_ref());
    }

    pub(super) fn visit_finally_section(&mut self, finally_section: &KtFinallySection) {
        self.sync(finally_section);
        self.builder.space();
        self.token("finally");
        self.builder.space();
        self.visit(finally_section.final_expression().as_ref());
    }

    pub(super) fn visit_throw_expression(&mut self, expression: &KtThrowExpression) {
        self.sync(expression);
        self.token("throw");
        self.builder.space();
        self.visit(expression.thrown_expression().as_ref());
    }

    /// Example `RED(0xFF0000)` in an enum class
    pub(super) fn visit_enum_entry(&mut self, enum_entry: &KtEnumEntry) {
        self.sync(enum_entry);
        self.block(Indent::ZERO, |v| {
            v.visit(enum_entry.modifier_list().as_ref());
            let Some(name) = enum_entry.name_identifier() else { return v.fail() };
            v.token(name.text_slice());
            for initializer in enum_entry.initializer_list().map(|l| l.initializers()).unwrap_or_default() {
                v.visit(Some(&initializer));
            }
            if let Some(enum_body) = enum_entry.body() {
                v.builder.space();
                v.visit(Some(&enum_body));
            }
        });
    }

    /// Example `private typealias TextChangedListener = (string: String) -> Unit`
    pub(super) fn visit_type_alias(&mut self, type_alias: &KtTypeAlias) {
        self.sync(type_alias);
        self.block(Indent::ZERO, |v| {
            v.visit(type_alias.modifier_list().as_ref());
            v.token("typealias");
            v.builder.space();
            let Some(name) = type_alias.name_identifier() else { return v.fail() };
            v.token(name.text_slice());
            v.visit(type_alias.type_parameter_list().as_ref());

            v.builder.space();
            v.token("=");
            v.builder.break_op(FillMode::Independent, " ", v.expression_break_indent());
            v.block(v.expression_break_indent(), |v| {
                v.visit(type_alias.type_reference().as_ref());
                v.visit(type_alias.type_constraint_list().as_ref());
                v.builder.guess_token(";");
            });
            v.builder.forced_break();
        });
    }

    /// Called for almost all AST nodes; tracks whether we're inside an expression and checks that
    /// every level opened below was closed.
    pub(super) fn visit_element(&mut self, element: &PsiElement) {
        let in_expression = element.is::<KtExpression>() || *self.in_expression.last().unwrap();
        self.in_expression.push(in_expression);
        let previous = self.builder.depth();
        kt_tree_visitor_void::visit_element(self, element);
        self.in_expression.pop();
        self.builder.check_closed(previous);
    }

    pub(super) fn visit_kt_file(&mut self, file: &KtFile) {
        self.mark_for_partial_format();
        let import_list_empty = file.import_list().is_none_or(|l| l.text_all(is_kotlin_whitespace));

        let mut is_first = true;
        for child in file.children() {
            if child.text_all(is_kotlin_whitespace) {
                continue;
            }

            let blank_line_wanted = if is_first {
                BlankLineWanted::NO
            } else if child.is::<PsiComment>() {
                continue;
            } else if child.is::<KtScript>() && import_list_empty {
                BlankLineWanted::PRESERVE
            } else {
                BlankLineWanted::YES
            };
            self.builder.blank_line_wanted(blank_line_wanted);

            self.visit(Some(&child));
            is_first = false;
        }
        self.mark_for_partial_format();
    }

    pub(super) fn visit_script(&mut self, script: &KtScript) {
        self.mark_for_partial_format();
        let mut last_child_had_blank_line_before = false;
        let mut last_child_is_context_receiver = false;
        let mut first = true;
        let Some(block_expression) = script.block_expression() else { return self.fail() };
        for child in block_expression.children() {
            if child.text_all(is_kotlin_whitespace) {
                continue;
            }
            self.builder.forced_break();
            let child_gets_blank_line_before = !child.is::<KtProperty>();
            if first {
                self.builder.blank_line_wanted(BlankLineWanted::PRESERVE);
            } else if last_child_is_context_receiver {
                self.builder.blank_line_wanted(BlankLineWanted::NO);
            } else if !child.is::<PsiComment>() && (child_gets_blank_line_before || last_child_had_blank_line_before) {
                self.builder.blank_line_wanted(BlankLineWanted::YES);
            }
            self.visit(Some(&child));
            self.builder.guess_token(";");
            last_child_had_blank_line_before = child_gets_blank_line_before;
            last_child_is_context_receiver = child.is::<KtScriptInitializer>()
                && child
                    .first_child()
                    .and_then(|c| c.first_child())
                    .and_then(|c| c.first_child())
                    .is_some_and(|c| c.text() == "context");
            first = false;
        }
        self.mark_for_partial_format();
    }
}
