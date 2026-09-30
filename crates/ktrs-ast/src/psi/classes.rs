//! The PSI classes as kind-based `instanceof` tests (`KtNodeTypes`/`KtStubBasedElementTypes` factories).

use ktrs_psi::classes::{is_declaration, is_expression, is_function, is_simple_name_expression};
use ktrs_syntax::SyntaxKind::{self, *};

use crate::arena::{Ast, NodeId};

macro_rules! psi_classes {
    ($( $(#[$doc:meta])* $name:ident($ast:ident, $n:ident) => $test:expr; )*) => {$(
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $name(pub(crate) NodeId);

        impl $name {
            /// `node.psi is` this class.
            pub fn is($ast: &Ast, $n: NodeId) -> bool {
                $test
            }

            /// `node.psi as?` this class.
            pub fn cast(ast: &Ast, n: NodeId) -> Option<$name> {
                Self::is(ast, n).then_some($name(n))
            }

            /// `node.psi as` this class.
            pub fn of(ast: &Ast, n: NodeId) -> $name {
                Self::cast(ast, n).unwrap_or_else(|| {
                    panic!("ClassCastException: {:?} cannot be cast to {}", ast.element_type(n), stringify!($name))
                })
            }

            /// `psi.node`.
            pub fn node(self) -> NodeId {
                self.0
            }
        }
    )*};
}

fn composite_of(ast: &Ast, n: NodeId, kinds: &[SyntaxKind]) -> bool {
    !ast.is_leaf_element(n) && kinds.contains(&ast.element_type(n))
}

fn composite_where(ast: &Ast, n: NodeId, test: fn(SyntaxKind) -> bool) -> bool {
    !ast.is_leaf_element(n) && test(ast.element_type(n))
}

psi_classes! {
    /// `KtFile`: the file element (`FileASTNode`) of a Kotlin file, not a dummy holder.
    KtFile(ast, n) => ast.is_file_element(n) && ast.element_type(n) == FILE;
    KtElement(ast, n) => KtFile::is(ast, n)
        || composite_where(ast, n, |k| !matches!(k, DOC_COMMENT | KDOC_SECTION | KDOC_TAG | ERROR_ELEMENT | DUMMY_HOLDER | FILE));
    KtExpression(ast, n) => composite_where(ast, n, is_expression);
    KtDeclaration(ast, n) => composite_where(ast, n, is_declaration);
    KtFunction(ast, n) => composite_where(ast, n, is_function);
    KtNamedFunction(ast, n) => composite_of(ast, n, &[FUN]);
    KtClass(ast, n) => composite_of(ast, n, &[CLASS, ENUM_ENTRY]);
    KtEnumEntry(ast, n) => composite_of(ast, n, &[ENUM_ENTRY]);
    KtProperty(ast, n) => composite_of(ast, n, &[PROPERTY]);
    KtPropertyAccessor(ast, n) => composite_of(ast, n, &[PROPERTY_ACCESSOR]);
    KtClassInitializer(ast, n) => composite_of(ast, n, &[CLASS_INITIALIZER]);
    KtPrimaryConstructor(ast, n) => composite_of(ast, n, &[PRIMARY_CONSTRUCTOR]);
    KtFunctionLiteral(ast, n) => composite_of(ast, n, &[FUNCTION_LITERAL]);
    KtLambdaExpression(ast, n) => composite_of(ast, n, &[LAMBDA_EXPRESSION]);
    KtBlockExpression(ast, n) => composite_of(ast, n, &[BLOCK]);
    KtBinaryExpression(ast, n) => composite_of(ast, n, &[BINARY_EXPRESSION]);
    KtAnnotatedExpression(ast, n) => composite_of(ast, n, &[ANNOTATED_EXPRESSION]);
    KtAnnotationEntry(ast, n) => composite_of(ast, n, &[ANNOTATION_ENTRY]);
    KtFileAnnotationList(ast, n) => composite_of(ast, n, &[FILE_ANNOTATION_LIST]);
    KtStringTemplateExpression(ast, n) => composite_of(ast, n, &[STRING_TEMPLATE]);
    KtPackageDirective(ast, n) => composite_of(ast, n, &[PACKAGE_DIRECTIVE]);
    KtImportList(ast, n) => composite_of(ast, n, &[IMPORT_LIST]);
    KtImportDirective(ast, n) => composite_of(ast, n, &[IMPORT_DIRECTIVE]);
    KtImportAlias(ast, n) => composite_of(ast, n, &[IMPORT_ALIAS]);
    KtQualifiedExpression(ast, n) => composite_of(ast, n, &[DOT_QUALIFIED_EXPRESSION, SAFE_ACCESS_EXPRESSION]);
    KtDotQualifiedExpression(ast, n) => composite_of(ast, n, &[DOT_QUALIFIED_EXPRESSION]);
    KtSimpleNameExpression(ast, n) => composite_where(ast, n, is_simple_name_expression);
    KtTypeReference(ast, n) => composite_of(ast, n, &[TYPE_REFERENCE]);
    KtSuperTypeList(ast, n) => composite_of(ast, n, &[SUPER_TYPE_LIST]);
    KtSuperTypeListEntry(ast, n) => composite_of(ast, n, &[DELEGATED_SUPER_TYPE_ENTRY, SUPER_TYPE_CALL_ENTRY, SUPER_TYPE_ENTRY]);
    KtWhenExpression(ast, n) => composite_of(ast, n, &[WHEN]);
    KtWhenEntry(ast, n) => composite_of(ast, n, &[WHEN_ENTRY]);
    KtWhenEntryGuard(ast, n) => composite_of(ast, n, &[WHEN_ENTRY_GUARD]);
    /// `PsiWhiteSpace` (`PsiWhiteSpaceImpl`).
    PsiWhiteSpace(ast, n) => ast.is_leaf_element(n) && ast.element_type(n) == WHITE_SPACE;
    /// `PsiComment`: comment leaves and KDoc.
    PsiComment(ast, n) => if ast.is_leaf_element(n) {
        matches!(ast.element_type(n), EOL_COMMENT | BLOCK_COMMENT | SHEBANG_COMMENT)
    } else {
        ast.element_type(n) == DOC_COMMENT
    };
}
