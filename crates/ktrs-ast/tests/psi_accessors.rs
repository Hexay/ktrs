//! The PSI accessors compose-rules uses, against the compiler's PSI (Kotlin 2.2.21 / 2.4.10 sources, AST paths).

mod common;

use common::{ast, find};
use ktrs_ast::psi::{self, *};
use ktrs_syntax::SyntaxKind::*;

#[test]
fn annotation_entries_and_modifiers() {
    // KtModifierListOwnerStub.getAnnotationEntries = modifierList.annotationEntries (entries and @[...] entries);
    // KtFile's come from the file annotation list; a type reference has its own modifier list.
    let a = ast("@file:Suppress(\"X\")\n@Composable @[A B] private fun f(c: @Composable () -> Unit) {}");
    let f = KtNamedFunction::of(&a, find(&a, FUN, 0));
    let callees: Vec<String> = f.annotation_entries(&a).iter().map(|e| a.text(e.callee_expression(&a).unwrap())).collect();
    assert_eq!(callees, ["Composable", "A", "B"]);
    assert_eq!(f.visibility_modifier_type(&a), Some(PRIVATE_KEYWORD));
    assert!(!f.is_public(&a) && f.has_modifier(&a, PRIVATE_KEYWORD));
    let file = KtFile::of(&a, a.root());
    assert_eq!(file.annotation_entries(&a).len(), 1);
    let type_ref = f.value_parameters(&a)[0].type_reference(&a).unwrap();
    assert_eq!(type_ref.annotation_entries(&a).len(), 1);
    assert!(type_ref.type_element(&a).unwrap().as_function_type(&a).is_some());
}

#[test]
fn is_public_excludes_local_declarations() {
    // isPublic: KtPsiUtil.isLocal(declaration) -> false, else no visibility modifier or `public`.
    let a = ast("fun top() { fun local() {} }\nclass C { internal fun member() {} public fun p() {} }");
    assert!(KtNamedFunction::of(&a, find(&a, FUN, 0)).is_public(&a));
    let local = KtNamedFunction::of(&a, find(&a, FUN, 1));
    assert!(local.is_local(&a) && psi::is_local(&a, local.node()) && !local.is_public(&a));
    assert!(!KtNamedFunction::of(&a, find(&a, FUN, 2)).is_public(&a));
    assert!(KtNamedFunction::of(&a, find(&a, FUN, 3)).is_public(&a));
}

#[test]
fn names() {
    // getName: unquoted identifier; constructors take the class name, function literals "<anonymous>", a companion
    // without a name "Companion"; getNameAsSafeName falls back to "<no name provided>".
    let a = ast("class `My C` constructor(val p: Int) { companion object {} }\nval l = { x: Int -> x }\nval f = fun() {}");
    assert_eq!(KtClass::of(&a, find(&a, CLASS, 0)).name(&a).as_deref(), Some("My C"));
    assert_eq!(KtPrimaryConstructor::of(&a, find(&a, PRIMARY_CONSTRUCTOR, 0)).name(&a).as_deref(), Some("My C"));
    assert_eq!(psi::name_identifier(&a, find(&a, PRIMARY_CONSTRUCTOR, 0)), None);
    assert_eq!(KtObjectDeclaration::of(&a, find(&a, OBJECT_DECLARATION, 0)).name(&a).as_deref(), Some("Companion"));
    assert_eq!(KtFunctionLiteral::of(&a, find(&a, FUNCTION_LITERAL, 0)).name(&a).as_deref(), Some(ANONYMOUS_STRING));
    let anonymous = KtNamedFunction::of(&a, find(&a, FUN, 0));
    assert_eq!(anonymous.name_as_safe_name(&a), NO_NAME_PROVIDED);
    assert!(psi::is_name_identifier_owner(&a, find(&a, VALUE_PARAMETER, 0)));
    assert!(!psi::is_name_identifier_owner(&a, find(&a, LAMBDA_EXPRESSION, 0)));
}

