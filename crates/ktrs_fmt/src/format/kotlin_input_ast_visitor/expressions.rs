//! `KotlinInputAstVisitor.kt` lines 2394-2513: destructuring declarations, string templates, `super`,
//! type parameters and constraints.

use ktrs_psi::*;

use crate::doc::{FillMode, Indent};
use crate::format::input::whitespace_tombstones::replace_trailing_whitespace_with_tombstone;

use super::KotlinInputAstVisitor;
use super::comma_separated::{EachCommaSeparated, psi_list};
use super::declarations::DeclarationKind;

impl KotlinInputAstVisitor<'_, '_, '_> {
    /// Example `val (a, b: Int) = Pair(1, 2)` or `val [a, b] = Pair(1, 2)`
    pub(super) fn visit_destructuring_declaration(&mut self, destructuring_declaration: &KtDestructuringDeclaration) {
        self.sync(destructuring_declaration);
        if let Some(val_or_var_keyword) = destructuring_declaration.val_or_var_keyword() {
            self.token(val_or_var_keyword.text_slice());
            self.builder.space();
        }
        let has_trailing_comma = destructuring_declaration.trailing_comma().is_some();
        let opening_delimiter = destructuring_declaration.l_par().map_or_else(|| "(".to_owned(), |p| p.text());
        let closing_delimiter = destructuring_declaration.r_par().map_or_else(|| ")".to_owned(), |p| p.text());
        let ebi = self.expression_break_indent();
        self.block(Indent::ZERO, |v| {
            v.token(&opening_delimiter);
            v.builder.break_op(FillMode::Unified, "", ebi.clone());
            v.block(ebi.clone(), |v| {
                v.visit_each_comma_separated(
                    &psi_list(destructuring_declaration.entries()),
                    EachCommaSeparated { has_trailing_comma, wrap_in_block: true, ..v.comma_separated() },
                );
            });
        });
        self.token(&closing_delimiter);
        if let Some(initializer) = destructuring_declaration.initializer() {
            self.builder.space();
            self.token("=");
            if has_trailing_comma {
                self.builder.space();
            } else {
                self.builder.break_op(FillMode::Independent, " ", ebi.clone());
            }
            self.block_if(ebi, !has_trailing_comma, |v| v.visit(Some(&initializer)));
        }
    }

    /// Example `a: String` or `x = a` which is part of `(a: String, x = a)`
    pub(super) fn visit_destructuring_declaration_entry(&mut self, multi_declaration_entry: &KtDestructuringDeclarationEntry) {
        self.sync(multi_declaration_entry);
        let Some(name) = multi_declaration_entry.name_identifier().map(|n| n.text()) else { return self.fail() };
        let initializer = multi_declaration_entry.initializer().map(|i| i.upcast::<KtExpression>());
        self.declare_one(
            DeclarationKind::Parameter,
            multi_declaration_entry.modifier_list().as_ref(),
            None,
            None,
            None,
            Some(&name),
            multi_declaration_entry.type_reference().as_ref(),
            None,
            initializer.as_ref(),
            None,
            None,
            None,
        );
    }

    /// Example `"Hello $world!"` or `"""Hello world!"""`
    pub(super) fn visit_string_template_expression(&mut self, expression: &KtStringTemplateExpression) {
        self.sync(expression);
        self.token(&replace_trailing_whitespace_with_tombstone(&expression.text()));
    }

    /// Example `super` in `super.doIt(5)` or `super<Foo>` in `super<Foo>.doIt(5)`
    pub(super) fn visit_super_expression(&mut self, expression: &KtSuperExpression) {
        self.sync(expression);
        self.token("super");
        if let Some(super_type_qualifier) = expression.super_type_qualifier() {
            self.token("<");
            self.visit(Some(&super_type_qualifier));
            self.token(">");
        }
        self.visit(expression.label_qualifier().as_ref());
    }

    /// Example `<T, S>`
    pub(super) fn visit_type_parameter_list(&mut self, list: &KtTypeParameterList) {
        self.sync(list);
        self.block(self.expression_break_indent(), |v| {
            v.visit_each_comma_separated(
                &psi_list(list.parameters()),
                EachCommaSeparated {
                    has_trailing_comma: list.trailing_comma().is_some(),
                    prefix: Some("<"),
                    postfix: Some(">"),
                    wrap_in_block: !v.options.manage_trailing_commas(),
                    ..v.comma_separated()
                },
            );
        });
    }

    pub(super) fn visit_type_parameter(&mut self, parameter: &KtTypeParameter) {
        self.sync(parameter);
        self.visit(parameter.modifier_list().as_ref());
        self.token(&parameter.name_identifier().map(|n| n.text()).unwrap_or_default());
        if let Some(extends_bound) = parameter.extends_bound() {
            self.builder.space();
            self.token(":");
            self.builder.space();
            self.visit(Some(&extends_bound));
        }
    }

    /// Example `where T : View, T : Listener`
    pub(super) fn visit_type_constraint_list(&mut self, list: &KtTypeConstraintList) {
        let ebi = self.expression_break_indent();
        self.block(ebi.clone(), |v| {
            v.builder.break_op(FillMode::Independent, " ", Indent::ZERO);
            v.token("where");
            v.block(ebi, |v| {
                v.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
                v.sync(list);
                v.visit_each_comma_separated(
                    &psi_list(list.constraints()),
                    EachCommaSeparated { wrap_in_block: false, ..v.comma_separated() },
                );
            });
        });
    }

    /// Example `T : Foo`
    pub(super) fn visit_type_constraint(&mut self, constraint: &KtTypeConstraint) {
        self.sync(constraint);
        // TODO(nreid260): What about annotations on the type reference? `where @A T : Int`
        self.visit(constraint.subject_type_parameter_name().as_ref());
        self.builder.space();
        self.token(":");
        self.builder.space();
        self.visit(constraint.bound_type_reference().as_ref());
    }
}
