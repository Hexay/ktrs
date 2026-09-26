//! Kotlin PSI over the ktrs syntax tree: typed views whose accessors return exactly what the compiler's
//! PSI classes (Kotlin v2.4.20 psi-api/psi-impl, AST code path) and IntelliJ core (idea/251.27812.49)
//! return. Scope: everything ktfmt v0.64 calls. Verified by `tests/fixtures.rs` against the JVM oracle
//! `tools/psi-accessors` (corpus: `psi_accessors compare`, see that example's header).
//!
//! # Porting conventions (Kotlin/Java call -> Rust)
//! - Every PSI class or interface is a struct of the same name that derefs to [`PsiElement`].
//!   `x is KtFoo` -> `x.is::<KtFoo>()`, `x as? KtFoo` -> `x.cast::<KtFoo>()`, `x as KtFoo` or an implicit
//!   upcast -> `x.upcast::<KtFoo>()`; anything taking `&PsiElement` accepts `&x`.
//! - Getters drop `get`: `getFoo()`/`foo` -> `foo()`, `isFoo` -> `is_foo()`, `hasFoo` -> `has_foo()`,
//!   `else` -> `r#else()`. Methods with arguments and psiUtil/PsiTreeUtil helpers keep their full name with
//!   Kotlin defaults spelled out: `getPrevSiblingIgnoringWhitespace()` ->
//!   `get_prev_sibling_ignoring_whitespace(false)`, `prevLeaf()` -> `prev_leaf(false)`.
//! - `T?` -> `Option<T>`, lists/arrays -> `Vec<T>`. A `@NotNull` getter that upstream can throw from
//!   returns `Option` too (its doc says so); `None` there means the JVM would have thrown.
//! - Element types: `KtTokens.X`, `KtNodeTypes.X`, `KtStubElementTypes.X` -> `SyntaxKind::X`.
//!   `elementType is KtModifierKeywordToken` -> [`is_modifier_keyword_token`]; `KtSingleValueToken.value`
//!   -> [`KtSingleValueToken::value`]; `KtTokens.ALL_ASSIGNMENTS` -> [`ALL_ASSIGNMENTS`].
//! - Offsets (`startOffset`, `endOffset`, `textRange`, `textOffset`) are UTF-8 byte offsets.
//! - `KtFile` = `KtFile::new(&parse)` for a `ktrs_parser::parse_file` result; ktfmt parses as a script
//!   (`FileKind::Script`). `PsiErrorElement.errorDescription` -> `error_description(&parse)`.
//! - Visitors implement [`KtVisitorVoid`] (method = snake_case of the upstream `visitX`, including the
//!   `(element, data)` forms ktfmt overrides). `super.visitX(x)` -> `kt_visitor_void::visit_x(self, x)`.
//!   A `KtTreeVisitorVoid` overrides `visit_element` with [`kt_tree_visitor_void::visit_element`], which
//!   is also its `super.visitElement(e)`. `e.accept(this)` -> `e.accept(self)`.
//!
//! # API
//! Base ([`PsiElement`], `element.rs`, `tree_util.rs`): `text`, `text_range`, `start_offset`, `end_offset`,
//! `text_length`, `text_offset`, `text_contains`, `element_type`, `parent`, `children` (class-dependent:
//! composites only for Kt classes, everything for files/lambdas/KDoc/error elements), `all_children`,
//! `first_child`, `last_child`, `next_sibling`, `prev_sibling`, `node`, `accept`, `accept_children`,
//! `find_child_by_type`, `find_child_by_type_set`, `find_children_by_type`, `find_last_child_by_type`,
//! `find_child_by_class`, `find_children_by_class`, `get_stub_or_psi_child`, `get_stub_or_psi_children`,
//! `get_stub_or_psi_children_set`, `siblings`, `get_{prev,next}_sibling_ignoring_whitespace[_and_comments]`,
//! `starts_with_comment`, `prev_leaf`, `next_leaf`, `get_parent_of_type`, `get_child_of_type`,
//! `get_children_of_type`, `collect_descendants_of_type`, `has_error_elements`, `deepest_{first,last}`.
//! Not upstream, for speed (`element_text.rs`, `element.rs`): `text_all`, `try_for_each_text_chunk`
//! (text checks without building it), `has_children`; `KtVisitorVoid::ignores_leaves`; `KtFile` caches its text.
//! Free: [`get_next_sibling_of_type`], [`get_prev_sibling_of_type`],
//! [`get_trailing_comma_by_closing_element`], [`get_trailing_comma_by_elements_list`],
//! [`try_flatten_string_concatenation_descendants`], [`unquote_identifier`], [`psi_class_name`].
//! [`AstNode`]: `psi`, `element_type`, `text`, `text_range`, `text_contains`, `is_psi_element`
//! (`node is PsiElement`), `children`, `first_child_node`, `last_child_node`, `tree_{next,prev,parent}`,
//! `find_child_by_type[_set]`, `get_children`.
//!
//! Interface methods (stamped on every implementing view, dispatching to the upstream override):
//! - `modifier_list` (KtModifierListOwner, KtNullableType); `name_identifier` (PsiNameIdentifierOwner:
//!   named declarations, KtImportAlias, KtLabeledExpression); `type_parameter_list`,
//!   `type_constraint_list` (KtTypeParameterListOwner); `receiver_type_reference`, `type_reference`,
//!   `value_parameter_list` (KtCallableDeclaration); `value_parameters`; `body_expression`,
//!   `body_block_expression` (KtDeclarationWithBody); `callee_expression`, `type_argument_list`,
//!   `value_argument_list`, `lambda_arguments` (KtCallElement and its classes).
//!
//! Classes (`kt/*.rs`):
//! - KtFile `import_list`; KtScript `block_expression`; KtPackageDirective `package_keyword`,
//!   `package_name_expression`, `package_names`, `qualified_name`, `fq_name`; KtImportList `imports`;
//!   KtImportDirective `imported_reference`, `alias`, `alias_name`, `is_all_under`, `imported_fq_name`,
//!   `import_path`, `is_valid_import`; KtImportAlias `name`; [`FqName`] `as_string`, `short_name`,
//!   `parent`, `child`; [`ImportPath`] `imported_name`.
//! - KtProperty `val_or_var_keyword`, `delegate`, `delegate_expression`, `initializer`, `accessors`,
//!   `getter`, `setter`, `field_declaration`, `is_var`; KtPropertyAccessor `is_getter`, `is_setter`,
//!   `parameter_list`, `parameter`, `name_placeholder`, `type_reference`, `return_type_reference`,
//!   `equals_token`, `initializer`, `{left,right}_parenthesis`; KtBackingField `name_placeholder`,
//!   `field_keyword`, `type_reference`, `return_type_reference`, `equals_token`, `initializer`;
//!   KtPropertyDelegate `expression`; KtParameter `destructuring_declaration`, `val_or_var_keyword`,
//!   `default_value`, `equals_token`; KtParameterList `parameters`, `trailing_comma`,
//!   `{left,right}_parenthesis`; KtDestructuringDeclaration `entries`, `initializer`, `val_or_var_keyword`,
//!   `l_par`, `r_par`, `trailing_comma`; KtDestructuringDeclarationEntry `initializer`, `equals_token`,
//!   `val_or_var_keyword`, `own_val_or_var_keyword`.
//! - KtClassOrObject (+KtClass, KtObjectDeclaration, KtEnumEntry) `colon`, `super_type_list`, `body`,
//!   `primary_constructor`, `declaration_keyword`; KtClass/KtEnumEntry `is_enum`; KtObjectDeclaration
//!   `is_companion`, `object_keyword`; KtEnumEntry `initializer_list`; KtInitializerList `initializers`;
//!   KtClassBody `enum_entries`; KtSuperTypeList `entries`; KtSuperTypeListEntry (+subclasses)
//!   `type_reference`, `type_as_user_type`; KtDelegatedSuperTypeEntry `delegate_expression`;
//!   KtConstructorCalleeExpression `type_reference`, `constructor_reference_expression`; KtConstructor
//!   (+primary, secondary) `constructor_keyword`, `has_constructor_keyword`; KtSecondaryConstructor
//!   `delegation_call`; KtConstructorDelegationCall `is_implicit`, `is_call_to_this`;
//!   KtConstructorDelegationReferenceExpression `is_this`; KtAnonymousInitializer (+class/script
//!   initializer) `body`; KtTypeAlias `type_alias_keyword`, `type_reference`; KtTypeParameterList
//!   `parameters`, `trailing_comma`; KtTypeParameter `extends_bound`; KtTypeConstraintList `constraints`;
//!   KtTypeConstraint `subject_type_parameter_name`, `bound_type_reference`.
//! - KtTypeReference `type_element`, `has_parentheses`; KtNullableType `inner_type`,
//!   `question_mark_node`; KtUserType `qualifier`, `reference_expression`, `type_argument_list`,
//!   `type_arguments`, `referenced_name`; KtIntersectionType `{left,right}_type_ref`; KtTypeArgumentList
//!   `arguments`, `trailing_comma`; KtTypeProjection `projection_kind` ([`KtProjectionKind`]),
//!   `projection_token`, `type_reference`; KtFunctionType `parameter_list`, `parameters`, `receiver`,
//!   `receiver_type_reference`, `context_receiver_list`, `context_parameter_list`,
//!   `return_type_reference`; KtFunctionTypeReceiver `type_reference`.
//! - KtModifierList `annotations`, `annotation_entries`, `context_parameter_list`,
//!   `context_receiver_list`, `has_modifier`, `modifier`; KtFileAnnotationList `annotations`,
//!   `annotation_entries`; KtAnnotation `entries`, `use_site_target`; KtAnnotationEntry `callee_expression`,
//!   `value_argument_list`, `type_reference`, `type_argument_list`, `lambda_arguments`, `at_symbol`,
//!   `use_site_target`; KtAnnotationUseSiteTarget `annotation_use_site_target`
//!   ([`AnnotationUseSiteTarget::render_name`]); KtContextParameterList/KtContextReceiverList
//!   `context_parameters`, `context_receivers`, `type_references`; KtContextReceiver `type_reference`.
//! - KtQualifiedExpression (+dot, safe) `receiver_expression`, `selector_expression`,
//!   `operation_token_node`, `operation_sign`; KtCallExpression `callee_expression`, `value_argument_list`,
//!   `type_argument_list`, `lambda_arguments`, `value_arguments`; KtValueArgumentList `arguments`,
//!   `trailing_comma`, `{left,right}_parenthesis`; KtValueArgument/KtLambdaArgument `argument_expression`,
//!   `argument_name`, `equals_token`, `is_named`, `spread_element`, `is_spread`; KtValueArgumentName
//!   `reference_expression`; KtLambdaExpression `function_literal`, `value_parameters`, `body_expression`,
//!   `{left,right}_curly_brace`; KtFunctionLiteral `arrow`, `l_brace`, `r_brace`,
//!   `has_parameter_specification`; KtSimpleNameExpression (+name/operation/label/enum-superclass
//!   references) `identifier`, `referenced_name_element`, `referenced_name`, `referenced_name_element_type`;
//!   KtOperationReferenceExpression `operation_sign_token_type`.
//! - KtUnaryExpression (+prefix, postfix) `operation_reference`, `operation_token`, `base_expression`;
//!   KtBinaryExpression `left`, `right`, `operation_reference`, `operation_token`; KtExpressionWithLabel
//!   (+return, break, continue, this, super, labeled) `target_label`, `label_qualifier`, `label_name`;
//!   KtReturnExpression `returned_expression`; KtLabeledExpression `base_expression`; KtSuperExpression
//!   `super_type_qualifier`; KtArrayAccessExpression `array_expression`, `index_expressions`,
//!   `indices_node`, `{left,right}_bracket`, `trailing_comma`; KtParenthesizedExpression `expression`;
//!   KtAnnotatedExpression `base_expression`, `annotations`, `annotation_entries`; KtDoubleColonExpression
//!   (+callable reference, class literal) `receiver_expression`, `has_question_marks`,
//!   `find_colon_colon`; KtCallableReferenceExpression `callable_reference`; KtCollectionLiteralExpression
//!   `inner_expressions`, `{left,right}_bracket`, `trailing_comma`; KtContainerNodeForControlStructureBody
//!   `expression`.
//! - KtWhenExpression `entries`, `subject_expression`, `subject_variable`; KtWhenEntry `is_else`,
//!   `else_keyword`, `expression`, `conditions`, `guard`, `arrow`, `trailing_comma`; KtWhenEntryGuard and
//!   KtWhenConditionWithExpression `expression`; KtWhenConditionIsPattern `is_negated`, `type_reference`;
//!   KtWhenConditionInRange `operation_reference`, `is_negated`, `range_expression`; KtIfExpression
//!   `condition`, `then`, `r#else`, `else_keyword`, `if_keyword`, `{left,right}_parenthesis`;
//!   KtLoopExpression (+for, while, do-while) `body`, `{left,right}_parenthesis`; KtWhileExpressionBase
//!   (+while, do-while) `condition`; KtForExpression `loop_parameter`, `loop_range`,
//!   `destructuring_declaration`; KtTryExpression `try_block`, `catch_clauses`, `finally_block`;
//!   KtCatchClause `parameter_list`, `catch_parameter`, `catch_body`; KtFinallySection `final_expression`;
//!   KtThrowExpression `thrown_expression`; KtIsExpression `left_hand_side`, `type_reference`,
//!   `operation_reference`; KtBinaryExpressionWithTypeRHS `left`, `right`, `operation_reference`.
//! - KDocName `qualifier`, `name_text_range`, `name_text`, `qualified_name`; KDocImpl/KDocSection/KDocTag/
//!   KDocLink are traversed with `get_children_of_type`.

mod cast;
mod classes;
mod element;
mod element_text;
mod kt;
mod tokens;
mod tree_util;
mod types;
mod visitor;

pub use cast::PsiType;
pub use classes::psi_class_name;
pub use element::{AstNode, PsiElement};
pub use kt::*;
pub use tokens::*;
pub use tree_util::{
    get_next_sibling_of_type, get_prev_sibling_of_type, get_trailing_comma_by_closing_element,
    get_trailing_comma_by_elements_list, try_flatten_string_concatenation_descendants,
};
pub use types::*;
pub use visitor::{KtVisitorVoid, kt_tree_visitor_void, kt_visitor_void};
