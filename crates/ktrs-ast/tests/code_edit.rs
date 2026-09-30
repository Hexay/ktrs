//! `CodeEditUtil`'s removal path and `PsiElement.delete()`, against the bytecode of the ktlint 2.0.0-ALPHA-4
//! fat jar (Kotlin 2.4.10 inside; `CodeEditUtil` is byte-identical to 2.4.20's). The method named in each
//! test is the cite.

mod common;

use common::{ast, find, kinds, root_text};
use ktrs_ast::{code_edit_util, psi};
use ktrs_syntax::SyntaxKind::*;

#[test]
fn leaf_delete_between_tokens_that_may_touch_leaves_no_whitespace() {
    // LeafPsiElement.delete -> CompositeElement.deleteChildInternal -> CodeEditUtil.removeChild; then
    // makePlaceHolderBetweenTokens(COMMA, `2`): spaceExistanceTypeBetweenTokens = MAY, so nothing is inserted;
    // finally invalidate() drops the leaf's parent.
    let mut a = ast("fun f() { a(1,  2) }");
    let ws = find(&a, WHITE_SPACE, 3);
    assert_eq!(a.leaf_text(ws), "  ");
    psi::delete(&mut a, ws);
    assert_eq!(root_text(&a), "fun f() { a(1,2) }");
    assert_eq!(a.tree_parent(ws), None);
}

#[test]
fn removing_the_space_between_two_keywords_inserts_a_new_one() {
    // makePlaceHolderBetweenTokens -> markToReformatBeforeOrInsertWhitespace: both KtKeywordTokens -> MUST ->
    // " ", added before TreeUtil.prevLeaf(right, state).nextLeafBranchStart (here `fun` itself).
    let mut a = ast("private fun f() {}");
    let fun = find(&a, FUN, 0);
    let ws = a.tree_next(a.find_child_by_type(fun, MODIFIER_LIST).unwrap()).unwrap();
    code_edit_util::remove_child(&mut a, fun, ws);
    assert_eq!(root_text(&a), "private fun f() {}");
    let new_ws = a.tree_prev(a.find_child_by_type(fun, FUN_KEYWORD).unwrap()).unwrap();
    assert_ne!(new_ws, ws);
    assert_eq!(a.element_type(a.tree_parent(ws).unwrap()), DUMMY_HOLDER);
}

#[test]
fn get_keyword_after_a_removed_space_gets_a_line_break() {
    // spaceExistanceTypeBetweenTokens: right is GET_KEYWORD -> MUST_LINE_BREAK -> "\n", inserted before the
    // PROPERTY_ACCESSOR (the branch start of `get`).
    let mut a = ast("val x: Int get() = 1");
    let accessor = find(&a, PROPERTY_ACCESSOR, 0);
    let property = a.tree_parent(accessor).unwrap();
    let ws = a.tree_prev(accessor).unwrap();
    code_edit_util::remove_child(&mut a, property, ws);
    assert_eq!(root_text(&a), "val x: Int\nget() = 1");
    assert_eq!(a.tree_parent(a.tree_prev(accessor).unwrap()), Some(property));
}

#[test]
fn whitespaces_either_side_merge_into_a_new_one() {
    // makePlaceHolderBetweenTokens, both WHITE_SPACE, 0 blank lines each: text = left + right; forceReformat
    // (needToForceReformat: `;` does not start its block) replaces left with the merged leaf, removes right.
    let mut a = ast("fun f() {a() ; b()}");
    let semi = find(&a, SEMICOLON, 0);
    let (left, right) = (a.tree_prev(semi).unwrap(), a.tree_next(semi).unwrap());
    psi::delete(&mut a, semi);
    assert_eq!(root_text(&a), "fun f() {a()  b()}");
    let merged = a.tree_next(find(&a, CALL_EXPRESSION, 0)).unwrap();
    assert!(merged != left && merged != right);
    assert_eq!(a.element_type(a.tree_parent(right).unwrap()), DUMMY_HOLDER);
}

