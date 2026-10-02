//! `KtClassOrObject`, `KtClass`, `KtClassBody`, `KtTypeAlias`, and the file-level containers (`KtFile`,
//! `KtImportList`).

use ktrs_syntax::SyntaxKind::*;

use super::classes::*;
use crate::arena::{Ast, NodeId};

macro_rules! class_or_object {
    ($($t:ident),*) => {$(impl $t {
        pub fn body(self, ast: &Ast) -> Option<KtClassBody> {
            KtClassBody::cast(ast, ast.find_child_by_type(self.0, CLASS_BODY)?)
        }

        pub fn primary_constructor(self, ast: &Ast) -> Option<KtPrimaryConstructor> {
            KtPrimaryConstructor::cast(ast, ast.find_child_by_type(self.0, PRIMARY_CONSTRUCTOR)?)
        }

        pub fn primary_constructor_parameters(self, ast: &Ast) -> Vec<KtParameter> {
            self.primary_constructor(ast).map(|c| c.value_parameters(ast)).unwrap_or_default()
        }

        /// `isAnnotation()`: has the `annotation` modifier.
        pub fn is_annotation(self, ast: &Ast) -> bool {
            self.has_modifier(ast, ANNOTATION_KEYWORD)
        }

        /// `getDeclarations()`: the class body's.
        pub fn declarations(self, ast: &Ast) -> Vec<NodeId> {
            self.body(ast).map(|b| b.declarations(ast)).unwrap_or_default()
        }
    })*};
}

class_or_object!(KtClassOrObject, KtClass, KtObjectDeclaration, KtEnumEntry);

impl KtClass {
    /// `isInterface()`: an `interface` keyword child.
    pub fn is_interface(self, ast: &Ast) -> bool {
        ast.find_child_by_type(self.0, INTERFACE_KEYWORD).is_some()
    }

    pub fn is_enum(self, ast: &Ast) -> bool {
        self.has_modifier(ast, ENUM_KEYWORD)
    }
}

impl KtClassBody {
    /// `getFunctions()`: the `fun` children.
    pub fn functions(self, ast: &Ast) -> Vec<KtNamedFunction> {
        ast.children(self.0).filter_map(|c| KtNamedFunction::cast(ast, c)).collect()
    }

    /// `getDeclarations()`: the declaration children.
    pub fn declarations(self, ast: &Ast) -> Vec<NodeId> {
        ast.children(self.0).filter(|&c| KtDeclaration::is(ast, c)).collect()
    }
}

impl KtTypeAlias {
    pub fn type_reference(self, ast: &Ast) -> Option<KtTypeReference> {
        KtTypeReference::cast(ast, ast.find_child_by_type(self.0, TYPE_REFERENCE)?)
    }
}

impl KtFile {
    pub fn import_list(self, ast: &Ast) -> Option<KtImportList> {
        KtImportList::cast(ast, ast.find_child_by_type(self.0, IMPORT_LIST)?)
    }

    pub fn package_directive(self, ast: &Ast) -> Option<KtPackageDirective> {
        KtPackageDirective::cast(ast, ast.find_child_by_type(self.0, PACKAGE_DIRECTIVE)?)
    }

    /// `getDeclarations()`: the declaration children (a script's are not unwrapped).
    pub fn declarations(self, ast: &Ast) -> Vec<NodeId> {
        ast.children(self.0).filter(|&c| KtDeclaration::is(ast, c)).collect()
    }
}

impl KtImportList {
    pub fn imports(self, ast: &Ast) -> Vec<KtImportDirective> {
        ast.children(self.0).filter_map(|c| KtImportDirective::cast(ast, c)).collect()
    }
}

/// psiUtil `containingKtFile` / `PsiElement.getContainingFile()`: the root of `element`'s tree when it is a Kotlin
/// file (`None` inside a dummy holder).
pub fn containing_kt_file(ast: &Ast, element: NodeId) -> Option<KtFile> {
    let root = ast.parents_with_self(element).last()?;
    KtFile::cast(ast, root)
}
