//! The incrementally kept text hash, UTF-16 surplus and newline count agree with a recomputation from the text, before
//! and after every kind of edit (non-ASCII text, so the UTF-16 paths are taken).

mod common;

use common::{ast, find};
use ktrs_ast::{Ast, NodeId};
use ktrs_syntax::SyntaxKind::{CALL_EXPRESSION, FUN, IDENTIFIER, PROPERTY, WHITE_SPACE};

fn java_hash(text: &str) -> i32 {
    text.encode_utf16().fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(i32::from(c)))
}

fn assert_caches(ast: &Ast) {
    for n in ast.preorder(ast.root()) {
        let text = ast.text(n);
        assert_eq!(ast.text_hash_code(n), java_hash(&text), "hash of {:?} {text:?}", ast.element_type(n));
        assert_eq!(ast.text_length_utf16(n), text.encode_utf16().count(), "utf16 length of {text:?}");
        assert_eq!(ast.text_contains(n, '\n'), text.contains('\n'), "newline in {text:?}");
    }
    let text = ast.text(ast.root());
    for (byte_offset, _) in text.char_indices().chain([(text.len(), ' ')]) {
        assert_eq!(ast.utf16_offset(ast.root(), byte_offset), text[..byte_offset].encode_utf16().count());
    }
}

const SOURCE: &str = "// é 😀\nfun f() {\n    val s = \"ü😀\" // ß\n    g(1, \"✓\")\n}\n\nval x = \"€\"\n";

#[test]
fn caches_follow_edits() {
    let mut ast = ast(SOURCE);
    assert_caches(&ast);

    let identifier = find(&ast, IDENTIFIER, 1);
    ast.raw_replace_with_text(identifier, "längerName");
    assert_caches(&ast);

    let call = find(&ast, CALL_EXPRESSION, 0);
    let parent = ast.tree_parent(call).unwrap();
    ast.remove_child(parent, call);
    assert_caches(&ast);

    let fun = find(&ast, FUN, 0);
    let property = find(&ast, PROPERTY, 1);
    let file = ast.root();
    ast.add_leaf(file, WHITE_SPACE, "\n\n// 🎉\n", Some(fun));
    ast.add_child(file, property, Some(fun));
    assert_caches(&ast);

    let moved: NodeId = ast.clone(fun);
    ast.add_child(file, moved, None);
    assert_caches(&ast);
}
