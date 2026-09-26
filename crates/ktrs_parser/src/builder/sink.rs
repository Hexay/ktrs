//! [`TreeSink`]: the tree builder `tree.rs` binds into, shared by a file and all of its
//! chameleons so each lazy node is built in place instead of being copied out of a sub-tree.

use ktrs_syntax::{Parse, SyntaxKind, TreeBuilder};

use super::chameleon_cache::ChameleonCache;

#[derive(Default)]
pub struct TreeSink {
    tree: TreeBuilder,
    cache: Option<ChameleonCache>,
    pub(super) errors: Vec<String>,
}

impl TreeSink {
    pub fn new() -> TreeSink {
        TreeSink::default()
    }

    pub fn with_cache(cache: ChameleonCache) -> TreeSink {
        TreeSink { cache: Some(cache), ..TreeSink::default() }
    }

    pub fn take_cache(&mut self) -> Option<ChameleonCache> {
        self.cache.take()
    }

    pub fn finish(self) -> Parse {
        Parse { tree: self.tree.finish().into(), error_messages: self.errors }
    }

    /// Emits the expanded chameleon `kind` over `text`: the cached subtree if there is one, else
    /// whatever `build` emits (one root node of `kind`), remembered when it added no errors.
    pub fn chameleon(&mut self, kind: SyntaxKind, text: &str, build: impl FnOnce(&mut TreeSink)) {
        let Some(cache) = &self.cache else { return build(self) };
        if let Some(subtree) = cache.get(kind, text) {
            self.tree.push_tree(subtree);
            return;
        }
        let (root, errors) = (self.tree.len(), self.errors.len());
        build(self);
        if self.errors.len() == errors {
            let subtree = self.tree.extract(root);
            self.cache.as_mut().expect("checked above").insert(subtree);
        }
    }

    pub(super) fn token(&mut self, kind: SyntaxKind, text: &str) {
        self.tree.token(kind, text);
    }

    pub(super) fn start_node(&mut self, kind: SyntaxKind) {
        self.tree.start_node(kind);
    }

    pub(super) fn finish_node(&mut self) {
        self.tree.finish_node();
    }
}
