//! Element type -> PSI class, from the factories in `KtNodeTypes`, `KtStubBasedElementTypes`,
//! `KDocElementTypes`/`KDocTokens` and IntelliJ's leaf factory (whitespace, comments).

use ktrs_syntax::SyntaxKind::{self, *};

use crate::element::PsiElement;

/// Simple name of the JVM class the compiler instantiates for `e`.
pub fn psi_class_name(e: &PsiElement) -> &'static str {
    if e.is_leaf() {
        return match e.kind() {
            WHITE_SPACE => "PsiWhiteSpaceImpl",
            EOL_COMMENT | BLOCK_COMMENT | SHEBANG_COMMENT => "PsiCommentImpl",
            _ => "LeafPsiElement",
        };
    }
    if e.is_file() {
        return "KtFile";
    }
    node_class_name(e.kind())
}

fn node_class_name(kind: SyntaxKind) -> &'static str {
    match kind {
        CLASS => "KtClass",
        FUN => "KtNamedFunction",
        PROPERTY => "KtProperty",
        DESTRUCTURING_DECLARATION => "KtDestructuringDeclaration",
        DESTRUCTURING_DECLARATION_ENTRY => "KtDestructuringDeclarationEntry",
        OBJECT_DECLARATION => "KtObjectDeclaration",
        TYPEALIAS => "KtTypeAlias",
        COMPANION_BLOCK => "KtCompanionBlock",
        ENUM_ENTRY => "KtEnumEntry",
        CLASS_INITIALIZER => "KtClassInitializer",
        SCRIPT_INITIALIZER => "KtScriptInitializer",
        SECONDARY_CONSTRUCTOR => "KtSecondaryConstructor",
        PRIMARY_CONSTRUCTOR => "KtPrimaryConstructor",
        CONTEXT_RECEIVER => "KtContextReceiver",
        CONTEXT_PARAMETER_LIST => "KtContextReceiverList",
        TYPE_PARAMETER_LIST => "KtTypeParameterList",
        TYPE_PARAMETER => "KtTypeParameter",
        SUPER_TYPE_LIST => "KtSuperTypeList",
        DELEGATED_SUPER_TYPE_ENTRY => "KtDelegatedSuperTypeEntry",
        SUPER_TYPE_CALL_ENTRY => "KtSuperTypeCallEntry",
        SUPER_TYPE_ENTRY => "KtSuperTypeEntry",
        PROPERTY_DELEGATE => "KtPropertyDelegate",
        CONSTRUCTOR_CALLEE => "KtConstructorCalleeExpression",
        VALUE_PARAMETER_LIST => "KtParameterList",
        VALUE_PARAMETER => "KtParameter",
        CLASS_BODY => "KtClassBody",
        IMPORT_LIST => "KtImportList",
        FILE_ANNOTATION_LIST => "KtFileAnnotationList",
        IMPORT_DIRECTIVE => "KtImportDirective",
        IMPORT_ALIAS => "KtImportAlias",
        MODIFIER_LIST => "KtDeclarationModifierList",
        ANNOTATION => "KtAnnotation",
        ANNOTATION_ENTRY => "KtAnnotationEntry",
        ANNOTATION_TARGET => "KtAnnotationUseSiteTarget",
        TYPE_ARGUMENT_LIST => "KtTypeArgumentList",
        VALUE_ARGUMENT_LIST => "KtValueArgumentList",
        VALUE_ARGUMENT => "KtValueArgument",
        CONTRACT_EFFECT_LIST => "KtContractEffectList",
        CONTRACT_EFFECT => "KtContractEffect",
        LAMBDA_ARGUMENT => "KtLambdaArgument",
        VALUE_ARGUMENT_NAME => "KtValueArgumentName",
        TYPE_REFERENCE => "KtTypeReference",
        USER_TYPE => "KtUserType",
        DYNAMIC_TYPE => "KtDynamicType",
        FUNCTION_TYPE => "KtFunctionType",
        FUNCTION_TYPE_RECEIVER => "KtFunctionTypeReceiver",
        NULLABLE_TYPE => "KtNullableType",
        INTERSECTION_TYPE => "KtIntersectionType",
        TYPE_PROJECTION => "KtTypeProjection",
        PROPERTY_ACCESSOR => "KtPropertyAccessor",
        BACKING_FIELD => "KtBackingField",
        INITIALIZER_LIST => "KtInitializerList",
        TYPE_CONSTRAINT_LIST => "KtTypeConstraintList",
        TYPE_CONSTRAINT => "KtTypeConstraint",
        CONSTRUCTOR_DELEGATION_CALL => "KtConstructorDelegationCall",
        CONSTRUCTOR_DELEGATION_REFERENCE => "KtConstructorDelegationReferenceExpression",
        NULL | BOOLEAN_CONSTANT | FLOAT_CONSTANT | CHARACTER_CONSTANT | INTEGER_CONSTANT => "KtConstantExpression",
        STRING_TEMPLATE => "KtStringTemplateExpression",
        LONG_STRING_TEMPLATE_ENTRY => "KtBlockStringTemplateEntry",
        SHORT_STRING_TEMPLATE_ENTRY => "KtSimpleNameStringTemplateEntry",
        LITERAL_STRING_TEMPLATE_ENTRY => "KtLiteralStringTemplateEntry",
        ESCAPE_STRING_TEMPLATE_ENTRY => "KtEscapeStringTemplateEntry",
        STRING_INTERPOLATION_PREFIX => "KtStringInterpolationPrefix",
        PARENTHESIZED => "KtParenthesizedExpression",
        RETURN => "KtReturnExpression",
        THROW => "KtThrowExpression",
        CONTINUE => "KtContinueExpression",
        BREAK => "KtBreakExpression",
        IF => "KtIfExpression",
        CONDITION | LOOP_RANGE | LABEL_QUALIFIER | INDICES => "KtContainerNode",
        THEN | ELSE | BODY => "KtContainerNodeForControlStructureBody",
        TRY => "KtTryExpression",
        CATCH => "KtCatchClause",
        FINALLY => "KtFinallySection",
        FOR => "KtForExpression",
        WHILE => "KtWhileExpression",
        DO_WHILE => "KtDoWhileExpression",
        BLOCK => "KtBlockExpression",
        LAMBDA_EXPRESSION => "KtLambdaExpression",
        FUNCTION_LITERAL => "KtFunctionLiteral",
        ANNOTATED_EXPRESSION => "KtAnnotatedExpression",
        REFERENCE_EXPRESSION => "KtNameReferenceExpression",
        ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION => "KtEnumEntrySuperclassReferenceExpression",
        OPERATION_REFERENCE => "KtOperationReferenceExpression",
        LABEL => "KtLabelReferenceExpression",
        THIS_EXPRESSION => "KtThisExpression",
        SUPER_EXPRESSION => "KtSuperExpression",
        BINARY_EXPRESSION => "KtBinaryExpression",
        BINARY_WITH_TYPE => "KtBinaryExpressionWithTypeRHS",
        IS_EXPRESSION => "KtIsExpression",
        PREFIX_EXPRESSION => "KtPrefixExpression",
        POSTFIX_EXPRESSION => "KtPostfixExpression",
        LABELED_EXPRESSION => "KtLabeledExpression",
        CALL_EXPRESSION => "KtCallExpression",
        ARRAY_ACCESS_EXPRESSION => "KtArrayAccessExpression",
        DOT_QUALIFIED_EXPRESSION => "KtDotQualifiedExpression",
        CALLABLE_REFERENCE_EXPRESSION => "KtCallableReferenceExpression",
        CLASS_LITERAL_EXPRESSION => "KtClassLiteralExpression",
        SAFE_ACCESS_EXPRESSION => "KtSafeQualifiedExpression",
        OBJECT_LITERAL => "KtObjectLiteralExpression",
        WHEN => "KtWhenExpression",
        WHEN_ENTRY => "KtWhenEntry",
        WHEN_ENTRY_GUARD => "KtWhenEntryGuard",
        WHEN_CONDITION_IN_RANGE => "KtWhenConditionInRange",
        WHEN_CONDITION_IS_PATTERN => "KtWhenConditionIsPattern",
        WHEN_CONDITION_EXPRESSION => "KtWhenConditionWithExpression",
        COLLECTION_LITERAL_EXPRESSION => "KtCollectionLiteralExpression",
        PACKAGE_DIRECTIVE => "KtPackageDirective",
        SCRIPT => "KtScript",
        DOC_COMMENT => "KDocImpl",
        KDOC_MARKDOWN_LINK => "KDocLink",
        KDOC_SECTION => "KDocSection",
        KDOC_TAG => "KDocTag",
        KDOC_NAME => "KDocName",
        ERROR_ELEMENT => "PsiErrorElementImpl",
        _ => "?",
    }
}

