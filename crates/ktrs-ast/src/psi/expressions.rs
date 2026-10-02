//! Expression classes beyond calls: blocks, `if`/`when`/loops, binary operators, `return`, constants and string
//! templates.

use ktrs_syntax::SyntaxKind::{self, *};

use super::classes::*;
use crate::arena::{Ast, NodeId};

fn first_expression_child(ast: &Ast, n: NodeId) -> Option<NodeId> {
    ast.children(n).find(|&c| KtExpression::is(ast, c))
}

/// `findExpressionUnder(type)`: the expression inside the `type` container child (`THEN`, `ELSE`, `BODY`, ...).
fn find_expression_under(ast: &Ast, n: NodeId, container: SyntaxKind) -> Option<NodeId> {
    first_expression_child(ast, ast.find_child_by_type(n, container)?)
}

impl KtBlockExpression {
    /// `getStatements()`: the expression children (declarations included).
    pub fn statements(self, ast: &Ast) -> Vec<NodeId> {
        ast.children(self.0).filter(|&c| KtExpression::is(ast, c)).collect()
    }
}

impl KtIfExpression {
    pub fn condition(self, ast: &Ast) -> Option<NodeId> {
        find_expression_under(ast, self.0, CONDITION)
    }

    pub fn then(self, ast: &Ast) -> Option<NodeId> {
        find_expression_under(ast, self.0, THEN)
    }

    /// `getElse()` (Kotlin `` `else` ``).
    pub fn r#else(self, ast: &Ast) -> Option<NodeId> {
        find_expression_under(ast, self.0, ELSE)
    }
}

impl KtLoopExpression {
    pub fn body(self, ast: &Ast) -> Option<NodeId> {
        find_expression_under(ast, self.0, BODY)
    }
}

impl KtWhenExpression {
    pub fn entries(self, ast: &Ast) -> Vec<KtWhenEntry> {
        ast.children(self.0).filter_map(|c| KtWhenEntry::cast(ast, c)).collect()
    }
}

impl KtWhenEntry {
    /// `getExpression()`: the first expression child (the branch body).
    pub fn expression(self, ast: &Ast) -> Option<NodeId> {
        first_expression_child(ast, self.0)
    }
}

impl KtBinaryExpression {
    pub fn operation_reference(self, ast: &Ast) -> Option<KtSimpleNameExpression> {
        KtSimpleNameExpression::cast(ast, ast.find_child_by_type(self.0, OPERATION_REFERENCE)?)
    }

    /// `getLeft()`: the nearest expression before the operation reference.
    pub fn left(self, ast: &Ast) -> Option<NodeId> {
        ast.siblings(self.operation_reference(ast)?.node(), false).find(|&s| KtExpression::is(ast, s))
    }

    /// `getRight()`: the nearest expression after the operation reference.
    pub fn right(self, ast: &Ast) -> Option<NodeId> {
        ast.siblings(self.operation_reference(ast)?.node(), true).find(|&s| KtExpression::is(ast, s))
    }

    /// `getOperationToken()`: the operation reference's referenced name element type (`ELVIS`, `PLUS`, ...).
    pub fn operation_token(self, ast: &Ast) -> Option<SyntaxKind> {
        Some(ast.element_type(self.operation_reference(ast)?.referenced_name_element(ast)))
    }
}

impl KtReturnExpression {
    /// `getReturnedExpression()`: the expression child.
    pub fn returned_expression(self, ast: &Ast) -> Option<NodeId> {
        first_expression_child(ast, self.0)
    }

    /// `getLabeledExpression()`: the `@label` qualifier (`LABEL_QUALIFIER`), if any.
    pub fn labeled_expression(self, ast: &Ast) -> Option<NodeId> {
        ast.find_child_by_type(self.0, LABEL_QUALIFIER)
    }
}

impl KtStringTemplateExpression {
    /// `getEntries()`: the template entry children.
    pub fn entries(self, ast: &Ast) -> Vec<NodeId> {
        ast.children(self.0)
            .filter(|&c| {
                matches!(
                    ast.element_type(c),
                    LITERAL_STRING_TEMPLATE_ENTRY
                        | SHORT_STRING_TEMPLATE_ENTRY
                        | LONG_STRING_TEMPLATE_ENTRY
                        | ESCAPE_STRING_TEMPLATE_ENTRY
                )
            })
            .collect()
    }
}
