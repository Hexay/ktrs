//! Parser: no panic, the tree is lossless (its tokens spell the input), and a cached re-parse is identical.
#![no_main]

use std::cell::RefCell;

use ktrs_parser::{ChameleonCache, FileKind};
use ktrs_syntax::Tree;
use libfuzzer_sys::fuzz_target;

ktrs_fuzz::splice_mutators!();

/// Shared across inputs so cached subtrees come from other files; reset to bound memory.
const CACHE_USES: u32 = 512;

thread_local! {
    static CACHE: RefCell<(ChameleonCache, u32)> = RefCell::new((ChameleonCache::new(), 0));
}

fuzz_target!(|data: &[u8]| {
    let Some(text) = ktrs_fuzz::fuzz_input(data) else { return };
    let text = ktrs_fuzz::normalize_newlines(text);
    for kind in [FileKind::Source, FileKind::Script] {
        let parse = ktrs_parser::parse_file(&text, kind);
        assert_lossless(&parse.tree, &text);
        let _ = ktrs_syntax::psi_dump(&parse, "fuzz.kt");
        CACHE.with_borrow_mut(|(cache, uses)| {
            if *uses >= CACHE_USES {
                (*cache, *uses) = (ChameleonCache::new(), 0);
            }
            *uses += 1;
            let cached = ktrs_parser::parse_file_cached(&text, kind, cache);
            assert!(cached.tree == parse.tree, "cached parse has a different tree ({kind:?})");
            assert_eq!(cached.error_messages, parse.error_messages, "cached parse has different errors ({kind:?})");
        });
    }
});

fn assert_lossless(tree: &Tree, text: &str) {
    assert_eq!(tree.text(), text, "tree text");
    assert_eq!(tree.text_of(Tree::ROOT), text, "root range");
    let mut spelled = String::with_capacity(text.len());
    for e in 0..tree.len() as u32 {
        if tree.is_token(e) {
            spelled.push_str(tree.text_of(e));
        }
        if let Some(parent) = tree.parent(e) {
            let (r, p) = (tree.text_range(e), tree.text_range(parent));
            assert!(p.contains_range(r), "element {e} {r:?} escapes its parent {parent} {p:?}");
        }
    }
    assert_eq!(spelled, text, "tokens do not spell the input");
}
