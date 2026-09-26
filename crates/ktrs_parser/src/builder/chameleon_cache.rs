//! [`ChameleonCache`]: expanded lazy nodes (`BLOCK`, `LAMBDA_EXPRESSION`, `DOC_COMMENT`, ...) keyed
//! by kind and text, for re-parsing text that is mostly unchanged (the formatter parses each file
//! about three times, and most bodies survive import sorting and trailing-comma edits verbatim).
//!
//! Exact: a chameleon's subtree depends only on its kind and text (see `LazyLeaf`), and green
//! nodes are position-independent values. Only error-free subtrees are kept, so a hit never has to
//! replay error messages.

use std::collections::HashMap;

use ktrs_syntax::{GreenNode, SyntaxKind};

use super::interner::{hash, raw};

#[derive(Default)]
pub struct ChameleonCache {
    nodes: HashMap<u64, Vec<(Box<str>, GreenNode)>>,
}

impl ChameleonCache {
    pub fn new() -> ChameleonCache {
        ChameleonCache::default()
    }

    pub(super) fn get(&self, kind: SyntaxKind, text: &str) -> Option<GreenNode> {
        let bucket = self.nodes.get(&key(kind, text))?;
        bucket.iter().find(|(t, node)| node.kind() == raw(kind) && **t == *text).map(|(_, node)| node.clone())
    }

    pub(super) fn insert(&mut self, kind: SyntaxKind, text: &str, node: GreenNode) {
        debug_assert_eq!(node.kind(), raw(kind));
        self.nodes.entry(key(kind, text)).or_default().push((text.into(), node));
    }
}

fn key(kind: SyntaxKind, text: &str) -> u64 {
    hash(raw(kind), text.as_bytes()) as u64
}