#[test]
fn callable_and_body_accessors() {
    // TypeRefHelpers.getTypeReference: first type reference after ':'; receiver before '(' (functions).
    // hasBlockBody: no '=' (a function literal never has one: false).
    let a = ast("fun Modifier.f(m: Modifier = Modifier): Int = 1\nval g = { (a, b): P -> }\nvar v: Int = 2");
    let f = KtNamedFunction::of(&a, find(&a, FUN, 0));
    assert_eq!(a.text(f.type_reference(&a).unwrap().node()), "Int");
    assert_eq!(a.text(f.receiver_type_reference(&a).unwrap().node()), "Modifier");
    assert!(!f.has_block_body(&a) && f.has_body(&a) && f.body_block_expression(&a).is_none());
    let params = f.value_parameters(&a);
    assert!(params[0].has_default_value(&a));
    assert_eq!(a.text(params[0].default_value(&a).unwrap()), "Modifier");
    let literal = KtFunctionLiteral::of(&a, find(&a, FUNCTION_LITERAL, 0));
    assert!(!literal.has_block_body(&a));
    let destructured = literal.value_parameters(&a)[0];
    assert!(destructured.name(&a).is_none() && destructured.is_lambda_parameter(&a));
    let entries = destructured.destructuring_declaration(&a).unwrap().entries(&a);
    assert_eq!(entries.iter().map(|e| e.name(&a).unwrap()).collect::<Vec<_>>(), ["a", "b"]);
    let v = KtProperty::of(&a, find(&a, PROPERTY, 1));
    assert!(v.is_var(&a) && v.has_initializer(&a) && !v.is_local(&a));
}

#[test]
fn calls_and_arguments() {
    // getValueArguments: parenthesized then lambda arguments; KtValueArgument.getName() is PsiElementBase's null.
    let a = ast("val x = Row(modifier = Modifier.a().b(), 1) @L { Text(\"a\") }\nval y = s?.let { it }");
    let call = KtCallExpression::of(&a, find(&a, CALL_EXPRESSION, 0));
    assert_eq!(a.text(call.callee_expression(&a).unwrap()), "Row");
    let arguments = call.value_arguments(&a);
    assert_eq!(arguments.len(), 3);
    assert!(arguments[0].is_named(&a) && arguments[0].name(&a).is_none());
    assert_eq!(arguments[0].argument_name(&a).unwrap().as_name(&a).as_deref(), Some("modifier"));
    let dot = KtDotQualifiedExpression::of(&a, arguments[0].argument_expression(&a).unwrap());
    assert_eq!(a.text(dot.receiver_expression(&a).unwrap()), "Modifier.a()");
    let lambda = call.lambda_arguments(&a)[0].lambda_expression(&a).expect("unpacked from the annotated expression");
    assert_eq!(lambda.body_expression(&a).unwrap().statements(&a).len(), 1);
    let text_call = find(&a, CALL_EXPRESSION, 3);
    assert_eq!(a.text(psi::reference_expression(&a, text_call).unwrap().node()), "Text");
    let b = find(&a, CALL_EXPRESSION, 2);
    assert!(psi::is_dot_selector(&a, b) && !psi::is_dot_selector(&a, text_call));
    let safe = KtSafeQualifiedExpression::of(&a, find(&a, SAFE_ACCESS_EXPRESSION, 0));
    assert_eq!(a.text(safe.receiver_expression(&a).unwrap()), "s");
}

#[test]
fn types() {
    // KtNullableType.innerType; KtUserType.referencedName (last segment); KtFunctionType.returnTypeReference.
    let a = ast("fun f(a: (() -> Unit)?, b: a.b.Foo<Int>?, c: Int.() -> String) {}");
    let params = KtNamedFunction::of(&a, find(&a, FUN, 0)).value_parameters(&a);
    let nullable = params[0].type_reference(&a).unwrap().type_element(&a).unwrap().as_nullable_type(&a).unwrap();
    assert!(nullable.inner_type(&a).unwrap().as_function_type(&a).is_some());
    let user = params[1].type_reference(&a).unwrap().type_element(&a).unwrap().as_nullable_type(&a).unwrap();
    let user = user.inner_type(&a).unwrap().as_user_type(&a).unwrap();
    assert_eq!(user.referenced_name(&a).as_deref(), Some("Foo"));
    let function = params[2].type_reference(&a).unwrap().type_element(&a).unwrap().as_function_type(&a).unwrap();
    assert_eq!(a.text(function.return_type_reference(&a).unwrap().node()), "String");
}

