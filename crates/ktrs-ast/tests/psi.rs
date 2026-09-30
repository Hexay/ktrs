//! Typed PSI over the mutable AST, against the compiler's PSI (psi-api sources; the ALPHA-4 jar's 2.4.10
//! bytecode for `KtImportDirective`, `KtNamedFunction`, `KtWhenEntry`, `ImportPath`).

mod common;

use common::{ast, find};
use ktrs_ast::psi::{self, *};
use ktrs_syntax::SyntaxKind::*;

#[test]
fn import_path_renders_like_import_path_to_string() {
    // KtImportDirective.getImportPath: ImportPath(importedFqName, isAllUnder, aliasName?.let(Name::identifier));
    // ImportPath.pathStr = NameRenderingUtils.render(fqName) + ".*"; toString adds " as <alias>".
    // Expected values checked on the JVM.
    let a = ast("import a.b.C as D\nimport x.y.*\nimport p.`q r`.s\nimport `in`.x");
    let aliased = KtImportDirective::of(&a, find(&a, IMPORT_DIRECTIVE, 0));
    let path = aliased.import_path(&a).unwrap();
    assert_eq!((path.path_str(), path.to_string()), ("a.b.C".to_owned(), "a.b.C as D".to_owned()));
    assert_eq!((path.has_alias(), path.imported_name()), (true, Some("D".to_owned())));
    let star = KtImportDirective::of(&a, find(&a, IMPORT_DIRECTIVE, 1)).import_path(&a).unwrap();
    assert!(star.is_all_under);
    assert_eq!((star.path_str(), star.imported_name()), ("x.y.*".to_owned(), None));
    let quoted = KtImportDirective::of(&a, find(&a, IMPORT_DIRECTIVE, 2)).import_path(&a).unwrap();
    assert_eq!(quoted.fq_name.as_string(), "p.q r.s");
    assert_eq!(quoted.path_str(), "p.`q r`.s");
    let keyword = KtImportDirective::of(&a, find(&a, IMPORT_DIRECTIVE, 3)).import_path(&a).unwrap();
    assert_eq!((keyword.fq_name.as_string(), keyword.path_str()), ("in.x", "`in`.x".to_owned()));
}

#[test]
fn package_directive_qualified_name() {
    // KtPackageDirective.getQualifiedName: the package names' referenced names joined by '.'.
    let a = ast("package a.`b`.c\n");
    let package = KtPackageDirective::of(&a, find(&a, PACKAGE_DIRECTIVE, 0));
    assert_eq!(package.qualified_name(&a), "a.b.c");
}

#[test]
fn named_function_accessors() {
    // KtNamedFunction: getTypeReference (TypeRefHelpers: first TYPE_REFERENCE after ':'), hasDeclaredReturnType,
    // getName (unquoted name identifier), getBodyExpression (first KtExpression child).
    let a = ast("fun Foo(): Foo = Foo()\nfun `bar baz`() = 1\nfun <T> T.ext() {}");
    let foo = KtFunction::of(&a, find(&a, FUN, 0));
    assert!(foo.has_declared_return_type(&a));
    assert_eq!(foo.name(&a).as_deref(), Some("Foo"));
    assert_eq!(a.text(foo.type_reference(&a).unwrap().node()), "Foo");
    assert_eq!(foo.body_expression(&a).map(|n| a.element_type(n)), Some(CALL_EXPRESSION));
    let bar = KtFunction::of(&a, find(&a, FUN, 1));
    assert!(!bar.has_declared_return_type(&a));
    assert_eq!(bar.name(&a).as_deref(), Some("bar baz"));
    let ext = KtFunction::of(&a, find(&a, FUN, 2));
    assert!(!ext.has_declared_return_type(&a), "the receiver type is before any ':'");
    assert_eq!(ext.body_expression(&a).map(|n| a.element_type(n)), Some(BLOCK));
}

