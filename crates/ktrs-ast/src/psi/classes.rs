//! The PSI classes as kind-based `instanceof` tests (`KtNodeTypes`/`KtStubBasedElementTypes` factories).

use ktrs_psi::TYPE_ELEMENT_TYPES;
use ktrs_psi::classes::{
    is_callable_declaration, is_declaration, is_expression, is_function, is_modifier_list_owner, is_named_declaration,
    is_reference_expression, is_simple_name_expression,
};
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

            /// `psi.text`.
            pub fn text(self, ast: &Ast) -> String {
                ast.text(self.0)
            }

            /// psiUtil `startOffset` (UTF-8, in the current tree).
            pub fn start_offset(self, ast: &Ast) -> usize {
                ast.start_offset(self.0)
            }
        }

        impl PsiClass for $name {
            fn cast(ast: &Ast, n: NodeId) -> Option<$name> {
                $name::cast(ast, n)
            }

            fn node(self) -> NodeId {
                self.0
            }
        }
    )*};
}

/// A PSI class, for code generic over one (`filterIsInstance<T>()`, `findChildrenByClass<T>()`).
pub trait PsiClass: Copy {
    fn cast(ast: &Ast, n: NodeId) -> Option<Self>;
    fn node(self) -> NodeId;
}

fn composite_of(ast: &Ast, n: NodeId, kinds: &[SyntaxKind]) -> bool {
    !ast.is_leaf_element(n) && kinds.contains(&ast.element_type(n))
}

fn composite_where(ast: &Ast, n: NodeId, test: fn(SyntaxKind) -> bool) -> bool {
    !ast.is_leaf_element(n) && test(ast.element_type(n))
}

/// `KtDeclarationWithBody` implementers.
pub(crate) fn is_declaration_with_body(kind: SyntaxKind) -> bool {
    is_function(kind) || kind == PROPERTY_ACCESSOR
}

/// `KtDeclarationWithInitializer` implementers (`KtParameter` is not one).
pub(crate) fn is_declaration_with_initializer(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        PROPERTY | DESTRUCTURING_DECLARATION_ENTRY | FUN | PROPERTY_ACCESSOR | BACKING_FIELD | DESTRUCTURING_DECLARATION
    )
}

