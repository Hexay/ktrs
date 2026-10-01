//! Port of IntelliJ `TreeElement` (links, offsets, raw relinking). Notifications (`subtreeChanged`,
//! `onInvalidated`, PSI events) have no observable effect for ktlint and are dropped; their cache
//! invalidation is done eagerly in `set_tree_parent`.

use crate::arena::{Ast, NONE, NodeId, opt, raw};

impl Ast {
    /// `clone()`: a deep, detached copy (`CompositeElement.clone` copies the children, a leaf clone
    /// shares its text).
    pub fn clone(&mut self, n: NodeId) -> NodeId {
        let copy = self.push_unlinked_copy(n);
        let mut child = self.node(n).first;
        while child != NONE {
            let child_copy = self.clone(NodeId(child));
            self.raw_add_children_without_notifications(copy, child_copy);
            child = self.nodes[child as usize].next;
        }
        copy
    }

    /// `getStartOffset()`: relative to the topmost ancestor (the file, or a dummy holder when detached).
    pub fn start_offset(&self, n: NodeId) -> usize {
        let mut result = 0;
        let mut current = n;
        while let Some(parent) = self.tree_parent(current) {
            result += self.start_offset_in_parent(current) as usize;
            current = parent;
        }
        result
    }

    /// `getStartOffsetInParent()`; `u32::MAX` (-1) without a parent.
    pub(crate) fn start_offset_in_parent(&self, n: NodeId) -> u32 {
        let node = self.node(n);
        if node.parent == NONE {
            return NONE;
        }
        let mut offset_in_parent = node.offset_in_parent.get();
        if offset_in_parent != NONE {
            return offset_in_parent;
        }
        let mut cur = n;
        loop {
            let prev = self.node(cur).prev;
            if prev == NONE {
                break;
            }
            cur = NodeId(prev);
            offset_in_parent = self.node(cur).offset_in_parent.get();
            if offset_in_parent != NONE {
                break;
            }
        }
        if offset_in_parent == NONE {
            offset_in_parent = 0;
            self.node(cur).offset_in_parent.set(0);
        }
        while cur != n {
            let next = NodeId(self.node(cur).next);
            offset_in_parent += self.node(cur).len;
            self.node(next).offset_in_parent.set(offset_in_parent);
            cur = next;
        }
        offset_in_parent
    }

    /// `setTreeParent`, plus the length bookkeeping IntelliJ does lazily via `subtreeChanged`.
    pub(crate) fn set_tree_parent(&mut self, n: NodeId, parent: u32) {
        let old = self.node(n).parent;
        if old == parent {
            return;
        }
        let metrics = (self.node(n).len, self.node(n).surplus, self.node(n).newlines);
        self.add_length_to_ancestors(old, metrics, false);
        self.node_mut(n).parent = parent;
        self.add_length_to_ancestors(parent, metrics, true);
        self.clear_text_hashes(old);
        self.clear_text_hashes(parent);
    }

    /// Clears the cached text hash of `element` and its ancestors, up to the first one already clear.
    pub(crate) fn clear_text_hashes(&self, element: u32) {
        let mut cur = element;
        while cur != NONE && self.nodes[cur as usize].text_hash.take().is_some() {
            cur = self.nodes[cur as usize].parent;
        }
    }

    /// Adds (or removes) a subtree's `(len, surplus, newlines)` to every ancestor.
    fn add_length_to_ancestors(&mut self, mut ancestor: u32, (len, surplus, newlines): (u32, u32, u32), add: bool) {
        if len == 0 {
            return;
        }
        while ancestor != NONE {
            let node = &mut self.nodes[ancestor as usize];
            node.len = if add { node.len + len } else { node.len - len };
            node.surplus = if add { node.surplus + surplus } else { node.surplus - surplus };
            node.newlines = if add { node.newlines + newlines } else { node.newlines - newlines };
            let (next, parent) = (node.next, node.parent);
            self.clear_relative_offsets(next);
            ancestor = parent;
        }
    }

    pub(crate) fn set_tree_prev(&mut self, n: NodeId, prev: u32) {
        self.node_mut(n).prev = prev;
        self.clear_relative_offsets(n.0);
        self.clear_text_hashes(self.node(n).parent);
    }

    pub(crate) fn set_tree_next(&mut self, n: NodeId, next: u32) {
        self.node_mut(n).next = next;
        self.clear_relative_offsets(next);
        self.clear_text_hashes(self.node(n).parent);
    }

    pub(crate) fn clear_relative_offsets(&self, element: u32) {
        let mut cur = element;
        while cur != NONE && self.nodes[cur as usize].offset_in_parent.get() != NONE {
            self.nodes[cur as usize].offset_in_parent.set(NONE);
            cur = self.nodes[cur as usize].next;
        }
    }