#[test]
fn when_entry_and_expression() {
    // KtWhenEntry.isElse = elseKeyword != null && guard == null; KtWhenExpression.getLeftParenthesis = LPAR child.
    let a = ast("val v = when (x) { 1 -> a\n else -> b }\nval w = when { else -> c }");
    let entry = KtWhenEntry::of(&a, find(&a, WHEN_ENTRY, 1));
    assert!(entry.is_else(&a));
    assert!(!KtWhenEntry::of(&a, find(&a, WHEN_ENTRY, 0)).is_else(&a));
    let parent = psi::psi_parent(&a, entry.node()).unwrap();
    assert!(KtWhenExpression::of(&a, parent).left_parenthesis(&a).is_some());
    assert!(KtWhenExpression::of(&a, find(&a, WHEN, 1)).left_parenthesis(&a).is_none());
}

#[test]
fn super_type_list_entries_and_dot_qualified_selector() {
    // KtSuperTypeList.getEntries = SUPER_TYPE_LIST_ENTRIES children; KtQualifiedExpression.getSelectorExpression =
    // the first KtExpression sibling after the operation token.
    let a = ast("class A : B(), C by d, E\nval v = a.b(1)");
    let list = KtSuperTypeList::of(&a, find(&a, SUPER_TYPE_LIST, 0));
    let entries: Vec<_> = list.entries(&a).iter().map(|e| a.element_type(e.node())).collect();
    assert_eq!(entries, [SUPER_TYPE_CALL_ENTRY, DELEGATED_SUPER_TYPE_ENTRY, SUPER_TYPE_ENTRY]);
    let dot = KtDotQualifiedExpression::of(&a, find(&a, DOT_QUALIFIED_EXPRESSION, 0));
    assert_eq!(dot.selector_expression(&a), Some(find(&a, CALL_EXPRESSION, 0)));
    assert_eq!(dot.receiver_expression(&a).map(|n| a.text(n)).as_deref(), Some("a"));
}

#[test]
fn class_checks_and_casts() {
    // `psi is KtX` by element type; composites only; KtFile only for the file root (not a dummy holder).
    let a = ast("fun f() = 1");
    let fun = find(&a, FUN, 0);
    assert!(KtFunction::is(&a, fun) && KtDeclaration::is(&a, fun) && KtExpression::is(&a, fun));
    assert!(KtFile::is(&a, a.root()) && KtElement::is(&a, a.root()));
    assert!(PsiWhiteSpace::is(&a, find(&a, WHITE_SPACE, 0)));
    assert_eq!(KtImportDirective::cast(&a, fun), None);
    assert!(panics(|| KtImportDirective::of(&a, fun)));
}

#[test]
fn is_kt_annotated_by_element_type() {
    // ASTNodeExtension.isKtAnnotated: dummyPsiElement() is KtAnnotated (KtModifierListOwner, KtAnnotatedExpression,
    // KtTypeConstraint, KtFile); NotImplementedError for types with no Kt PSI factory.
    let a = ast("fun f(): Int = 1");
    assert!(psi::is_kt_annotated(&a, find(&a, FUN, 0)));
    assert!(psi::is_kt_annotated(&a, find(&a, TYPE_REFERENCE, 0)));
    assert!(psi::is_kt_annotated(&a, a.root()));
    assert!(!psi::is_kt_annotated(&a, find(&a, VALUE_PARAMETER_LIST, 0)));
    assert!(!psi::is_kt_annotated(&a, find(&a, FUN_KEYWORD, 0)));
    let ws = find(&a, WHITE_SPACE, 0);
    assert!(panics(|| psi::is_kt_annotated(&a, ws)));
    let b = ast("fun f() {}");
    assert!(panics(|| psi::is_kt_annotated(&b, find(&b, BLOCK, 0))), "a lazy type in the jar's 2.4.10");
}

fn panics<T>(f: impl FnOnce() -> T) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
}

#[test]
fn virtual_file_path_is_the_light_virtual_file_path() {
    // LightVirtualFileBase.getPath = parent path ("" without a parent) + "/" + name.
    let mut a = ast("val x = 1");
    assert_eq!(KtFile::of(&a, a.root()).virtual_file_path(&a), "/File.kt");
    a.set_psi_file_name("src/Foo.kt");
    let file = KtFile::of(&a, a.root());
    assert_eq!((file.virtual_file_path(&a), file.virtual_file_name(&a)), ("/src/Foo.kt".to_owned(), "src/Foo.kt"));
}
