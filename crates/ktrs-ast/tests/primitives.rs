//! Each primitive against IntelliJ's semantics, hand-derived from the platform classes in
//! kotlin-compiler-embeddable-2.4.20.jar (decompiled `org.jetbrains.kotlin.com.intellij.psi.impl.source.tree.*`;
//! the method named in each test is the cite).

use ktrs_ast::{Ast, NodeId, tree_util};
use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::SyntaxKind::{self, *};

fn ast(text: &str) -> Ast {
    Ast::from_parse(&parse_file(text, FileKind::Source))
}

/// The `nth` node of `kind` in preorder.
fn find(ast: &Ast, kind: SyntaxKind, nth: usize) -> NodeId {
    let mut found = Vec::new();
    walk(ast, ast.root(), &mut |n| {
        if ast.element_type(n) == kind {
            found.push(n)
        }
    });
    found[nth]
}

fn walk(ast: &Ast, n: NodeId, f: &mut impl FnMut(NodeId)) {
    f(n);
    let mut c = ast.first_child_node(n);
    while let Some(child) = c {
        walk(ast, child, f);
        c = ast.tree_next(child);
    }
}

fn kinds(ast: &Ast, parent: NodeId) -> Vec<SyntaxKind> {
    let mut out = Vec::new();
    ast.get_children(parent, &mut out);
    out.into_iter().map(|c| ast.element_type(c)).collect()
}

fn root_text(ast: &Ast) -> String {
    ast.text(ast.root())
}

#[test]
fn remove_child_parks_the_child_in_a_dummy_holder_and_keeps_both_whitespaces() {
    // CompositeElement.removeChild -> removeChildInner -> removeChildrenInner: remove + repairRemovedElement.
    let mut a = ast("fun f() {a() ; b()}");
    let semi = find(&a, SEMICOLON, 0);
    let parent = a.tree_parent(semi).unwrap();
    let (before, after) = (a.tree_prev(semi).unwrap(), a.tree_next(semi).unwrap());
    a.remove_child(parent, semi);
    let holder = a.tree_parent(semi).expect("removed node is not orphaned");
    assert_eq!(a.element_type(holder), DUMMY_HOLDER);
    assert_eq!(a.tree_parent(holder), None);
    assert_eq!((a.tree_prev(semi), a.tree_next(semi)), (None, None));
    assert_eq!(a.start_offset(semi), 0);
    assert_eq!(root_text(&a), "fun f() {a()  b()}");
    assert_eq!((a.element_type(before), a.element_type(after)), (WHITE_SPACE, WHITE_SPACE));
    assert_eq!(a.tree_next(before), Some(after), "no whitespace merging");
    assert_eq!(a.start_offset(find(&a, CALL_EXPRESSION, 1)), 14);
    assert_eq!(a.text_length(a.root()), 18);
}

#[test]
fn add_child_moves_an_attached_node_alone() {
    // CompositeElement.addChild: removeChildrenInner(child, child.getTreeNext()) first, then insertBefore/add.
    let mut a = ast("fun f() { a(1, 2) }");
    let list = find(&a, VALUE_ARGUMENT_LIST, 0);
    let first_arg = find(&a, VALUE_ARGUMENT, 0);
    let rpar = a.find_child_by_type(list, RPAR).unwrap();
    a.add_child(list, first_arg, Some(rpar));
    assert_eq!(kinds(&a, list), [LPAR, COMMA, WHITE_SPACE, VALUE_ARGUMENT, VALUE_ARGUMENT, RPAR]);
    assert_eq!(root_text(&a), "fun f() { a(, 21) }");
    assert_eq!(a.start_offset(first_arg), 15);

    let lpar = a.find_child_by_type(list, LPAR).unwrap();
    a.add_child(list, lpar, None);
    assert_eq!(a.last_child_node(list), Some(lpar));
    assert_eq!(root_text(&a), "fun f() { a, 21)( }");
}

#[test]
fn replace_child_then_fill_the_new_composite() {
    // CompositeElement.replaceChild: replace (rawReplaceWithList) + repairRemovedElement(old); the new
    // composite is attached, so its own addChild calls find a file element.
    let mut a = ast("fun f() { if (c) g() }");
    let then = find(&a, THEN, 0);
    let call = a.first_child_node(then).unwrap();
    let block = a.new_composite(BLOCK);
    a.replace_child(then, call, block);
    assert_eq!(a.element_type(a.tree_parent(call).unwrap()), DUMMY_HOLDER);
    let lbrace = a.new_leaf(LBRACE, "{");
    a.add_child(block, lbrace, None);
    a.add_child(block, call, None);
    let rbrace = a.new_leaf(RBRACE, "}");
    a.add_child(block, rbrace, None);
    assert_eq!(root_text(&a), "fun f() { if (c) {g()} }");
    assert_eq!(a.tree_parent(call), Some(block));
    assert_eq!(a.start_offset(rbrace), 21);
}

#[test]
#[should_panic(expected = "NullPointerException")]
fn add_child_to_a_detached_composite_throws() {
    // ChangeUtil.prepareAndRunChangeAction dereferences TreeUtil.getFileElement(changedElement).
    let mut a = ast("val x = 1");
    let block = a.new_composite(BLOCK);
    let lbrace = a.new_leaf(LBRACE, "{");
    a.add_child(block, lbrace, None);
}

