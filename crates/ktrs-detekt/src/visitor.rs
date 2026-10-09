//! `DetektVisitor` (= `KtTreeVisitorVoid`), traversed sparsely.
//!
//! A tree visitor's default for every element is "visit the children", so an element the visitor has no
//! override for is transparent: only the elements whose dispatch chain reaches an overridden `visit_*` do
//! anything. [`accept_children`] therefore `accept`s just the descendants of those kinds, in preorder; each of
//! them recurses the same way when (and where) its override calls `super`, and its subtree is skipped when it
//! doesn't. Visit order, pruning and the visitor's state between calls are exactly those of the full walk.
//!
//! The descendants come from a per-file index ([`SparseIndex`]): one pass over the tree collects the nodes of
//! every kind in [`visit_kinds`], and each visitor type gets its own sorted id list on first use.
//!
//! [`detekt_visitor!`] writes the `KtVisitorVoid` impl of a visitor and derives the kinds from the methods it
//! overrides (table: [`visit_kinds`]), so the two cannot disagree. A visitor of leaves or comments needs a plain
//! `KtVisitorVoid` impl with `kt_tree_visitor_void::visit_element` instead.

use std::cell::{OnceCell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ktrs_psi::{KtVisitorVoid, PsiElement};
use ktrs_syntax::{ElementId, SyntaxKind, Tree};

use crate::kt_file;

/// The composite kinds one visitor type reacts to.
pub struct VisitorKinds {
    /// Bit per raw kind (kinds are under 512).
    bits: [u64; 8],
    any: bool,
    /// This visitor type's list in every [`SparseIndex`], plus one; 0 until first used.
    slot: AtomicUsize,
}

static NEXT_SLOT: AtomicUsize = AtomicUsize::new(0);

impl VisitorKinds {
    /// The union of the kind lists of the overridden methods.
    pub const fn new(lists: &[&[SyntaxKind]]) -> VisitorKinds {
        let mut bits = [0u64; 8];
        let mut any = false;
        let mut i = 0;
        while i < lists.len() {
            let mut j = 0;
            while j < lists[i].len() {
                let kind = lists[i][j] as usize;
                bits[kind / 64] |= 1 << (kind % 64);
                any = true;
                j += 1;
            }
            i += 1;
        }
        VisitorKinds { bits, any, slot: AtomicUsize::new(0) }
    }

    fn contains(&self, kind: SyntaxKind) -> bool {
        let kind = kind as usize;
        self.bits[kind / 64] >> (kind % 64) & 1 != 0
    }

    fn slot(&self) -> usize {
        let slot = self.slot.load(Ordering::Relaxed);
        if slot != 0 {
            return slot - 1;
        }
        let new = NEXT_SLOT.fetch_add(1, Ordering::Relaxed) + 1;
        match self.slot.compare_exchange(0, new, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => new - 1,
            Err(assigned) => assigned - 1,
        }
    }
}

pub trait DetektVisitor: KtVisitorVoid {
    fn kinds(&self) -> &'static VisitorKinds;
}

/// Per file: the nodes any sparse visitor can react to, and per visitor type the ids of its kinds, in preorder.
#[derive(Default)]
pub(crate) struct SparseIndex {
    nodes: OnceCell<Vec<(SyntaxKind, ElementId)>>,
    lists: RefCell<Vec<Option<Rc<[ElementId]>>>>,
}

static ALL_KINDS: VisitorKinds = VisitorKinds::new(visit_kinds::ALL);

impl SparseIndex {
    fn node_ids(&self, tree: &Tree, kinds: &VisitorKinds) -> Rc<[ElementId]> {
        let slot = kinds.slot();
        let mut lists = self.lists.borrow_mut();
        if lists.len() <= slot {
            lists.resize(slot + 1, None);
        }
        if let Some(list) = &lists[slot] {
            return list.clone();
        }
        let nodes = self.nodes.get_or_init(|| {
            (0..tree.len() as ElementId)
                .filter(|&id| !tree.is_token(id))
                .map(|id| (tree.kind(id), id))
                .filter(|(kind, _)| ALL_KINDS.contains(*kind))
                .collect()
        });
        let list: Rc<[ElementId]> = nodes.iter().filter(|(kind, _)| kinds.contains(*kind)).map(|&(_, id)| id).collect();
        lists[slot] = Some(list.clone());
        list
    }
}

/// `KtTreeVisitorVoid.visitElement` (`element.acceptChildren(this)`) of a [`DetektVisitor`].
pub fn accept_children<V: DetektVisitor + ?Sized>(v: &mut V, element: &PsiElement) {
    let kinds = v.kinds();
    if !kinds.any {
        return;
    }
    let tree = element.tree();
    let ids = kt_file::containing_file(element).sparse_index().node_ids(tree, kinds);
    let end = tree.subtree_end(element.id());
    let mut index = ids.partition_point(|&id| id <= element.id());
    while let Some(&id) = ids.get(index).filter(|&&id| id < end) {
        element.at(id).accept(v);
        let subtree_end = tree.subtree_end(id);
        index += 1 + ids[index + 1..].partition_point(|&next| next < subtree_end);
    }
}

macro_rules! visit_kinds {
    ($($method:ident => [$($kind:ident),*];)*) => {
        /// Per `KtVisitorVoid` method: the element kinds whose `accept` reaches it (ktrs-psi `visitor/dispatch.rs`
        /// plus the super chain in `visitor/mod.rs`). Only the methods some ported visitor overrides are listed.
        #[allow(non_upper_case_globals)]
        pub mod visit_kinds {
            use ktrs_syntax::SyntaxKind::{self, *};

            $(pub const $method: &[SyntaxKind] = &[$($kind),*];)*

            pub(super) const ALL: &[&[SyntaxKind]] = &[$($method),*];
        }
    };
}

visit_kinds! {
    // The root is always visited.
    visit_kt_file => [];
    visit_package_directive => [PACKAGE_DIRECTIVE];
    visit_import_directive => [IMPORT_DIRECTIVE];
    visit_class => [CLASS, ENUM_ENTRY];
    visit_object_declaration => [OBJECT_DECLARATION];
    visit_class_or_object => [CLASS, ENUM_ENTRY, OBJECT_DECLARATION];
    visit_primary_constructor => [PRIMARY_CONSTRUCTOR];
    visit_secondary_constructor => [SECONDARY_CONSTRUCTOR];
    visit_class_initializer => [CLASS_INITIALIZER];
    visit_named_function => [FUN];
    visit_parameter => [VALUE_PARAMETER];
    visit_constant_expression => [NULL, BOOLEAN_CONSTANT, FLOAT_CONSTANT, CHARACTER_CONSTANT, INTEGER_CONSTANT];
    visit_binary_expression => [BINARY_EXPRESSION];
    visit_call_expression => [CALL_EXPRESSION];
    visit_if_expression => [IF];
    visit_when_expression => [WHEN];
    visit_try_expression => [TRY];
    visit_catch_section => [CATCH];
    visit_finally_section => [FINALLY];
    visit_loop_expression => [FOR, WHILE, DO_WHILE];
    visit_for_expression => [FOR];
    visit_while_expression => [WHILE];
    visit_do_while_expression => [DO_WHILE];
    visit_break_expression => [BREAK];
    visit_continue_expression => [CONTINUE];
}

/// The `KtVisitorVoid` and [`DetektVisitor`] impls of a visitor from its overridden `visit_*` methods.
#[macro_export]
macro_rules! detekt_visitor {
    (impl $visitor:ty { $(fn $name:ident(&mut $this:ident, $arg:ident: &$arg_type:ty) $body:block)* }) => {
        impl ktrs_psi::KtVisitorVoid for $visitor {
            fn visit_element(&mut self, element: &ktrs_psi::PsiElement) {
                $crate::visitor::accept_children(self, element);
            }

            fn ignores_leaves(&self) -> bool {
                true
            }

            $(fn $name(&mut $this, $arg: &$arg_type) $body)*
        }

        impl $crate::visitor::DetektVisitor for $visitor {
            fn kinds(&self) -> &'static $crate::visitor::VisitorKinds {
                static KINDS: $crate::visitor::VisitorKinds =
                    $crate::visitor::VisitorKinds::new(&[$($crate::visitor::visit_kinds::$name),*]);
                &KINDS
            }
        }
    };
}
