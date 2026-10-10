//! `DeclarationFormatter.kt` (classes, constructors, initializers), `ExpressionFormatter.kt` (constants,
//! parentheses), `FileFormatter.kt` (package and import directives), `ListFormatter.kt` (import and
//! context receiver lists).

use ktrs_psi::*;
use ktrs_syntax::SyntaxKind;

use crate::doc::{FillMode, Indent};

use super::KotlinInputAstVisitor;
use super::comma_separated::{EachCommaSeparated, psi_list};
use super::function_like::ParameterList;

impl KotlinInputAstVisitor<'_, '_, '_> {
    pub(super) fn visit_class_or_object(&mut self, class_or_object: &KtClassOrObject) {
        self.sync(class_or_object);
        let context_receiver_list = class_or_object
            .get_stub_or_psi_child::<PsiElement>(SyntaxKind::CONTEXT_PARAMETER_LIST)
            .and_then(|e| e.cast::<KtContextReceiverList>());
        let modifier_list = class_or_object.modifier_list();
        self.block(Indent::ZERO, |v| {
            if let Some(context_receiver_list) = &context_receiver_list {
                v.visit_context_receiver_list(context_receiver_list);
                v.builder.forced_break();
            }
            if let Some(modifier_list) = &modifier_list {
                v.visit_modifier_list(modifier_list);
            }
            if let Some(declaration_keyword) = class_or_object.declaration_keyword() {
                v.token(declaration_keyword.text_slice());
            }
            if let Some(name) = class_or_object.name_identifier() {
                v.builder.space();
                v.token(name.text_slice());
                v.visit(class_or_object.type_parameter_list().as_ref());
            }
            v.visit(class_or_object.primary_constructor().as_ref());
            let super_types = class_or_object.super_type_list();
            if let Some(super_types) = &super_types {
                v.builder.space();
                v.block(Indent::ZERO, |v| {
                    v.token(":");
                    v.builder.break_op(FillMode::Unified, " ", v.expression_break_indent());
                    v.visit(Some(super_types));
                });
            }
            if let Some(type_constraint_list) = class_or_object.type_constraint_list() {
                if super_types
                    .and_then(|s| s.entries().last().cloned())
                    .is_some_and(|e| e.is::<KtDelegatedSuperTypeEntry>())
                {
                    v.builder.forced_break_indent(v.expression_break_indent());
                }
                v.visit(Some(&type_constraint_list));
                v.builder.space();
            } else if class_or_object.body().is_some() {
                v.builder.space();
            }
            v.visit(class_or_object.body().as_ref());
        });
        if class_or_object.name_identifier().is_some() {
            self.builder.forced_break();
        }
    }

    pub(super) fn visit_primary_constructor(&mut self, constructor: &KtPrimaryConstructor) {
        self.sync(constructor);
        let parameter_list = constructor.value_parameter_list().map(|l| ParameterList::of(&l));
        self.block(Indent::ZERO, |v| {
            if constructor.has_constructor_keyword() {
                v.builder.break_op(FillMode::Unified, " ", Indent::ZERO);
            }
            v.visit_function_like_expression(
                None,
                constructor.modifier_list().as_ref(),
                if constructor.has_constructor_keyword() { Some("constructor") } else { None },
                None,
                None,
                None,
                parameter_list.as_ref(),
                None,
                constructor.body_expression().as_ref(),
                None,
            );
        });
    }

    /// Example `private constructor(n: Int) : this(4, 5) { ... }` inside a class's body
    pub(super) fn visit_secondary_constructor(&mut self, constructor: &KtSecondaryConstructor) {
        self.sync(constructor);
        let Some(delegation_call) = constructor.delegation_call() else { return self.fail() };
        let context_receiver_list = constructor
            .get_stub_or_psi_child::<PsiElement>(SyntaxKind::CONTEXT_PARAMETER_LIST)
            .and_then(|e| e.cast::<KtContextReceiverList>());
        let parameter_list = constructor.value_parameter_list().map(|l| ParameterList::of(&l));
        let type_or_delegation_call = (!delegation_call.is_implicit()).then(|| PsiElement::from(delegation_call));
        self.block(Indent::ZERO, |v| {
            v.visit_function_like_expression(
                context_receiver_list.as_ref(),
                constructor.modifier_list().as_ref(),
                Some("constructor"),
                None,
                None,
                None,
                parameter_list.as_ref(),
                None,
                constructor.body_expression().as_ref(),
                type_or_delegation_call.as_ref(),
            );
        });
    }

    /// `call.calleeExpression.accept` reaches `visitElement` instead of `visitReferenceExpression`,
    /// so the keyword is emitted here.
    pub(super) fn visit_constructor_delegation_call(&mut self, call: &KtConstructorDelegationCall) {
        self.block(Indent::ZERO, |v| {
            v.token(if call.is_call_to_this() { "this" } else { "super" });
            v.visit_call_element(
                None,
                call.type_argument_list().as_ref(),
                call.value_argument_list().as_ref(),
                &call.lambda_arguments(),
                v.expression_break_indent(),
                Indent::ZERO,
                Indent::ZERO,
            );
        });
    }

    pub(super) fn visit_class_initializer(&mut self, initializer: &KtClassInitializer) {
        self.sync(initializer);
        self.token("init");
        self.builder.space();
        self.visit(initializer.body().as_ref());
    }

    pub(super) fn visit_constant_expression(&mut self, expression: &KtConstantExpression) {
        self.sync(expression);
        self.token(expression.text_slice());
    }

    /// Example `(1 + 1)`
    pub(super) fn visit_parenthesized_expression(&mut self, expression: &KtParenthesizedExpression) {
        self.sync(expression);
        self.token("(");
        self.visit(expression.expression().as_ref());
        self.token(")");
    }

    pub(super) fn visit_package_directive(&mut self, directive: &KtPackageDirective) {
        self.sync(directive);
        if directive.package_keyword().is_none() {
            return;
        }
        self.token("package");
        self.builder.space();
        let mut first = true;
        for package_name in directive.package_names() {
            if first {
                first = false;
            } else {
                self.token(".");
            }
            let text = package_name.identifier().map_or_else(|| package_name.referenced_name(), |i| i.text());
            self.token(&text);
        }

        self.builder.guess_token(";");
        self.builder.forced_break();
    }

    /// Example `import com.foo.A; import com.bar.B`
    pub(super) fn visit_import_list(&mut self, import_list: &KtImportList) {
        self.sync(import_list);
        for import in import_list.imports() {
            self.visit(Some(&import));
        }
    }

    /// Example `import com.foo.A`
    pub(super) fn visit_import_directive(&mut self, directive: &KtImportDirective) {
        self.sync(directive);
        self.token("import");
        self.builder.space();

        if let Some(imported_reference) = directive.imported_reference() {
            self.in_import = true;
            self.visit(Some(&imported_reference));
            self.in_import = false;
        }
        if directive.is_all_under() {
            self.token(".");
            self.token("*");
        }

        // Possible alias.
        if let Some(alias) = directive.alias().and_then(|a| a.name_identifier()) {
            self.builder.space();
            self.token("as");
            self.builder.space();
            self.token(alias.text_slice());
        }

        // Force a newline afterwards.
        self.builder.guess_token(";");
        self.builder.forced_break();
    }

    /// Example `context(logger: Logger, raise: Raise<Error>)`, or the legacy receiver form
    /// `context(Logger, Raise<Error>)` (still used by function types).
    pub(super) fn visit_context_receiver_list(&mut self, context_receiver_list: &KtContextReceiverList) {
        self.sync(context_receiver_list);
        self.token("context");
        self.visit_each_comma_separated(
            &list_to_visit(context_receiver_list),
            EachCommaSeparated {
                prefix: Some("("),
                postfix: Some(")"),
                break_after_prefix: false,
                break_before_postfix: false,
                ..self.comma_separated()
            },
        );
    }
}

/// ktfmt's `CompatibilityUtils.listToVisit` (kotlin-2.3): context parameters, else receivers.
fn list_to_visit(list: &KtContextReceiverList) -> Vec<PsiElement> {
    let parameters = list.context_parameters();
    if parameters.is_empty() { psi_list(list.context_receivers()) } else { psi_list(parameters) }
}