#[test]
fn raw_replace_with_text_swaps_in_a_new_node_and_orphans_the_old_one() {
    // LeafElement.rawReplaceWithText -> ASTFactory.leaf + rawReplaceWithList -> rawRemove -> invalidate.
    let mut a = ast("val x  = 1");
    let ws = find(&a, WHITE_SPACE, 1);
    let new = a.raw_replace_with_text(ws, " ");
    assert_ne!(new, ws);
    assert_eq!((a.tree_parent(ws), a.tree_prev(ws), a.tree_next(ws)), (None, None, None));
    assert_eq!(a.leaf_text(ws), "  ", "the orphan keeps its text");
    assert_eq!(a.element_type(new), WHITE_SPACE);
    assert_eq!(root_text(&a), "val x = 1");
    assert_eq!(a.start_offset(find(&a, EQ, 0)), 6);
}

#[test]
fn raw_insert_before_me_moves_the_whole_following_chain() {
    // TreeElement.rawInsertBeforeMe -> rawInsertAfterMe(anchorPrev) -> rawRemoveUpToWithoutNotifications(null):
    // firstNew and every later sibling of it move.
    let mut a = ast("fun f() { a(1, 2) }");
    let list = find(&a, VALUE_ARGUMENT_LIST, 0);
    let comma = a.find_child_by_type(list, COMMA).unwrap();
    let lpar = a.find_child_by_type(list, LPAR).unwrap();
    let block = find(&a, BLOCK, 0);
    let lbrace = a.find_child_by_type(block, LBRACE).unwrap();
    a.raw_insert_before_me(lbrace, comma);
    assert_eq!(kinds(&a, list), [LPAR, VALUE_ARGUMENT]);
    assert_eq!(a.tree_next(lpar).map(|n| a.element_type(n)), Some(VALUE_ARGUMENT));
    assert_eq!(root_text(&a), "fun f() , 2){ a(1 }");
    assert_eq!(a.first_child_node(block).map(|n| a.element_type(n)), Some(COMMA));
}

#[test]
fn raw_remove_leaves_no_parent() {
    // TreeElement.rawRemove -> invalidate: no dummy holder on the raw path.
    let mut a = ast("val x = 1");
    let eq = find(&a, EQ, 0);
    a.raw_remove(eq);
    assert_eq!(a.tree_parent(eq), None);
    assert_eq!(root_text(&a), "val x  1");
}

#[test]
fn remove_range_and_add_children() {
    // CompositeElement.removeRange -> removeChildrenInner(first, firstWhichStayInTree);
    // addChildren = one addChild per node, each moved alone.
    let mut a = ast("fun f() { a(1, 2) }");
    let list = find(&a, VALUE_ARGUMENT_LIST, 0);
    let comma = a.find_child_by_type(list, COMMA).unwrap();
    let rpar = a.find_child_by_type(list, RPAR).unwrap();
    a.remove_range(list, comma, Some(rpar));
    assert_eq!(kinds(&a, list), [LPAR, VALUE_ARGUMENT, RPAR]);
    let holder = a.tree_parent(comma).unwrap();
    assert_eq!(kinds(&a, holder), [COMMA, WHITE_SPACE, VALUE_ARGUMENT]);
    a.add_children(list, comma, None, Some(rpar));
    assert_eq!(kinds(&a, list), [LPAR, VALUE_ARGUMENT, COMMA, WHITE_SPACE, VALUE_ARGUMENT, RPAR]);
    assert_eq!(root_text(&a), "fun f() { a(1, 2) }");
}

#[test]
fn clone_is_deep_and_detached() {
    // CompositeElement.clone: super.clone (links cleared) + a clone of every child.
    let mut a = ast("fun f() { a(1, 2) }");
    let list = find(&a, VALUE_ARGUMENT_LIST, 0);
    let copy = a.clone(list);
    assert_ne!(copy, list);
    assert_eq!(a.tree_parent(copy), None);
    assert_eq!(a.text(copy), "(1, 2)");
    assert_eq!(kinds(&a, copy), kinds(&a, list));
    assert_ne!(a.first_child_node(copy), a.first_child_node(list));
}

#[test]
fn tree_util_leaf_walks_skip_empty_composites() {
    // TreeUtil.nextLeaf/prevLeaf descend through composites (findFirstLeafOrType) and only return LeafElements.
    let a = ast("@A class B");
    let at = find(&a, AT, 0);
    let class_kw = find(&a, CLASS_KEYWORD, 0);
    let leaves: Vec<_> = a.leaves(at, true).map(|n| a.element_type(n)).collect();
    assert_eq!(leaves, [IDENTIFIER, WHITE_SPACE, CLASS_KEYWORD, WHITE_SPACE, IDENTIFIER]);
    assert_eq!(tree_util::prev_leaf(&a, class_kw).map(|n| a.element_type(n)), Some(WHITE_SPACE));
    let mut b = ast("val x = 1");
    let empty = b.new_composite(BLOCK);
    let eq = find(&b, EQ, 0);
    let prop = b.tree_parent(eq).unwrap();
    b.add_child(prop, empty, Some(eq));
    assert_eq!(tree_util::next_leaf(&b, find(&b, WHITE_SPACE, 1)).map(|n| b.element_type(n)), Some(EQ));
}

#[test]
fn create_ast_node_from_text_returns_the_script_initializer() {
    // KtlintKotlinCompiler.createASTNodeFromText: File.kts > SCRIPT > BLOCK > SCRIPT_INITIALIZER.
    let mut a = ast("val x = 1");
    let node = a.create_ast_node_from_text("foo(1)").unwrap();
    assert_eq!(a.element_type(node), SCRIPT_INITIALIZER);
    assert_eq!(a.text(node), "foo(1)");
    let call = a.first_child_node(node).unwrap();
    let prop = find(&a, PROPERTY, 0);
    a.add_child(prop, call, None);
    assert_eq!(root_text(&a), "val x = 1foo(1)");
}
