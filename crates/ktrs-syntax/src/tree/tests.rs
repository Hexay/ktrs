use super::{Tree, TreeBuilder};
use crate::SyntaxKind::{self, *};

/// `CALL_EXPRESSION(REFERENCE_EXPRESSION(IDENTIFIER "f") VALUE_ARGUMENT_LIST(LPAR "(" RPAR ")")) WHITE_SPACE " "`
/// under a `BLOCK`, plus an empty `VALUE_ARGUMENT_LIST` node at the end.
fn sample() -> Tree {
    let mut b = TreeBuilder::new();
    b.start_node(BLOCK);
    b.start_node(CALL_EXPRESSION);
    b.start_node(REFERENCE_EXPRESSION);
    b.token(IDENTIFIER, "f");
    b.finish_node();
    b.start_node(VALUE_ARGUMENT_LIST);
    b.token(LPAR, "(");
    b.token(RPAR, ")");
    b.finish_node();
    b.finish_node();
    b.token(WHITE_SPACE, " ");
    b.start_node(VALUE_ARGUMENT_LIST);
    b.finish_node();
    b.finish_node();
    b.finish()
}

fn kinds(tree: &Tree, ids: impl Iterator<Item = u32>) -> Vec<SyntaxKind> {
    ids.map(|e| tree.kind(e)).collect()
}

#[test]
fn navigation() {
    let t = sample();
    assert_eq!(t.text(), "f() ");
    assert_eq!(kinds(&t, t.children(Tree::ROOT)), [CALL_EXPRESSION, WHITE_SPACE, VALUE_ARGUMENT_LIST]);
    let call = t.first_child(Tree::ROOT).unwrap();
    assert_eq!(kinds(&t, t.children(call)), [REFERENCE_EXPRESSION, VALUE_ARGUMENT_LIST]);
    assert_eq!(t.last_child(call).map(|e| t.kind(e)), Some(VALUE_ARGUMENT_LIST));
    assert_eq!(t.last_child(Tree::ROOT).map(|e| t.kind(e)), Some(VALUE_ARGUMENT_LIST));
    let args = t.last_child(call).unwrap();
    assert_eq!(t.prev_sibling(args).map(|e| t.kind(e)), Some(REFERENCE_EXPRESSION));
    assert_eq!(t.prev_sibling(call), None);
    assert_eq!(t.next_sibling(args), None);
    assert_eq!(t.parent(args), Some(call));
    assert_eq!(t.text_of(args), "()");
    assert_eq!(t.text_of(call), "f()");
}

#[test]
fn extract_then_push_tree_round_trips() {
    let t = sample();
    let call = t.first_child(Tree::ROOT).unwrap();

    // Rebuild `sample` with the call copied out of a builder and spliced back in as a block.
    let mut source = TreeBuilder::new();
    source.start_node(BLOCK);
    source.token(WHITE_SPACE, "  ");
    let root = source.len();
    source.push_subtree(&t, call);
    let call_tree = source.extract(root);
    assert_eq!(call_tree.text(), "f()");
    assert_eq!(call_tree.parent(Tree::ROOT), None);

    let mut b = TreeBuilder::new();
    b.start_node(BLOCK);
    b.push_tree(&call_tree);
    b.token(WHITE_SPACE, " ");
    b.start_node(VALUE_ARGUMENT_LIST);
    b.finish_node();
    b.finish_node();
    assert_eq!(b.finish(), t);
}

#[test]
fn tokens_and_empty_nodes() {
    let t = sample();
    let empty = t.last_child(Tree::ROOT).unwrap();
    assert!(!t.is_token(empty));
    assert_eq!(t.first_child(empty), None);
    assert_eq!(t.text_of(empty), "");
    let space = t.prev_sibling(empty).unwrap();
    assert!(t.is_token(space));
    assert_eq!(t.text_of(space), " ");
    assert!(t.has_descendant_of_kind(Tree::ROOT, RPAR));
    assert!(!t.has_descendant_of_kind(empty, RPAR));
}

#[test]
fn find_kinds_matches_a_filter_across_chunks() {
    let mut b = TreeBuilder::new();
    b.start_node(BLOCK);
    for i in 0..150 {
        b.start_node(if i % 7 == 0 { VALUE_ARGUMENT_LIST } else { CALL_EXPRESSION });
        b.token(if i % 5 == 0 { COMMA } else { IDENTIFIER }, "x");
        b.finish_node();
    }
    b.finish_node();
    let t = b.finish();
    let first_call = t.first_child(Tree::ROOT).unwrap() + 2;
    for start in [Tree::ROOT, first_call] {
        let expected: Vec<u32> = (start..t.subtree_end(start))
            .filter(|&e| {
                (!t.is_token(e) && t.kind(e) == VALUE_ARGUMENT_LIST) || (t.is_token(e) && t.kind(e) == COMMA)
            })
            .collect();
        let found: Vec<u32> = t.find_kinds(start, [VALUE_ARGUMENT_LIST], [COMMA]).collect();
        assert_eq!(found, expected);
        assert!(!found.is_empty() || start != Tree::ROOT);
    }
}
