//! Calls and their arguments: `KtCallExpression`, `KtValueArgumentList`, `KtValueArgument`, `KtLambdaArgument`,
//! `KtValueArgumentName`, `KtLambdaExpression`, `KtFunctionLiteral`, `KtSafeQualifiedExpression`,
//! `KtNameReferenceExpression`.

use ktrs_syntax::SyntaxKind::*;

use super::classes::*;
use crate::arena::{Ast, NodeId};

impl KtCallExpression {
    /// `getCalleeExpression()`: the reference expression child, else the first expression child (2.2.21 takes the
    /// first expression child directly; a call never has a reference expression after another expression).
    pub fn callee_expression(self, ast: &Ast) -> Option<NodeId> {
        ast.find_child_by_type(self.0, REFERENCE_EXPRESSION).or_else(|| ast.children(self.0).find(|&c| KtExpression::is(ast, c)))
    }

    pub fn value_argument_list(self, ast: &Ast) -> Option<KtValueArgumentList> {
        KtValueArgumentList::cast(ast, ast.find_child_by_type(self.0, VALUE_ARGUMENT_LIST)?)
    }

    pub fn lambda_arguments(self, ast: &Ast) -> Vec<KtLambdaArgument> {
        ast.children(self.0).filter_map(|c| KtLambdaArgument::cast(ast, c)).collect()
    }

    /// `getValueArguments()`: the parenthesized arguments, then the lambda arguments.
    pub fn value_arguments(self, ast: &Ast) -> Vec<KtValueArgument> {
        let mut arguments = self.value_argument_list(ast).map(|l| l.arguments(ast)).unwrap_or_default();
        arguments.extend(self.lambda_arguments(ast).into_iter().map(|a| KtValueArgument(a.0)));
        arguments
    }

    /// `getTypeArguments()`: the `TYPE_PROJECTION`s (`KtTypeProjection`) of the type argument list.
    pub fn type_arguments(self, ast: &Ast) -> Vec<NodeId> {
        let Some(list) = ast.find_child_by_type(self.0, TYPE_ARGUMENT_LIST) else { return Vec::new() };
        ast.children(list).filter(|&c| ast.element_type(c) == TYPE_PROJECTION).collect()
    }
}

impl KtValueArgumentList {
    pub fn arguments(self, ast: &Ast) -> Vec<KtValueArgument> {
        ast.children(self.0).filter(|&c| ast.element_type(c) == VALUE_ARGUMENT).map(KtValueArgument).collect()
    }
}

macro_rules! value_argument {
    ($($t:ident),*) => {$(impl $t {
        /// `getArgumentExpression()`: the first expression child.
        pub fn argument_expression(self, ast: &Ast) -> Option<NodeId> {
            ast.children(self.0).find(|&c| KtExpression::is(ast, c))
        }

        pub fn argument_name(self, ast: &Ast) -> Option<KtValueArgumentName> {
            KtValueArgumentName::cast(ast, ast.find_child_by_type(self.0, VALUE_ARGUMENT_NAME)?)
        }

        pub fn is_named(self, ast: &Ast) -> bool {
            self.argument_name(ast).is_some()
        }

        /// `getName()`: `PsiElementBase.getName()`, always null (`KtValueArgument` is not a named element).
        pub fn name(self, _ast: &Ast) -> Option<String> {
            None
        }
    })*};
}

value_argument!(KtValueArgument, KtLambdaArgument);

impl KtLambdaArgument {
    /// `getLambdaExpression()`: `getArgumentExpression()?.unpackFunctionLiteral()`.
    pub fn lambda_expression(self, ast: &Ast) -> Option<KtLambdaExpression> {
        unpack_function_literal(ast, self.argument_expression(ast)?)
    }
}

/// psiUtil `KtExpression.unpackFunctionLiteral(allowParentheses = false)`.
pub fn unpack_function_literal(ast: &Ast, expression: NodeId) -> Option<KtLambdaExpression> {
    match ast.element_type(expression) {
        LAMBDA_EXPRESSION => KtLambdaExpression::cast(ast, expression),
        LABELED_EXPRESSION | ANNOTATED_EXPRESSION => {
            let base = ast.children(expression).find(|&c| KtExpression::is(ast, c))?;
            unpack_function_literal(ast, base)
        }
        _ => None,
    }
}

impl KtValueArgumentName {
    /// `getReferenceExpression()`.
    pub fn reference_expression(self, ast: &Ast) -> Option<KtSimpleNameExpression> {
        KtSimpleNameExpression::cast(ast, ast.find_child_by_type(self.0, REFERENCE_EXPRESSION)?)
    }

    /// `getAsName().asString()`: the referenced name.
    pub fn as_name(self, ast: &Ast) -> Option<String> {
        Some(self.reference_expression(ast)?.referenced_name(ast))
    }
}

impl KtLambdaExpression {
    pub fn function_literal(self, ast: &Ast) -> Option<KtFunctionLiteral> {
        KtFunctionLiteral::cast(ast, ast.find_child_by_type(self.0, FUNCTION_LITERAL)?)
    }

    /// `getBodyExpression()`: the function literal's block.
    pub fn body_expression(self, ast: &Ast) -> Option<KtBlockExpression> {
        self.function_literal(ast)?.body_block_expression(ast)
    }

    pub fn value_parameters(self, ast: &Ast) -> Vec<KtParameter> {
        self.function_literal(ast).map(|f| f.value_parameters(ast)).unwrap_or_default()
    }
}

impl KtSafeQualifiedExpression {
    pub fn receiver_expression(self, ast: &Ast) -> Option<NodeId> {
        KtQualifiedExpression(self.0).receiver_expression(ast)
    }

    pub fn selector_expression(self, ast: &Ast) -> Option<NodeId> {
        KtQualifiedExpression(self.0).selector_expression(ast)
    }
}

impl KtNameReferenceExpression {
    /// `getReferencedName()`.
    pub fn referenced_name(self, ast: &Ast) -> String {
        KtSimpleNameExpression(self.0).referenced_name(ast)
    }
}