#[test]
fn class_accessors_and_get_children() {
    // KtClass.isInterface (keyword child), isAnnotation (modifier); KtClassBody.functions; getChildren lists only
    // composites, except for files and lazy-parseable elements.
    let a = ast("fun interface I { @Composable fun c() }\nannotation class A");
    let i = KtClass::of(&a, find(&a, CLASS, 0));
    assert!(i.is_interface(&a) && i.has_modifier(&a, FUN_KEYWORD));
    assert_eq!(i.body(&a).unwrap().functions(&a).len(), 1);
    assert!(KtClass::of(&a, find(&a, CLASS, 1)).is_annotation(&a));
    assert!(psi::children(&a, i.node()).iter().all(|&c| !a.is_leaf_element(c)));
    assert!(psi::children(&a, a.root()).iter().any(|&c| a.is_leaf_element(c)));
}

#[test]
fn factory_create_parameter_and_set_name() {
    // KtPsiFactory.createParameter = createClass("class A($text)").primaryConstructorParameters.first();
    // setName replaces the name identifier with createNameIdentifierIfPossible(quoteIfNeeded(name)).
    let mut a = ast("@Composable fun Foo(modifier: Modifier) {}");
    let parameter = find(&a, VALUE_PARAMETER, 0);
    let text = format!("{} = Modifier", a.text(parameter));
    let new_parameter = psi::kt_psi_factory::create_parameter(&mut a, &text);
    let parent = a.tree_parent(parameter).unwrap();
    a.replace_child(parent, parameter, new_parameter.node());
    let function = KtNamedFunction::of(&a, find(&a, FUN, 0));
    function.set_name(&mut a, "FooPreview", EmbeddedKotlin::V2_4_10);
    assert_eq!(a.text(a.root()), "@Composable fun FooPreview(modifier: Modifier = Modifier) {}");
    function.set_name(&mut a, "in", EmbeddedKotlin::V2_4_10);
    assert_eq!(function.name_identifier(&a).map(|n| a.text(n)).as_deref(), Some("`in`"));
}

#[test]
fn quote_if_needed_differs_on_latin1_letters() {
    // isIdentifier: 2.2.21 accepts only ASCII letters below U+0100, 2.4.10 any Character.isLetter.
    assert_eq!(psi::quote_if_needed("caféPreview", EmbeddedKotlin::V2_2_21), "`caféPreview`");
    assert_eq!(psi::quote_if_needed("caféPreview", EmbeddedKotlin::V2_4_10), "caféPreview");
    assert_eq!(psi::quote_if_needed("1a", EmbeddedKotlin::V2_4_10), "`1a`");
    assert!(psi::is_identifier("`a b`", EmbeddedKotlin::V2_2_21) && !psi::is_identifier("`a", EmbeddedKotlin::V2_2_21));
}

#[test]
fn expressions() {
    // KtIfExpression.then/else; KtBinaryExpression.operationToken; KtReturnExpression.labeledExpression.
    let a = ast("fun f() { val x = if (c) a else b ?: d\n return@f }");
    let r#if = KtIfExpression::of(&a, find(&a, IF, 0));
    assert_eq!(a.text(r#if.then(&a).unwrap()), "a");
    let elvis = KtBinaryExpression::of(&a, r#if.r#else(&a).unwrap());
    assert_eq!(elvis.operation_token(&a), Some(ELVIS));
    assert_eq!(a.text(elvis.right(&a).unwrap()), "d");
    let ret = KtReturnExpression::of(&a, find(&a, RETURN, 0));
    assert!(ret.labeled_expression(&a).is_some() && ret.returned_expression(&a).is_none());
}
