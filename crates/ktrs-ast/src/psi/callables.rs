//! `KtCallableDeclaration` / `KtDeclarationWithBody` interface methods (stamped on every implementing class) and
//! `KtParameter`, `KtProperty`, `KtNamedFunction`, `KtParameterList`, `KtDestructuringDeclaration`.

use ktrs_syntax::SyntaxKind::*;

use super::annotated::modifier_list;
use super::classes::*;
use crate::arena::{Ast, NodeId};

/// TypeRefHelpers.kt `getTypeReference(declaration)`: the first type reference after the first `:`.
fn type_ref_helpers_get_type_reference(ast: &Ast, declaration: NodeId) -> Option<KtTypeReference> {
    ast.children(declaration).skip_while(|&c| ast.element_type(c) != COLON).find_map(|c| KtTypeReference::cast(ast, c))
}

/// `KtCallableDeclaration.getTypeReference()`, dispatched to the implementing class.
fn type_reference(ast: &Ast, callable: NodeId) -> Option<KtTypeReference> {
    match ast.element_type(callable) {
        FUN | PROPERTY | DESTRUCTURING_DECLARATION_ENTRY | FUNCTION_LITERAL => type_ref_helpers_get_type_reference(ast, callable),
        VALUE_PARAMETER | PROPERTY_ACCESSOR => KtTypeReference::cast(ast, ast.find_child_by_type(callable, TYPE_REFERENCE)?),
        _ => None,
    }
}

/// `getReceiverTypeReference()`: `KtNamedFunction.receiverTypeRefByTree` (before `(` or `:`),
/// `KtProperty.getReceiverTypeRefByTree` (before `:`); none for the other callables.
fn receiver_type_reference(ast: &Ast, callable: NodeId) -> Option<KtTypeReference> {
    let stops = match ast.element_type(callable) {
        FUN => &[LPAR, COLON][..],
        PROPERTY => &[COLON][..],
        _ => return None,
    };
    ast.children(callable).take_while(|&c| !stops.contains(&ast.element_type(c))).find_map(|c| KtTypeReference::cast(ast, c))
}

fn value_parameter_list(ast: &Ast, callable: NodeId) -> Option<KtParameterList> {
    match ast.element_type(callable) {
        PROPERTY | VALUE_PARAMETER | DESTRUCTURING_DECLARATION_ENTRY => None,
        _ => KtParameterList::cast(ast, ast.find_child_by_type(callable, VALUE_PARAMETER_LIST)?),
    }
}