psi_classes! {
    /// `KtFile`: the file element (`FileASTNode`) of a Kotlin file, not a dummy holder.
    KtFile(ast, n) => ast.is_file_element(n) && ast.element_type(n) == FILE;
    KtElement(ast, n) => KtFile::is(ast, n)
        || composite_where(ast, n, |k| !matches!(k, DOC_COMMENT | KDOC_SECTION | KDOC_TAG | ERROR_ELEMENT | DUMMY_HOLDER | FILE));
    KtExpression(ast, n) => composite_where(ast, n, is_expression);
    KtDeclaration(ast, n) => composite_where(ast, n, is_declaration);
    KtNamedDeclaration(ast, n) => composite_where(ast, n, is_named_declaration);
    KtCallableDeclaration(ast, n) => composite_where(ast, n, is_callable_declaration);
    KtDeclarationWithBody(ast, n) => composite_where(ast, n, is_declaration_with_body);
    KtDeclarationWithInitializer(ast, n) => composite_where(ast, n, is_declaration_with_initializer);
    KtModifierListOwner(ast, n) => composite_where(ast, n, is_modifier_list_owner);
    /// `KtAnnotated`: modifier list owners, annotated expressions, type constraints and the file.
    KtAnnotated(ast, n) => KtFile::is(ast, n)
        || composite_where(ast, n, |k| is_modifier_list_owner(k) || matches!(k, ANNOTATED_EXPRESSION | TYPE_CONSTRAINT));
    KtFunction(ast, n) => composite_where(ast, n, is_function);
    KtNamedFunction(ast, n) => composite_of(ast, n, &[FUN]);
    KtClassOrObject(ast, n) => composite_of(ast, n, &[CLASS, OBJECT_DECLARATION, ENUM_ENTRY]);
    KtClass(ast, n) => composite_of(ast, n, &[CLASS, ENUM_ENTRY]);
    KtObjectDeclaration(ast, n) => composite_of(ast, n, &[OBJECT_DECLARATION]);
    KtEnumEntry(ast, n) => composite_of(ast, n, &[ENUM_ENTRY]);
    KtClassBody(ast, n) => composite_of(ast, n, &[CLASS_BODY]);
    KtTypeAlias(ast, n) => composite_of(ast, n, &[TYPEALIAS]);
    KtProperty(ast, n) => composite_of(ast, n, &[PROPERTY]);
    KtPropertyAccessor(ast, n) => composite_of(ast, n, &[PROPERTY_ACCESSOR]);
    KtParameter(ast, n) => composite_of(ast, n, &[VALUE_PARAMETER]);
    KtParameterList(ast, n) => composite_of(ast, n, &[VALUE_PARAMETER_LIST]);
    KtDestructuringDeclaration(ast, n) => composite_of(ast, n, &[DESTRUCTURING_DECLARATION]);
    KtDestructuringDeclarationEntry(ast, n) => composite_of(ast, n, &[DESTRUCTURING_DECLARATION_ENTRY]);
    KtClassInitializer(ast, n) => composite_of(ast, n, &[CLASS_INITIALIZER]);
    KtPrimaryConstructor(ast, n) => composite_of(ast, n, &[PRIMARY_CONSTRUCTOR]);
    KtFunctionLiteral(ast, n) => composite_of(ast, n, &[FUNCTION_LITERAL]);
    KtLambdaExpression(ast, n) => composite_of(ast, n, &[LAMBDA_EXPRESSION]);
    KtBlockExpression(ast, n) => composite_of(ast, n, &[BLOCK]);
    KtBinaryExpression(ast, n) => composite_of(ast, n, &[BINARY_EXPRESSION]);
    KtAnnotatedExpression(ast, n) => composite_of(ast, n, &[ANNOTATED_EXPRESSION]);
    KtModifierList(ast, n) => composite_of(ast, n, &[MODIFIER_LIST]);
    KtAnnotation(ast, n) => composite_of(ast, n, &[ANNOTATION]);
    KtAnnotationEntry(ast, n) => composite_of(ast, n, &[ANNOTATION_ENTRY]);
    KtFileAnnotationList(ast, n) => composite_of(ast, n, &[FILE_ANNOTATION_LIST]);
    KtCallExpression(ast, n) => composite_of(ast, n, &[CALL_EXPRESSION]);
    KtValueArgumentList(ast, n) => composite_of(ast, n, &[VALUE_ARGUMENT_LIST]);
    /// `KtValueArgument`, which `KtLambdaArgument` extends.
    KtValueArgument(ast, n) => composite_of(ast, n, &[VALUE_ARGUMENT, LAMBDA_ARGUMENT]);
    KtLambdaArgument(ast, n) => composite_of(ast, n, &[LAMBDA_ARGUMENT]);
    KtValueArgumentName(ast, n) => composite_of(ast, n, &[VALUE_ARGUMENT_NAME]);
    /// `KtReferenceExpression`: simple names and calls (`KtCallExpression` is one), array accesses, ...
    KtReferenceExpression(ast, n) => composite_where(ast, n, is_reference_expression);
    KtNameReferenceExpression(ast, n) => composite_of(ast, n, &[REFERENCE_EXPRESSION]);
    KtThisExpression(ast, n) => composite_of(ast, n, &[THIS_EXPRESSION]);
    KtConstantExpression(ast, n) => composite_of(ast, n, &[NULL, BOOLEAN_CONSTANT, FLOAT_CONSTANT, CHARACTER_CONSTANT, INTEGER_CONSTANT]);
    KtStringTemplateExpression(ast, n) => composite_of(ast, n, &[STRING_TEMPLATE]);
    KtLiteralStringTemplateEntry(ast, n) => composite_of(ast, n, &[LITERAL_STRING_TEMPLATE_ENTRY]);
    KtPackageDirective(ast, n) => composite_of(ast, n, &[PACKAGE_DIRECTIVE]);
    KtImportList(ast, n) => composite_of(ast, n, &[IMPORT_LIST]);
    KtImportDirective(ast, n) => composite_of(ast, n, &[IMPORT_DIRECTIVE]);
    KtImportAlias(ast, n) => composite_of(ast, n, &[IMPORT_ALIAS]);
    KtQualifiedExpression(ast, n) => composite_of(ast, n, &[DOT_QUALIFIED_EXPRESSION, SAFE_ACCESS_EXPRESSION]);
    KtDotQualifiedExpression(ast, n) => composite_of(ast, n, &[DOT_QUALIFIED_EXPRESSION]);
    KtSafeQualifiedExpression(ast, n) => composite_of(ast, n, &[SAFE_ACCESS_EXPRESSION]);
    KtSimpleNameExpression(ast, n) => composite_where(ast, n, is_simple_name_expression);
    KtIfExpression(ast, n) => composite_of(ast, n, &[IF]);
    KtLoopExpression(ast, n) => composite_of(ast, n, &[FOR, WHILE, DO_WHILE]);
    KtReturnExpression(ast, n) => composite_of(ast, n, &[RETURN]);
    KtTypeReference(ast, n) => composite_of(ast, n, &[TYPE_REFERENCE]);
    KtTypeElement(ast, n) => !ast.is_leaf_element(n) && TYPE_ELEMENT_TYPES.contains(ast.element_type(n));
    KtUserType(ast, n) => composite_of(ast, n, &[USER_TYPE]);
    KtNullableType(ast, n) => composite_of(ast, n, &[NULLABLE_TYPE]);
    KtFunctionType(ast, n) => composite_of(ast, n, &[FUNCTION_TYPE]);
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
