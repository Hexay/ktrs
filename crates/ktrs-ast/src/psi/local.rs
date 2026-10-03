//! `KtPsiUtil.isLocal(declaration)` / `getEnclosingElementForLocalDeclaration`, and the `isLocal()` of
//! `KtFunction` and `KtProperty` it relies on (same in Kotlin 2.2.21 and 2.4.10).

use ktrs_psi::classes::{is_function, is_named_declaration};
use ktrs_syntax::SyntaxKind::*;

use super::classes::*;
use crate::arena::{Ast, NodeId};

/// `KtPsiUtil.isLocal(declaration)`.
pub fn is_local(ast: &Ast, declaration: NodeId) -> bool {
    get_enclosing_element_for_local_declaration(ast, declaration, true).is_some()
}

/// `KtPsiUtil.isMemberOfObjectExpression`.
fn is_member_of_object_expression(ast: &Ast, property_or_function: NodeId) -> bool {
    let Some(parent) = ast.tree_parent(property_or_function).filter(|&p| ast.element_type(p) == CLASS_BODY) else {
        return false;
    };
    let Some(grandparent) = ast.tree_parent(parent).filter(|&g| ast.element_type(g) == OBJECT_DECLARATION) else {
        return false;
    };
    ast.tree_parent(grandparent).is_some_and(|g| ast.element_type(g) == OBJECT_LITERAL)
}

/// `KtPsiUtil.isNonLocalCallable`.
fn is_non_local_callable(ast: &Ast, declaration: NodeId) -> bool {
    match ast.element_type(declaration) {
        PROPERTY => !KtProperty(declaration).is_local(ast),
        kind if is_function(kind) => !KtFunction(declaration).is_local(ast),
        _ => false,
    }
}

/// `KtPsiUtil.getEnclosingElementForLocalDeclaration(declaration, skipParameters)`.
pub fn get_enclosing_element_for_local_declaration(ast: &Ast, declaration: NodeId, skip_parameters: bool) -> Option<NodeId> {
    let mut declaration = declaration;
    if ast.element_type(declaration) == TYPE_PARAMETER && skip_parameters {
        declaration = ast.parents(declaration).find(|&p| !ast.is_leaf_element(p) && is_named_declaration(ast.element_type(p)))?;
    } else if ast.element_type(declaration) == VALUE_PARAMETER {
        if let Some(function_type) = ast.parents(declaration).find(|&p| ast.element_type(p) == FUNCTION_TYPE) {
            return Some(function_type);
        }
        let parent = ast.tree_parent(declaration);
        let grandparent = parent.and_then(|p| ast.tree_parent(p));
        let has_val_or_var = ast.children(declaration).any(|c| matches!(ast.element_type(c), VAL_KEYWORD | VAR_KEYWORD));
        if has_val_or_var && grandparent.is_some_and(|g| ast.element_type(g) == PRIMARY_CONSTRUCTOR) {
            let containing_class = ast.tree_parent(grandparent.unwrap()).expect("primary constructor outside a class");
            return get_enclosing_element_for_local_declaration(ast, containing_class, skip_parameters);
        } else if skip_parameters
            && parent.is_some_and(|p| ast.element_type(p) != FOR)
            && grandparent.is_some_and(|g| ast.element_type(g) == FUN)
        {
            declaration = grandparent.unwrap();
        }
    }
    if ast.is_file_element(declaration) {
        return Some(declaration);
    }
    let mut current = ast.tree_parent(declaration);
    let non_local_callable = is_non_local_callable(ast, declaration);
    while let Some(c) = current {
        if ast.is_file_element(c) {
            return None;
        }
        let parent = ast.tree_parent(c);
        let parent_kind = parent.map(|p| ast.element_type(p));
        if parent_kind == Some(SCRIPT) {
            return None;
        }
        let kind = ast.element_type(c);
        if matches!(kind, CLASS_INITIALIZER | SCRIPT_INITIALIZER) {
            return ast.find_child_by_type(c, BLOCK);
        }
        if kind == PROPERTY || is_function(kind) {
            if parent.is_some_and(|p| KtFile::is(ast, p)) {
                return Some(c);
            }
            let in_class_body = parent_kind == Some(CLASS_BODY) && !is_member_of_object_expression(ast, c);
            let in_script_block =
                parent_kind == Some(BLOCK) && parent.and_then(|p| ast.tree_parent(p)).is_some_and(|g| ast.element_type(g) == SCRIPT);
            if in_class_body || in_script_block {
                return parent;
            }
        }
        if kind == VALUE_PARAMETER {
            return Some(c);
        }
        if matches!(kind, VALUE_ARGUMENT | LAMBDA_ARGUMENT) && !non_local_callable {
            return Some(c);
        }
        if kind == BLOCK && (!non_local_callable || parent_kind != Some(FUNCTION_LITERAL)) {
            return Some(c);
        }
        if matches!(kind, DELEGATED_SUPER_TYPE_ENTRY | SUPER_TYPE_CALL_ENTRY) {
            let grandparent = parent.and_then(|p| ast.tree_parent(p));
            if let Some(g) = grandparent.filter(|&g| KtClassOrObject::is(ast, g))
                && ast.tree_parent(g).is_none_or(|gg| ast.element_type(gg) != OBJECT_LITERAL)
            {
                return Some(g);
            }
        }
        current = parent;
    }
    None
}

impl KtFunction {
    /// `isLocal()`: `KtNamedFunction` unless directly in a file, class body or script block; always for a
    /// function literal; never for constructors.
    pub fn is_local(self, ast: &Ast) -> bool {
        match ast.element_type(self.0) {
            FUN => match ast.tree_parent(self.0) {
                None => false,
                Some(parent) if KtFile::is(ast, parent) || ast.element_type(parent) == CLASS_BODY => false,
                Some(parent) => ast.tree_parent(parent).is_none_or(|g| ast.element_type(g) != SCRIPT),
            },
            FUNCTION_LITERAL => true,
            _ => false,
        }
    }
}

impl KtNamedFunction {
    pub fn is_local(self, ast: &Ast) -> bool {
        KtFunction(self.0).is_local(ast)
    }
}

impl KtProperty {
    /// `isLocal()`: neither top level nor a member.
    pub fn is_local(self, ast: &Ast) -> bool {
        !self.is_top_level(ast) && !self.is_member(ast)
    }

    pub fn is_member(self, ast: &Ast) -> bool {
        let Some(parent) = ast.tree_parent(self.0) else { return false };
        KtClassOrObject::is(ast, parent)
            || ast.element_type(parent) == CLASS_BODY
            || (ast.element_type(parent) == BLOCK && ast.tree_parent(parent).is_some_and(|g| ast.element_type(g) == SCRIPT))
    }

    pub fn is_top_level(self, ast: &Ast) -> bool {
        ast.tree_parent(self.0).is_some_and(|p| KtFile::is(ast, p))
    }
}