    /// `rawInsertBeforeMe(firstNew)`: inserts `first_new` and every sibling after it.
    pub fn raw_insert_before_me(&mut self, this: NodeId, first_new: NodeId) {
        match self.tree_prev(this) {
            None => {
                self.raw_remove_up_to_last(first_new);
                let p = self.node(this).parent;
                if p != NONE {
                    self.set_first_child_node(NodeId(p), first_new.0);
                }
                let mut first_new = first_new;
                loop {
                    let tree_next = self.node(first_new).next;
                    assert!(tree_next != this.0, "Attempt to create cycle");
                    self.set_tree_parent(first_new, p);
                    if tree_next == NONE {
                        self.set_tree_prev(this, first_new.0);
                        self.set_tree_next(first_new, this.0);
                        break;
                    }
                    first_new = NodeId(tree_next);
                }
            }
            Some(anchor_prev) => self.raw_insert_after_me(anchor_prev, first_new),
        }
    }

    /// `rawInsertAfterMe(firstNew)`: inserts `first_new` and every sibling after it.
    pub fn raw_insert_after_me(&mut self, this: NodeId, first_new: NodeId) {
        self.raw_insert_after_me_without_notifications(this, first_new);
    }

    pub(crate) fn raw_insert_after_me_without_notifications(&mut self, this: NodeId, first_new: NodeId) {
        self.raw_remove_up_to_without_notifications(first_new, None, false);
        let p = self.node(this).parent;
        let tree_next = self.node(this).next;
        self.set_tree_prev(first_new, this.0);
        self.set_tree_next(this, first_new.0);
        let mut first_new = first_new;
        loop {
            let n = self.node(first_new).next;
            assert!(n != this.0, "Attempt to create cycle");
            self.set_tree_parent(first_new, p);
            if n == NONE {
                if tree_next == NONE {
                    if p != NONE {
                        self.set_tree_parent(first_new, p);
                        self.set_last_child_node(NodeId(p), first_new.0);
                    }
                } else {
                    self.set_tree_next(first_new, tree_next);
                    self.set_tree_prev(NodeId(tree_next), first_new.0);
                }
                return;
            }
            first_new = NodeId(n);
        }
    }

    /// `rawRemove()`: unlinks `this`, leaving it with no parent or siblings.
    pub fn raw_remove(&mut self, this: NodeId) {
        let node = self.node(this);
        let (next, parent, prev) = (node.next, node.parent, node.prev);
        if prev != NONE {
            self.set_tree_next(NodeId(prev), next);
        } else if parent != NONE {
            self.set_first_child_node(NodeId(parent), next);
        }
        if next != NONE {
            self.set_tree_prev(NodeId(next), prev);
        } else if parent != NONE {
            self.set_last_child_node(NodeId(parent), prev);
        }
        self.invalidate(this);
    }

    /// `rawReplaceWithList(firstNew)`.
    pub fn raw_replace_with_list(&mut self, this: NodeId, first_new: Option<NodeId>) {
        if let Some(first_new) = first_new {
            self.raw_insert_after_me_without_notifications(this, first_new);
        }
        self.raw_remove(this);
    }

    pub(crate) fn invalidate(&mut self, this: NodeId) {
        self.set_tree_next(this, NONE);
        self.set_tree_prev(this, NONE);
        self.set_tree_parent(this, NONE);
    }

    pub fn raw_remove_up_to_last(&mut self, this: NodeId) {
        self.raw_remove_up_to(this, None);
    }

    /// `rawRemoveUpTo(end)`: detaches `this` up to (excluding) `end`; the range stays linked as a chain.
    pub fn raw_remove_up_to(&mut self, this: NodeId, end: Option<NodeId>) {
        self.raw_remove_up_to_without_notifications(this, end, true);
    }

    pub(crate) fn raw_remove_up_to_without_notifications(&mut self, this: NodeId, end: Option<NodeId>, _invalidate: bool) {
        if Some(this) == end {
            return;
        }
        let parent = self.node(this).parent;
        let start_prev = self.node(this).prev;
        let end_prev = end.map_or(NONE, |e| self.node(e).prev);
        if let Some(end) = end {
            assert!(self.node(end).parent == parent, "Trying to remove non-child");
            let mut element = opt(this.0);
            while element.is_some() && element != Some(end) {
                element = self.tree_next(element.unwrap());
            }
            assert!(element == Some(end), "end is not successor of this in the getTreeNext() chain");
        }
        if parent != NONE {
            if self.node(this).prev == NONE {
                self.set_first_child_node(NodeId(parent), raw(end));
            }
            if end.is_none() {
                self.set_last_child_node(NodeId(parent), start_prev);
            }
        }
        if start_prev != NONE {
            self.set_tree_next(NodeId(start_prev), raw(end));
        }
        if let Some(end) = end {
            self.set_tree_prev(end, start_prev);
        }
        self.set_tree_prev(this, NONE);
        if end_prev != NONE {
            self.set_tree_next(NodeId(end_prev), NONE);
        }
        if parent != NONE {
            let mut element = this.0;
            while element != NONE {
                self.set_tree_parent(NodeId(element), NONE);
                element = self.nodes[element as usize].next;
            }
        }
    }
}
