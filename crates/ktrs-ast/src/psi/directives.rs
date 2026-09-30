//! `KtImportDirective`, `KtImportAlias`, `KtPackageDirective` and the name-expression accessors behind their
//! fq names (`KtQualifiedExpression`, `KtSimpleNameExpression`). Same logic as `ktrs_psi::kt::file`.

use ktrs_psi::classes::is_expression;
use ktrs_psi::{INSIDE_DIRECTIVE_EXPRESSIONS, NAME_REFERENCE_EXPRESSIONS, OPERATION_TOKENS, OPERATIONS};
use ktrs_psi::{FqName, ImportPath, unquote_identifier_or_field_reference};
use ktrs_parser::token_set::TokenSet;
use ktrs_syntax::SyntaxKind::*;

use super::classes::{KtImportAlias, KtImportDirective, KtPackageDirective, KtQualifiedExpression, KtSimpleNameExpression};
use crate::arena::{Ast, NodeId};

/// `getStubOrPsiChildren(set).firstOrNull()` on the AST path.
fn first_child_in(ast: &Ast, n: NodeId, set: TokenSet) -> Option<NodeId> {
    ast.children(n).find(|&c| set.contains(ast.element_type(c)))
}

impl KtImportDirective {
    /// `getImportedReference()`.
    pub fn imported_reference(self, ast: &Ast) -> Option<NodeId> {
        first_child_in(ast, self.node(), INSIDE_DIRECTIVE_EXPRESSIONS)
    }

    pub fn alias(self, ast: &Ast) -> Option<KtImportAlias> {
        KtImportAlias::cast(ast, ast.find_child_by_type(self.node(), IMPORT_ALIAS)?)
    }

    pub fn alias_name(self, ast: &Ast) -> Option<String> {
        self.alias(ast)?.name(ast)
    }

    pub fn is_all_under(self, ast: &Ast) -> bool {
        ast.find_child_by_type(self.node(), MUL).is_some()
    }

    /// `getImportedFqName()`. Upstream caches it on the PSI object; recomputed here (rules don't edit imports
    /// before reading them).
    pub fn imported_fq_name(self, ast: &Ast) -> Option<FqName> {
        fq_name_from_expression(ast, self.imported_reference(ast))
    }

    pub fn import_path(self, ast: &Ast) -> Option<ImportPath> {
        let fq_name = self.imported_fq_name(ast)?;
        Some(ImportPath { fq_name, is_all_under: self.is_all_under(ast), alias: self.alias_name(ast) })
    }
}

/// `KtImportDirective.fqNameFromExpression`.
fn fq_name_from_expression(ast: &Ast, expression: Option<NodeId>) -> Option<FqName> {
    let expression = expression?;
    if ast.element_type(expression) == DOT_QUALIFIED_EXPRESSION {
        let dot = KtQualifiedExpression::of(ast, expression);
        let parent_fqn = fq_name_from_expression(ast, dot.receiver_expression(ast));
        let Some(child) = name_from_expression(ast, dot.selector_expression(ast)) else { return parent_fqn };
        return Some(parent_fqn?.child(&child));
    }
    Some(FqName::root().child(&KtSimpleNameExpression::cast(ast, expression)?.referenced_name(ast)))
}

fn name_from_expression(ast: &Ast, expression: Option<NodeId>) -> Option<String> {
    Some(KtSimpleNameExpression::cast(ast, expression?)?.referenced_name(ast))
}

impl KtImportAlias {
    /// `getName()`: the alias identifier's text, backticks kept.
    pub fn name(self, ast: &Ast) -> Option<String> {
        Some(ast.text(ast.find_child_by_type(self.node(), IDENTIFIER)?))
    }
}

impl KtPackageDirective {
    pub fn package_name_expression(self, ast: &Ast) -> Option<NodeId> {
        first_child_in(ast, self.node(), INSIDE_DIRECTIVE_EXPRESSIONS)
    }

    /// `getPackageNames()`: the simple names of the package expression, outermost first.
    pub fn package_names(self, ast: &Ast) -> Vec<KtSimpleNameExpression> {
        let mut package_names = Vec::new();
        let mut name_expression = self.package_name_expression(ast);
        while let Some(qualified) = name_expression.and_then(|e| KtQualifiedExpression::cast(ast, e)) {
            if let Some(selector) = qualified.selector_expression(ast).and_then(|s| KtSimpleNameExpression::cast(ast, s)) {
                package_names.push(selector);
            }
            name_expression = qualified.receiver_expression(ast);
        }
        if let Some(simple) = name_expression.and_then(|e| KtSimpleNameExpression::cast(ast, e)) {
            package_names.push(simple);
        }
        package_names.reverse();
        package_names
    }

    pub fn qualified_name(self, ast: &Ast) -> String {
        self.package_names(ast).iter().map(|e| e.referenced_name(ast)).collect::<Vec<_>>().join(".")
    }
}

impl KtQualifiedExpression {
    pub fn receiver_expression(self, ast: &Ast) -> Option<NodeId> {
        self.get_expression(ast, false)
    }

    pub fn selector_expression(self, ast: &Ast) -> Option<NodeId> {
        self.get_expression(ast, true)
    }

    /// `operationTokenNode.psi.siblings(afterOperation, false).firstIsInstanceOrNull<KtExpression>()`.
    fn get_expression(self, ast: &Ast, after_operation: bool) -> Option<NodeId> {
        let operation = first_child_in(ast, self.node(), OPERATIONS)?;
        ast.siblings(operation, after_operation)
            .find(|&s| !ast.is_leaf_element(s) && is_expression(ast.element_type(s)))
    }
}

impl KtSimpleNameExpression {
    /// `getReferencedNameElement()`. The enum-superclass reference (never inside a directive) resolves to
    /// the node itself here.
    pub fn referenced_name_element(self, ast: &Ast) -> NodeId {
        let n = self.node();
        let found = match ast.element_type(n) {
            REFERENCE_EXPRESSION => first_child_in(ast, n, NAME_REFERENCE_EXPRESSIONS),
            OPERATION_REFERENCE => first_child_in(ast, n, OPERATION_TOKENS),
            LABEL => ast.find_child_by_type(n, IDENTIFIER),
            _ => None,
        };
        found.unwrap_or(n)
    }

    /// `getReferencedName()`.
    pub fn referenced_name(self, ast: &Ast) -> String {
        unquote_identifier_or_field_reference(&ast.text(self.referenced_name_element(ast)))
    }
}