/// `LazyParseablePsiElement` / `CompositePsiElement` classes that keep IntelliJ's all-children `getChildren()`.
pub(crate) fn children_include_leaves(kind: SyntaxKind) -> bool {
    matches!(kind, LAMBDA_EXPRESSION | DOC_COMMENT | ERROR_ELEMENT)
}

/// Composite classes whose `ASTNode` is the PSI element itself.
pub(crate) fn node_is_psi(kind: SyntaxKind) -> bool {
    matches!(kind, BLOCK | LAMBDA_EXPRESSION | DOC_COMMENT | ERROR_ELEMENT)
}

pub(crate) fn is_kt_element(e: &PsiElement) -> bool {
    !e.is_leaf() && !matches!(e.kind(), DOC_COMMENT | KDOC_SECTION | KDOC_TAG | ERROR_ELEMENT)
}

pub(crate) fn is_declaration(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        CLASS
            | OBJECT_DECLARATION
            | ENUM_ENTRY
            | FUN
            | PROPERTY
            | TYPEALIAS
            | DESTRUCTURING_DECLARATION
            | DESTRUCTURING_DECLARATION_ENTRY
            | CLASS_INITIALIZER
            | SCRIPT_INITIALIZER
            | SECONDARY_CONSTRUCTOR
            | PRIMARY_CONSTRUCTOR
            | PROPERTY_ACCESSOR
            | BACKING_FIELD
            | VALUE_PARAMETER
            | TYPE_PARAMETER
            | SCRIPT
            | FUNCTION_LITERAL
    )
}

