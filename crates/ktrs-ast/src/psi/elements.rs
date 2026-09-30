//! Accessors of the declaration, expression and file classes the standard rules cast to.

use ktrs_psi::SUPER_TYPE_LIST_ENTRIES;
use ktrs_syntax::SyntaxKind::*;

use super::classes::*;
use crate::arena::{Ast, NodeId};

impl KtFile {
    /// psiUtil `KtFile.virtualFilePath`: the `LightVirtualFile`'s path, `"/" + psiFileName`.
    pub fn virtual_file_path(self, ast: &Ast) -> String {
        format!("/{}", ast.psi_file_name())
    }

    /// `containingFile.virtualFile.name`: the `psiFileName` as given.
    pub fn virtual_file_name(self, ast: &Ast) -> &str {
        ast.psi_file_name()
    }
}

impl KtFunction {
    /// `getNameIdentifier()`; only `KtNamedFunction` has one.
    pub fn name_identifier(self, ast: &Ast) -> Option<NodeId> {
        match ast.element_type(self.node()) {
            FUN => ast.find_child_by_type(self.node(), IDENTIFIER),
            _ => None,
        }
    }

    /// `getName()` of a `KtNamedFunction` (`KtNamedDeclarationStub`): the unquoted identifier.
    /// Constructors (named after their class upstream) and function literals give `None`.
    pub fn name(self, ast: &Ast) -> Option<String> {
        Some(ktrs_psi::unquote_identifier(&ast.text(self.name_identifier(ast)?)))
    }

    /// `getTypeReference()`: for `fun`, the first type reference after the first `:` (TypeRefHelpers.kt).
    pub fn type_reference(self, ast: &Ast) -> Option<KtTypeReference> {
        if ast.element_type(self.node()) != FUN {
            return None;
        }
        ast.children(self.node())
            .skip_while(|&c| ast.element_type(c) != COLON)
            .find_map(|c| KtTypeReference::cast(ast, c))
    }

    /// `hasDeclaredReturnType()`: `typeReference != null` for `fun`; false otherwise.
    pub fn has_declared_return_type(self, ast: &Ast) -> bool {
        self.type_reference(ast).is_some()
    }

    /// `getBodyExpression()`: the first `KtExpression` child (a block for secondary constructors).
    pub fn body_expression(self, ast: &Ast) -> Option<NodeId> {
        match ast.element_type(self.node()) {
            PRIMARY_CONSTRUCTOR => None,
            SECONDARY_CONSTRUCTOR => ast.find_child_by_type(self.node(), BLOCK),
            _ => ast.children(self.node()).find(|&c| KtExpression::is(ast, c)),
        }
    }
}

impl KtWhenEntry {
    pub fn else_keyword(self, ast: &Ast) -> Option<NodeId> {
        ast.find_child_by_type(self.node(), ELSE_KEYWORD)
    }

    pub fn guard(self, ast: &Ast) -> Option<KtWhenEntryGuard> {
        ast.children(self.node()).find_map(|c| KtWhenEntryGuard::cast(ast, c))
    }

    /// `isElse()`: an `else` keyword and no guard.
    pub fn is_else(self, ast: &Ast) -> bool {
        self.else_keyword(ast).is_some() && self.guard(ast).is_none()
    }
}

impl KtWhenExpression {
    pub fn left_parenthesis(self, ast: &Ast) -> Option<NodeId> {
        ast.find_child_by_type(self.node(), LPAR)
    }
}

impl KtSuperTypeList {
    pub fn entries(self, ast: &Ast) -> Vec<KtSuperTypeListEntry> {
        ast.children(self.node())
            .filter(|&c| SUPER_TYPE_LIST_ENTRIES.contains(ast.element_type(c)))
            .map(|c| KtSuperTypeListEntry::of(ast, c))
            .collect()
    }
}

impl KtDotQualifiedExpression {
    pub fn receiver_expression(self, ast: &Ast) -> Option<NodeId> {
        KtQualifiedExpression::of(ast, self.node()).receiver_expression(ast)
    }

    pub fn selector_expression(self, ast: &Ast) -> Option<NodeId> {
        KtQualifiedExpression::of(ast, self.node()).selector_expression(ast)
    }
}
