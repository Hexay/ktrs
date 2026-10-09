//! Every PSI class ktfmt touches, as a typed view. Concrete classes test their element type; abstract classes
//! and interfaces test membership (see `classes.rs`). Accessors live in `kt/*` traits and impls.

use ktrs_syntax::SyntaxKind::{self, *};

use crate::cast::psi_types;
use crate::classes::*;
use crate::element::PsiElement;
pub use crate::kt::KtFile;

fn node_of(e: &PsiElement, kinds: &[SyntaxKind]) -> bool {
    kinds.contains(&e.kind()) && !e.is_leaf() && !e.is_file()
}

fn node_where(e: &PsiElement, test: fn(SyntaxKind) -> bool) -> bool {
    test(e.kind()) && !e.is_leaf() && !e.is_file()
}

psi_types! {
    // ---- IntelliJ core ----
    /// `PsiComment`: comment leaves and `KDoc`.
    PsiComment(e) => if e.is_leaf() { matches!(e.kind(), EOL_COMMENT | BLOCK_COMMENT | SHEBANG_COMMENT) } else { node_of(e, &[DOC_COMMENT]) };
    PsiWhiteSpace(e) => e.is_leaf() && e.kind() == WHITE_SPACE;
    /// Any leaf token.
    LeafPsiElement(e) => e.is_leaf();
    PsiErrorElement(e) => node_of(e, &[ERROR_ELEMENT]);

    // ---- abstract classes and interfaces ----
    KtElement(e) => is_kt_element(e);
    KtExpression(e) => node_where(e, is_expression);
    KtDeclaration(e) => node_where(e, is_declaration);
    KtNamedDeclaration(e) => node_where(e, is_named_declaration);
    KtCallableDeclaration(e) => node_where(e, is_callable_declaration);
    KtDeclarationWithBody(e) => node_where(e, is_function) || node_of(e, &[PROPERTY_ACCESSOR]);
    KtFunction(e) => node_where(e, is_function);
    KtClassOrObject(e) => node_of(e, &[CLASS, ENUM_ENTRY, OBJECT_DECLARATION]);
    KtModifierListOwner(e) => node_where(e, is_modifier_list_owner);
    KtAnnotated(e) => e.is_file() || node_where(e, is_annotated);
    KtTypeParameterListOwner(e) => node_where(e, is_type_parameter_list_owner);
    KtValVarKeywordOwner(e) => node_of(e, &[PROPERTY, VALUE_PARAMETER, DESTRUCTURING_DECLARATION, DESTRUCTURING_DECLARATION_ENTRY]);
    KtDeclarationWithInitializer(e) => node_of(e, &[FUN, PROPERTY, DESTRUCTURING_DECLARATION, DESTRUCTURING_DECLARATION_ENTRY, PROPERTY_ACCESSOR, BACKING_FIELD]);
    KtConstructor(e) => node_of(e, &[PRIMARY_CONSTRUCTOR, SECONDARY_CONSTRUCTOR]);
    KtAnonymousInitializer(e) => node_of(e, &[CLASS_INITIALIZER, SCRIPT_INITIALIZER]);
    KtQualifiedExpression(e) => node_of(e, &[DOT_QUALIFIED_EXPRESSION, SAFE_ACCESS_EXPRESSION]);
    KtReferenceExpression(e) => node_where(e, is_reference_expression);
    KtSimpleNameExpression(e) => node_where(e, is_simple_name_expression);
    KtCallElement(e) => node_of(e, &[CALL_EXPRESSION, ANNOTATION_ENTRY, SUPER_TYPE_CALL_ENTRY, CONSTRUCTOR_DELEGATION_CALL]);
    KtUnaryExpression(e) => node_of(e, &[PREFIX_EXPRESSION, POSTFIX_EXPRESSION]);
    KtExpressionWithLabel(e) => node_of(e, &[RETURN, CONTINUE, BREAK, THIS_EXPRESSION, SUPER_EXPRESSION, LABELED_EXPRESSION]);
    KtLoopExpression(e) => node_of(e, &[FOR, WHILE, DO_WHILE]);
    KtWhileExpressionBase(e) => node_of(e, &[WHILE, DO_WHILE]);
    KtDoubleColonExpression(e) => node_of(e, &[CALLABLE_REFERENCE_EXPRESSION, CLASS_LITERAL_EXPRESSION]);
    KtContainerNode(e) => node_of(e, &[CONDITION, THEN, ELSE, LOOP_RANGE, BODY, LABEL_QUALIFIER, INDICES]);
    KtContainerNodeForControlStructureBody(e) => node_of(e, &[THEN, ELSE, BODY]);
    KtValueArgument(e) => node_of(e, &[VALUE_ARGUMENT, LAMBDA_ARGUMENT]);
    KtTypeElement(e) => node_of(e, &[USER_TYPE, DYNAMIC_TYPE, FUNCTION_TYPE, NULLABLE_TYPE, INTERSECTION_TYPE]);
    KtWhenCondition(e) => node_of(e, &[WHEN_CONDITION_IN_RANGE, WHEN_CONDITION_IS_PATTERN, WHEN_CONDITION_EXPRESSION]);
    KtStringTemplateEntry(e) => node_of(e, &[LONG_STRING_TEMPLATE_ENTRY, SHORT_STRING_TEMPLATE_ENTRY, LITERAL_STRING_TEMPLATE_ENTRY, ESCAPE_STRING_TEMPLATE_ENTRY]);
    KtStringTemplateEntryWithExpression(e) => node_of(e, &[LONG_STRING_TEMPLATE_ENTRY, SHORT_STRING_TEMPLATE_ENTRY]);
    KtSuperTypeListEntry(e) => node_of(e, &[DELEGATED_SUPER_TYPE_ENTRY, SUPER_TYPE_CALL_ENTRY, SUPER_TYPE_ENTRY]);
    KtModifierList(e) => node_of(e, &[MODIFIER_LIST]);
    KtContextParameterList(e) => node_of(e, &[CONTEXT_PARAMETER_LIST]);
    /// `KDoc` (the interface); `KDocImpl` is its only class.
    KDoc(e) => node_of(e, &[DOC_COMMENT]);
    KDocTag(e) => node_of(e, &[KDOC_SECTION, KDOC_TAG]);

    // ---- concrete classes (`KtFile` is in `kt/file.rs`) ----
    KtScript(e) => node_of(e, &[SCRIPT]);
    KtPackageDirective(e) => node_of(e, &[PACKAGE_DIRECTIVE]);
    KtImportList(e) => node_of(e, &[IMPORT_LIST]);
    KtImportDirective(e) => node_of(e, &[IMPORT_DIRECTIVE]);
    KtImportAlias(e) => node_of(e, &[IMPORT_ALIAS]);
    KtFileAnnotationList(e) => node_of(e, &[FILE_ANNOTATION_LIST]);
    KtClass(e) => node_of(e, &[CLASS, ENUM_ENTRY]);
    KtObjectDeclaration(e) => node_of(e, &[OBJECT_DECLARATION]);
    KtEnumEntry(e) => node_of(e, &[ENUM_ENTRY]);
    KtClassBody(e) => node_of(e, &[CLASS_BODY]);
    KtCompanionBlock(e) => node_of(e, &[COMPANION_BLOCK]);
    KtNamedFunction(e) => node_of(e, &[FUN]);
    KtProperty(e) => node_of(e, &[PROPERTY]);
    KtPropertyAccessor(e) => node_of(e, &[PROPERTY_ACCESSOR]);
    KtBackingField(e) => node_of(e, &[BACKING_FIELD]);
    KtPropertyDelegate(e) => node_of(e, &[PROPERTY_DELEGATE]);
    KtTypeAlias(e) => node_of(e, &[TYPEALIAS]);
    KtDestructuringDeclaration(e) => node_of(e, &[DESTRUCTURING_DECLARATION]);
    KtDestructuringDeclarationEntry(e) => node_of(e, &[DESTRUCTURING_DECLARATION_ENTRY]);
    KtClassInitializer(e) => node_of(e, &[CLASS_INITIALIZER]);
    KtScriptInitializer(e) => node_of(e, &[SCRIPT_INITIALIZER]);
    KtPrimaryConstructor(e) => node_of(e, &[PRIMARY_CONSTRUCTOR]);
    KtSecondaryConstructor(e) => node_of(e, &[SECONDARY_CONSTRUCTOR]);
    KtConstructorDelegationCall(e) => node_of(e, &[CONSTRUCTOR_DELEGATION_CALL]);
    KtConstructorDelegationReferenceExpression(e) => node_of(e, &[CONSTRUCTOR_DELEGATION_REFERENCE]);
    KtConstructorCalleeExpression(e) => node_of(e, &[CONSTRUCTOR_CALLEE]);
    KtSuperTypeList(e) => node_of(e, &[SUPER_TYPE_LIST]);
    KtDelegatedSuperTypeEntry(e) => node_of(e, &[DELEGATED_SUPER_TYPE_ENTRY]);
    KtSuperTypeCallEntry(e) => node_of(e, &[SUPER_TYPE_CALL_ENTRY]);
    KtSuperTypeEntry(e) => node_of(e, &[SUPER_TYPE_ENTRY]);
    KtInitializerList(e) => node_of(e, &[INITIALIZER_LIST]);
    KtTypeParameterList(e) => node_of(e, &[TYPE_PARAMETER_LIST]);
    KtTypeParameter(e) => node_of(e, &[TYPE_PARAMETER]);
    KtTypeConstraintList(e) => node_of(e, &[TYPE_CONSTRAINT_LIST]);
    KtTypeConstraint(e) => node_of(e, &[TYPE_CONSTRAINT]);
    KtParameterList(e) => node_of(e, &[VALUE_PARAMETER_LIST]);
    KtParameter(e) => node_of(e, &[VALUE_PARAMETER]);
    KtContextReceiverList(e) => node_of(e, &[CONTEXT_PARAMETER_LIST]);
    KtContextReceiver(e) => node_of(e, &[CONTEXT_RECEIVER]);
    KtDeclarationModifierList(e) => node_of(e, &[MODIFIER_LIST]);
    KtAnnotation(e) => node_of(e, &[ANNOTATION]);
    KtAnnotationEntry(e) => node_of(e, &[ANNOTATION_ENTRY]);
    KtAnnotationUseSiteTarget(e) => node_of(e, &[ANNOTATION_TARGET]);
    KtTypeReference(e) => node_of(e, &[TYPE_REFERENCE]);
    KtUserType(e) => node_of(e, &[USER_TYPE]);
    KtDynamicType(e) => node_of(e, &[DYNAMIC_TYPE]);
    KtFunctionType(e) => node_of(e, &[FUNCTION_TYPE]);
    KtFunctionTypeReceiver(e) => node_of(e, &[FUNCTION_TYPE_RECEIVER]);
    KtNullableType(e) => node_of(e, &[NULLABLE_TYPE]);
    KtIntersectionType(e) => node_of(e, &[INTERSECTION_TYPE]);
    KtTypeArgumentList(e) => node_of(e, &[TYPE_ARGUMENT_LIST]);
    KtTypeProjection(e) => node_of(e, &[TYPE_PROJECTION]);
    KtValueArgumentList(e) => node_of(e, &[VALUE_ARGUMENT_LIST]);
    KtLambdaArgument(e) => node_of(e, &[LAMBDA_ARGUMENT]);
    KtValueArgumentName(e) => node_of(e, &[VALUE_ARGUMENT_NAME]);
    KtContractEffectList(e) => node_of(e, &[CONTRACT_EFFECT_LIST]);
    KtContractEffect(e) => node_of(e, &[CONTRACT_EFFECT]);
    KtConstantExpression(e) => node_of(e, &[NULL, BOOLEAN_CONSTANT, FLOAT_CONSTANT, CHARACTER_CONSTANT, INTEGER_CONSTANT]);
    KtStringTemplateExpression(e) => node_of(e, &[STRING_TEMPLATE]);
    KtBlockStringTemplateEntry(e) => node_of(e, &[LONG_STRING_TEMPLATE_ENTRY]);
    KtSimpleNameStringTemplateEntry(e) => node_of(e, &[SHORT_STRING_TEMPLATE_ENTRY]);
    KtLiteralStringTemplateEntry(e) => node_of(e, &[LITERAL_STRING_TEMPLATE_ENTRY]);
    KtEscapeStringTemplateEntry(e) => node_of(e, &[ESCAPE_STRING_TEMPLATE_ENTRY]);
    KtStringInterpolationPrefix(e) => node_of(e, &[STRING_INTERPOLATION_PREFIX]);
    KtParenthesizedExpression(e) => node_of(e, &[PARENTHESIZED]);
    KtReturnExpression(e) => node_of(e, &[RETURN]);
    KtThrowExpression(e) => node_of(e, &[THROW]);
    KtContinueExpression(e) => node_of(e, &[CONTINUE]);
    KtBreakExpression(e) => node_of(e, &[BREAK]);
    KtIfExpression(e) => node_of(e, &[IF]);
    KtTryExpression(e) => node_of(e, &[TRY]);
    KtCatchClause(e) => node_of(e, &[CATCH]);
    KtFinallySection(e) => node_of(e, &[FINALLY]);
    KtForExpression(e) => node_of(e, &[FOR]);
    KtWhileExpression(e) => node_of(e, &[WHILE]);
    KtDoWhileExpression(e) => node_of(e, &[DO_WHILE]);
    KtBlockExpression(e) => node_of(e, &[BLOCK]);
    KtLambdaExpression(e) => node_of(e, &[LAMBDA_EXPRESSION]);
    KtFunctionLiteral(e) => node_of(e, &[FUNCTION_LITERAL]);
    KtAnnotatedExpression(e) => node_of(e, &[ANNOTATED_EXPRESSION]);
    KtNameReferenceExpression(e) => node_of(e, &[REFERENCE_EXPRESSION]);
    KtEnumEntrySuperclassReferenceExpression(e) => node_of(e, &[ENUM_ENTRY_SUPERCLASS_REFERENCE_EXPRESSION]);
    KtOperationReferenceExpression(e) => node_of(e, &[OPERATION_REFERENCE]);
    KtLabelReferenceExpression(e) => node_of(e, &[LABEL]);
    KtThisExpression(e) => node_of(e, &[THIS_EXPRESSION]);
    KtSuperExpression(e) => node_of(e, &[SUPER_EXPRESSION]);
    KtBinaryExpression(e) => node_of(e, &[BINARY_EXPRESSION]);
    KtBinaryExpressionWithTypeRHS(e) => node_of(e, &[BINARY_WITH_TYPE]);
    KtIsExpression(e) => node_of(e, &[IS_EXPRESSION]);
    KtPrefixExpression(e) => node_of(e, &[PREFIX_EXPRESSION]);
    KtPostfixExpression(e) => node_of(e, &[POSTFIX_EXPRESSION]);
    KtLabeledExpression(e) => node_of(e, &[LABELED_EXPRESSION]);
    KtCallExpression(e) => node_of(e, &[CALL_EXPRESSION]);
    KtArrayAccessExpression(e) => node_of(e, &[ARRAY_ACCESS_EXPRESSION]);
    KtDotQualifiedExpression(e) => node_of(e, &[DOT_QUALIFIED_EXPRESSION]);
    KtSafeQualifiedExpression(e) => node_of(e, &[SAFE_ACCESS_EXPRESSION]);
    KtCallableReferenceExpression(e) => node_of(e, &[CALLABLE_REFERENCE_EXPRESSION]);
    KtClassLiteralExpression(e) => node_of(e, &[CLASS_LITERAL_EXPRESSION]);
    KtObjectLiteralExpression(e) => node_of(e, &[OBJECT_LITERAL]);
    KtCollectionLiteralExpression(e) => node_of(e, &[COLLECTION_LITERAL_EXPRESSION]);
    KtWhenExpression(e) => node_of(e, &[WHEN]);
    KtWhenEntry(e) => node_of(e, &[WHEN_ENTRY]);
    KtWhenEntryGuard(e) => node_of(e, &[WHEN_ENTRY_GUARD]);
    KtWhenConditionInRange(e) => node_of(e, &[WHEN_CONDITION_IN_RANGE]);
    KtWhenConditionIsPattern(e) => node_of(e, &[WHEN_CONDITION_IS_PATTERN]);
    KtWhenConditionWithExpression(e) => node_of(e, &[WHEN_CONDITION_EXPRESSION]);
    KDocImpl(e) => node_of(e, &[DOC_COMMENT]);
    KDocSection(e) => node_of(e, &[KDOC_SECTION]);
    KDocLink(e) => node_of(e, &[KDOC_MARKDOWN_LINK]);
    KDocName(e) => node_of(e, &[KDOC_NAME]);
}