/// `getValueParameters()`: a property accessor's is its setter parameter.
fn value_parameters(ast: &Ast, callable: NodeId) -> Vec<KtParameter> {
    match ast.element_type(callable) {
        FUN | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR | FUNCTION_LITERAL | PROPERTY_ACCESSOR => {
            value_parameter_list(ast, callable).map(|l| l.parameters(ast)).unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

/// `getBodyExpression()`: the first expression child (none for a primary constructor, the block for a
/// secondary one).
fn body_expression(ast: &Ast, declaration: NodeId) -> Option<NodeId> {
    match ast.element_type(declaration) {
        PRIMARY_CONSTRUCTOR => None,
        SECONDARY_CONSTRUCTOR => ast.find_child_by_type(declaration, BLOCK),
        _ => ast.children(declaration).find(|&c| KtExpression::is(ast, c)),
    }
}

/// `hasBlockBody()`: no `=` for functions and accessors, never for a function literal, `hasBody()` for constructors.
fn has_block_body(ast: &Ast, declaration: NodeId) -> bool {
    match ast.element_type(declaration) {
        FUNCTION_LITERAL => false,
        PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR => body_expression(ast, declaration).is_some(),
        _ => ast.find_child_by_type(declaration, EQ).is_none(),
    }
}

macro_rules! callable {
    ($($t:ident),*) => {$(impl $t {
        /// `getTypeReference()` (the declared return type of a function).
        pub fn type_reference(self, ast: &Ast) -> Option<KtTypeReference> {
            type_reference(ast, self.0)
        }

        /// `hasDeclaredReturnType()` for functions: `typeReference != null`.
        pub fn has_declared_return_type(self, ast: &Ast) -> bool {
            self.type_reference(ast).is_some()
        }

        pub fn receiver_type_reference(self, ast: &Ast) -> Option<KtTypeReference> {
            receiver_type_reference(ast, self.0)
        }

        pub fn value_parameter_list(self, ast: &Ast) -> Option<KtParameterList> {
            value_parameter_list(ast, self.0)
        }

        pub fn value_parameters(self, ast: &Ast) -> Vec<KtParameter> {
            value_parameters(ast, self.0)
        }
    })*};
}

callable!(
    KtCallableDeclaration, KtFunction, KtNamedFunction, KtProperty, KtParameter, KtPrimaryConstructor, KtFunctionLiteral,
    KtDestructuringDeclarationEntry
);

macro_rules! with_body {
    ($($t:ident),*) => {$(impl $t {
        /// `getBodyExpression()`.
        pub fn body_expression(self, ast: &Ast) -> Option<NodeId> {
            body_expression(ast, self.0)
        }

        /// `getBodyBlockExpression()`.
        pub fn body_block_expression(self, ast: &Ast) -> Option<KtBlockExpression> {
            KtBlockExpression::cast(ast, body_expression(ast, self.0)?)
        }

        pub fn has_block_body(self, ast: &Ast) -> bool {
            has_block_body(ast, self.0)
        }

        pub fn has_body(self, ast: &Ast) -> bool {
            body_expression(ast, self.0).is_some()
        }
    })*};
}

with_body!(KtDeclarationWithBody, KtFunction, KtNamedFunction, KtPrimaryConstructor, KtFunctionLiteral, KtPropertyAccessor);

impl KtParameterList {
    pub fn parameters(self, ast: &Ast) -> Vec<KtParameter> {
        ast.children(self.0).filter_map(|c| KtParameter::cast(ast, c)).collect()
    }
}

impl KtParameter {
    pub fn equals_token(self, ast: &Ast) -> Option<NodeId> {
        ast.find_child_by_type(self.0, EQ)
    }

    /// `getDefaultValue()`: the next expression sibling of `=`.
    pub fn default_value(self, ast: &Ast) -> Option<NodeId> {
        ast.siblings(self.equals_token(ast)?, true).find(|&s| KtExpression::is(ast, s))
    }

    pub fn has_default_value(self, ast: &Ast) -> bool {
        self.default_value(ast).is_some()
    }

    pub fn destructuring_declaration(self, ast: &Ast) -> Option<KtDestructuringDeclaration> {
        KtDestructuringDeclaration::cast(ast, ast.find_child_by_type(self.0, DESTRUCTURING_DECLARATION)?)
    }

    pub fn has_val_or_var(self, ast: &Ast) -> bool {
        ast.children(self.0).any(|c| matches!(ast.element_type(c), VAL_KEYWORD | VAR_KEYWORD))
    }

    /// `isLambdaParameter()`: the parent's parent is a function literal.
    pub fn is_lambda_parameter(self, ast: &Ast) -> bool {
        ast.tree_parent(self.0).and_then(|p| ast.tree_parent(p)).is_some_and(|g| KtFunctionLiteral::is(ast, g))
    }
}

impl KtProperty {
    pub fn is_var(self, ast: &Ast) -> bool {
        ast.find_child_by_type(self.0, VAR_KEYWORD).is_some()
    }

    /// `getInitializer()`: the next expression sibling of `=`.
    pub fn initializer(self, ast: &Ast) -> Option<NodeId> {
        ast.siblings(ast.find_child_by_type(self.0, EQ)?, true).find(|&s| KtExpression::is(ast, s))
    }

    pub fn has_initializer(self, ast: &Ast) -> bool {
        self.initializer(ast).is_some()
    }
}

impl KtNamedFunction {
    /// `getContextReceivers()`: the `context(T)` entries without a name.
    pub fn context_receivers(self, ast: &Ast) -> Vec<NodeId> {
        self.context_list_children(ast, CONTEXT_RECEIVER)
    }

    /// `getContextParameters()` (Kotlin 2.4 API; 2.2.21 has no such method): the named `context(p: T)` entries.
    pub fn context_parameters(self, ast: &Ast) -> Vec<KtParameter> {
        self.context_list_children(ast, VALUE_PARAMETER).into_iter().map(KtParameter).collect()
    }

    fn context_list_children(self, ast: &Ast, kind: ktrs_syntax::SyntaxKind) -> Vec<NodeId> {
        let Some(list) = modifier_list(ast, self.0).and_then(|m| m.context_parameter_list(ast)) else { return Vec::new() };
        ast.children(list).filter(|&c| ast.element_type(c) == kind).collect()
    }
}

impl KtDestructuringDeclaration {
    pub fn entries(self, ast: &Ast) -> Vec<KtDestructuringDeclarationEntry> {
        ast.children(self.0).filter_map(|c| KtDestructuringDeclarationEntry::cast(ast, c)).collect()
    }
}
