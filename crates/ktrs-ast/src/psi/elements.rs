//! Accessors of the file, `when` and super type classes the standard rules cast to.

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