#[test]
fn more_blank_lines_on_the_right_replace_both_with_a_copy_of_the_right() {
    // leftBlankLines < rightBlankLines: right is replaced by ASTFactory.whitespace(right text), left removed.
    let mut a = ast("fun f() {\n  a()\n  ;\n\n  b()\n}");
    let semi = find(&a, SEMICOLON, 0);
    let (left, right) = (a.tree_prev(semi).unwrap(), a.tree_next(semi).unwrap());
    psi::delete(&mut a, semi);
    assert_eq!(root_text(&a), "fun f() {\n  a()\n\n  b()\n}");
    let merged = a.tree_next(find(&a, CALL_EXPRESSION, 0)).unwrap();
    assert!(merged != left && merged != right);
    assert_eq!(a.leaf_text(merged), "\n\n  ");
}

#[test]
fn whitespace_after_a_removed_node_is_recreated() {
    // Left not whitespace, right whitespace: markWhitespaceForReformat(right) swaps in an equal new leaf.
    let mut a = ast("val v = foo(a,b )");
    let b = find(&a, VALUE_ARGUMENT, 1);
    let list = a.tree_parent(b).unwrap();
    let ws = a.tree_next(b).unwrap();
    code_edit_util::remove_child(&mut a, list, b);
    assert_eq!(root_text(&a), "val v = foo(a, )");
    assert_eq!(kinds(&a, list), [LPAR, VALUE_ARGUMENT, COMMA, WHITE_SPACE, RPAR]);
    assert_ne!(a.find_child_by_type(list, WHITE_SPACE), Some(ws));
}

#[test]
fn trailing_whitespace_of_a_removed_last_child_goes_too() {
    // removeChildren: tailingElement (the entry ends its MODIFIER_LIST); makePlaceHolderBetweenTokens drops the
    // whitespace that is now the list's last child. KtElementImplStub.delete -> deleteSemicolon (none) first.
    let mut a = ast("@A @B fun f() {}");
    let b = find(&a, ANNOTATION_ENTRY, 1);
    psi::delete(&mut a, b);
    assert_eq!(root_text(&a), "@A fun f() {}");
    assert_eq!(kinds(&a, find(&a, MODIFIER_LIST, 0)), [ANNOTATION_ENTRY]);
}

#[test]
fn deleting_the_last_modifier_deletes_the_modifier_list() {
    // KtModifierList.deleteChildInternal: super, then delete() itself once it has no children left.
    let mut a = ast("@A fun f() {}");
    let fun = find(&a, FUN, 0);
    let list = a.find_child_by_type(fun, MODIFIER_LIST).unwrap();
    let entry = find(&a, ANNOTATION_ENTRY, 0);
    psi::delete(&mut a, entry);
    assert_eq!(a.find_child_by_type(fun, MODIFIER_LIST), None);
    assert_eq!(a.element_type(a.tree_parent(list).unwrap()), DUMMY_HOLDER);
    assert_eq!(root_text(&a), " fun f() {}");
}

#[test]
fn delete_takes_a_following_semicolon_and_its_whitespace() {
    // ktElementUtils.deleteSemicolon: deleteChildRange(nextSibling, skipSiblingsForward(`;`, PsiWhiteSpace)
    // .prevSibling), then the element itself.
    let mut a = ast("fun f() { val a = 1; val b = 2 }");
    let property = find(&a, PROPERTY, 0);
    psi::delete(&mut a, property);
    assert_eq!(root_text(&a), "fun f() { val b = 2 }");
    assert_eq!(kinds(&a, find(&a, BLOCK, 0)), [LBRACE, WHITE_SPACE, PROPERTY, WHITE_SPACE, RBRACE]);
}

#[test]
fn deleting_an_import_leaves_the_newline_after_it() {
    // KtImportDirective.delete: no prev leaf, so makePlaceHolderBetweenTokens(null, ..) only marks.
    let mut a = ast("import a.b\nimport c.d\n\nfun f() {}");
    let import = find(&a, IMPORT_DIRECTIVE, 0);
    psi::delete(&mut a, import);
    assert_eq!(root_text(&a), "\nimport c.d\n\nfun f() {}");
}
