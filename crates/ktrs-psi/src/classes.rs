//! Element type -> PSI class membership (`instanceof` of the abstract classes and interfaces), from the
//! factories in `KtNodeTypes`, `KtStubBasedElementTypes` and `KDocElementTypes`. Public so `ktrs_ast::psi`
//! (the same classes over the mutable AST) shares the tables.

use ktrs_syntax::SyntaxKind::{self, *};

use crate::element::PsiElement;

/// `LazyParseablePsiElement` / `CompositePsiElement` classes that keep IntelliJ's all-children `getChildren()`.
pub(crate) fn children_include_leaves(kind: SyntaxKind) -> bool {
    matches!(kind, LAMBDA_EXPRESSION | DOC_COMMENT | ERROR_ELEMENT)
}

/// Composite classes whose `ASTNode` is the PSI element itself.
pub(crate) fn node_is_psi(kind: SyntaxKind) -> bool {
    matches!(kind, BLOCK | LAMBDA_EXPRESSION | DOC_COMMENT | ERROR_ELEMENT)
}

pub fn is_kt_element(e: &PsiElement) -> bool {
    !e.is_leaf() && !matches!(e.kind(), DOC_COMMENT | KDOC_SECTION | KDOC_TAG | ERROR_ELEMENT)
}

const DECLARATIONS: [SyntaxKind; 18] = [
    CLASS,
    OBJECT_DECLARATION,
    ENUM_ENTRY,
    FUN,
    PROPERTY,
    TYPEALIAS,
    DESTRUCTURING_DECLARATION,
    DESTRUCTURING_DECLARATION_ENTRY,
    CLASS_INITIALIZER,
    SCRIPT_INITIALIZER,
    SECONDARY_CONSTRUCTOR,
    PRIMARY_CONSTRUCTOR,
    PROPERTY_ACCESSOR,
    BACKING_FIELD,
    VALUE_PARAMETER,
    TYPE_PARAMETER,
    SCRIPT,
    FUNCTION_LITERAL,
];

/// `KtExpression` kinds besides the declarations.
const OTHER_EXPRESSIONS: [SyntaxKind; 42] = [
    CONSTRUCTOR_CALLEE,
    CONSTRUCTOR_DELEGATION_REFERENCE,
    NULL,
    BOOLEAN_CONSTANT,
    FLOAT_CONSTANT,
    CHARACTER_CONSTANT,
    INTEGER_CONSTANT,
    STRING_TEMPLATE,
    PARENTHESIZED,
    RETURN,
    THROW,
    CONTINUE,
    BREAK,
    IF,
    TRY,
    FOR,
    WHILE,
    DO_WHILE,
    BLOCK,
    LAMBDA_EXPRESSION,
    ANNOTATED_EXPRESSION,
    REFERENCE_EXPRESSION,
    ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION,
    OPERATION_REFERENCE,
    LABEL,
    THIS_EXPRESSION,
    SUPER_EXPRESSION,
    BINARY_EXPRESSION,
    BINARY_WITH_TYPE,
    IS_EXPRESSION,
    PREFIX_EXPRESSION,
    POSTFIX_EXPRESSION,
    LABELED_EXPRESSION,
    CALL_EXPRESSION,
    ARRAY_ACCESS_EXPRESSION,
    DOT_QUALIFIED_EXPRESSION,
    CALLABLE_REFERENCE_EXPRESSION,
    CLASS_LITERAL_EXPRESSION,
    SAFE_ACCESS_EXPRESSION,
    OBJECT_LITERAL,
    WHEN,
    COLLECTION_LITERAL_EXPRESSION,
];

const DECLARATION_MASK: u128 = window_mask(&DECLARATIONS);
const EXPRESSION_MASK: u128 = DECLARATION_MASK | window_mask(&OTHER_EXPRESSIONS);

/// The kinds as bits of a 128-kind window starting at `CLASS` (fails to compile if one lies outside).
const fn window_mask(kinds: &[SyntaxKind]) -> u128 {
    let mut mask = 0;
    let mut i = 0;
    while i < kinds.len() {
        let offset = kinds[i] as u16 - CLASS as u16;
        assert!(offset < 128);
        mask |= 1 << offset;
        i += 1;
    }
    mask
}

fn in_window(kind: SyntaxKind, mask: u128) -> bool {
    let offset = (kind as u16).wrapping_sub(CLASS as u16);
    offset < 128 && mask >> offset & 1 != 0
}

pub fn is_declaration(kind: SyntaxKind) -> bool {
    in_window(kind, DECLARATION_MASK)
}

pub fn is_expression(kind: SyntaxKind) -> bool {
    in_window(kind, EXPRESSION_MASK)
}

pub fn is_named_declaration(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        CLASS
            | OBJECT_DECLARATION
            | ENUM_ENTRY
            | FUN
            | PROPERTY
            | TYPEALIAS
            | DESTRUCTURING_DECLARATION_ENTRY
            | SECONDARY_CONSTRUCTOR
            | PRIMARY_CONSTRUCTOR
            | VALUE_PARAMETER
            | TYPE_PARAMETER
            | SCRIPT
            | FUNCTION_LITERAL
    )
}

pub fn is_callable_declaration(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        FUN | PROPERTY | VALUE_PARAMETER | DESTRUCTURING_DECLARATION_ENTRY | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR
            | FUNCTION_LITERAL
    )
}

pub fn is_function(kind: SyntaxKind) -> bool {
    matches!(kind, FUN | FUNCTION_LITERAL | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR)
}

pub fn is_type_parameter_list_owner(kind: SyntaxKind) -> bool {
    is_callable_declaration(kind) || matches!(kind, CLASS | OBJECT_DECLARATION | ENUM_ENTRY | TYPEALIAS)
}

pub fn is_modifier_list_owner(kind: SyntaxKind) -> bool {
    is_declaration(kind) || matches!(kind, TYPE_REFERENCE | PACKAGE_DIRECTIVE | TYPE_PROJECTION)
}

/// `KtAnnotated` besides the file: modifier list owners, annotated expressions and type constraints.
pub fn is_annotated(kind: SyntaxKind) -> bool {
    is_modifier_list_owner(kind) || matches!(kind, ANNOTATED_EXPRESSION | TYPE_CONSTRAINT)
}

pub fn is_reference_expression(kind: SyntaxKind) -> bool {
    is_simple_name_expression(kind)
        || matches!(
            kind,
            CALL_EXPRESSION | ARRAY_ACCESS_EXPRESSION | COLLECTION_LITERAL_EXPRESSION | CONSTRUCTOR_DELEGATION_REFERENCE
        )
}

pub fn is_simple_name_expression(kind: SyntaxKind) -> bool {
    matches!(kind, REFERENCE_EXPRESSION | ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION | OPERATION_REFERENCE | LABEL)
}