pub(crate) fn is_expression(kind: SyntaxKind) -> bool {
    is_declaration(kind)
        || matches!(
            kind,
            CONSTRUCTOR_CALLEE
                | CONSTRUCTOR_DELEGATION_REFERENCE
                | NULL
                | BOOLEAN_CONSTANT
                | FLOAT_CONSTANT
                | CHARACTER_CONSTANT
                | INTEGER_CONSTANT
                | STRING_TEMPLATE
                | PARENTHESIZED
                | RETURN
                | THROW
                | CONTINUE
                | BREAK
                | IF
                | TRY
                | FOR
                | WHILE
                | DO_WHILE
                | BLOCK
                | LAMBDA_EXPRESSION
                | ANNOTATED_EXPRESSION
                | REFERENCE_EXPRESSION
                | ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION
                | OPERATION_REFERENCE
                | LABEL
                | THIS_EXPRESSION
                | SUPER_EXPRESSION
                | BINARY_EXPRESSION
                | BINARY_WITH_TYPE
                | IS_EXPRESSION
                | PREFIX_EXPRESSION
                | POSTFIX_EXPRESSION
                | LABELED_EXPRESSION
                | CALL_EXPRESSION
                | ARRAY_ACCESS_EXPRESSION
                | DOT_QUALIFIED_EXPRESSION
                | CALLABLE_REFERENCE_EXPRESSION
                | CLASS_LITERAL_EXPRESSION
                | SAFE_ACCESS_EXPRESSION
                | OBJECT_LITERAL
                | WHEN
                | COLLECTION_LITERAL_EXPRESSION
        )
}

pub(crate) fn is_named_declaration(kind: SyntaxKind) -> bool {
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

pub(crate) fn is_callable_declaration(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        FUN | PROPERTY | VALUE_PARAMETER | DESTRUCTURING_DECLARATION_ENTRY | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR
            | FUNCTION_LITERAL
    )
}

pub(crate) fn is_function(kind: SyntaxKind) -> bool {
    matches!(kind, FUN | FUNCTION_LITERAL | PRIMARY_CONSTRUCTOR | SECONDARY_CONSTRUCTOR)
}

pub(crate) fn is_type_parameter_list_owner(kind: SyntaxKind) -> bool {
    is_callable_declaration(kind) || matches!(kind, CLASS | OBJECT_DECLARATION | ENUM_ENTRY | TYPEALIAS)
}

pub(crate) fn is_modifier_list_owner(kind: SyntaxKind) -> bool {
    is_declaration(kind) || matches!(kind, TYPE_REFERENCE | PACKAGE_DIRECTIVE | TYPE_PROJECTION)
}

pub(crate) fn is_reference_expression(kind: SyntaxKind) -> bool {
    is_simple_name_expression(kind)
        || matches!(
            kind,
            CALL_EXPRESSION | ARRAY_ACCESS_EXPRESSION | COLLECTION_LITERAL_EXPRESSION | CONSTRUCTOR_DELEGATION_REFERENCE
        )
}

pub(crate) fn is_simple_name_expression(kind: SyntaxKind) -> bool {
    matches!(kind, REFERENCE_EXPRESSION | ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION | OPERATION_REFERENCE | LABEL)
}
