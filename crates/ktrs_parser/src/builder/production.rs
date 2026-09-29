//! Port of `MarkerProduction` + `MarkerOptionalData` + the marker objects of `PsiBuilderImpl`.
//!
//! The production is the flat event list IntelliJ builds the tree from: `+id` is a start marker or
//! an error item, `-id` the "done" of start marker `id`. Marker ids are never reused (IntelliJ pools
//! them; reuse is unobservable), so a freed marker just gets `lexeme = -1`.

use std::num::NonZeroU32;

use ktrs_syntax::SyntaxKind;

use super::binders::EdgeBinder;

#[derive(Debug)]
pub(crate) struct MarkerData {
    /// `ErrorItem` (from `builder.error`) rather than `StartMarker`.
    pub(crate) is_error_item: bool,
    /// `myLexemeIndex`; -1 once disposed.
    pub(crate) lexeme: i32,
    /// `myDoneLexeme`; -1 until done.
    pub(crate) done_lexeme: i32,
    pub(crate) kind: Option<SyntaxKind>,
    /// Error-item message, or the done-error of an `ERROR_ELEMENT` marker: an index into
    /// `Production::messages`, keeping this struct small (markers are the hottest allocation).
    message: Option<NonZeroU32>,
    pub(crate) left_binder: Option<EdgeBinder>,
    pub(crate) right_binder: Option<EdgeBinder>,
    pub(crate) collapsed: bool,
}

impl MarkerData {
    fn new(is_error_item: bool, lexeme: i32) -> MarkerData {
        MarkerData {
            is_error_item,
            lexeme,
            done_lexeme: -1,
            kind: None,
            message: None,
            left_binder: None,
            right_binder: None,
            collapsed: false,
        }
    }

    pub(crate) fn has_message(&self) -> bool {
        self.message.is_some()
    }

    pub(crate) fn is_done(&self) -> bool {
        self.done_lexeme != -1
    }

    /// `ProductionMarker.getLexemeIndex(done)`.
    pub(crate) fn get_lexeme_index(&self, done: bool) -> i32 {
        if done { self.done_lexeme } else { self.lexeme }
    }

    /// `ProductionMarker.setLexemeIndex(lexemeIndex, done)`.
    pub(crate) fn set_lexeme_index(&mut self, lexeme_index: i32, done: bool) {
        if done {
            debug_assert!(!self.is_error_item);
            self.done_lexeme = lexeme_index;
        } else {
            self.lexeme = lexeme_index;
        }
    }

    /// `MarkerOptionalData.getBinder(markerId, right)`.
    pub(crate) fn get_binder(&self, right: bool) -> EdgeBinder {
        if right {
            self.right_binder.unwrap_or(EdgeBinder::DefaultRight)
        } else {
            self.left_binder.unwrap_or(EdgeBinder::DefaultLeft)
        }
    }
}

#[derive(Debug)]
pub(crate) struct Production {
    /// Index `id - 1` holds marker `id`.
    pub(crate) markers: Vec<MarkerData>,
    pub(crate) list: Vec<i32>,
    messages: Vec<String>,
    /// Error items ever allocated (dropped or rolled back ones included).
    error_items: u32,
}

impl Production {
    /// Over (empty, possibly recycled) vectors; see `pool.rs`.
    pub(crate) fn from_vecs(markers: Vec<MarkerData>, list: Vec<i32>) -> Production {
        debug_assert!(markers.is_empty() && list.is_empty());
        Production { markers, list, messages: Vec::new(), error_items: 0 }
    }

    pub(crate) fn has_error_items(&self) -> bool {
        self.error_items > 0
    }

    pub(crate) fn take_vecs(&mut self) -> (Vec<MarkerData>, Vec<i32>) {
        (std::mem::take(&mut self.markers), std::mem::take(&mut self.list))
    }

    pub(crate) fn set_message(&mut self, id: i32, message: &str) {
        self.messages.push(message.to_owned());
        self.marker_mut(id).message = NonZeroU32::new(self.messages.len() as u32);
    }

    pub(crate) fn message(&self, id: i32) -> Option<&str> {
        self.marker(id).message.map(|i| self.messages[i.get() as usize - 1].as_str())
    }

    pub(crate) fn allocate(&mut self, is_error_item: bool, lexeme: i32) -> i32 {
        self.error_items += u32::from(is_error_item);
        self.markers.push(MarkerData::new(is_error_item, lexeme));
        self.markers.len() as i32
    }

    pub(crate) fn marker(&self, id: i32) -> &MarkerData {
        &self.markers[id.unsigned_abs() as usize - 1]
    }

    pub(crate) fn marker_mut(&mut self, id: i32) -> &mut MarkerData {
        &mut self.markers[id.unsigned_abs() as usize - 1]
    }

    pub(crate) fn size(&self) -> usize {
        self.list.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub(crate) fn add_before(&mut self, id: i32, anchor: i32) {
        let idx = self.index_of(anchor);
        self.list.insert(idx, id);
    }

    /// The upstream linear-then-binary search finds the unique `+id` entry; so does this.
    fn index_of(&self, id: i32) -> usize {
        self.list.iter().rposition(|&x| x == id).expect("Dropped or rolled-back marker")
    }

    pub(crate) fn add_marker(&mut self, id: i32) {
        self.list.push(id);
    }

    pub(crate) fn rollback_to(&mut self, id: i32) {
        let idx = self.index_of(id);
        for i in (idx..self.list.len()).rev() {
            let marker_id = self.list[i];
            if marker_id > 0 {
                self.free_marker(marker_id);
            }
        }
        self.list.truncate(idx);
    }

    pub(crate) fn has_errors_after(&self, id: i32) -> bool {
        (self.index_of(id) + 1..self.list.len()).any(|i| {
            let m = self.list[i];
            m > 0 && {
                let data = self.marker(m);
                data.is_error_item || data.kind == Some(SyntaxKind::ERROR_ELEMENT) && data.has_message()
            }
        })
    }

    pub(crate) fn drop_marker(&mut self, id: i32) {
        if self.marker(id).is_done() {
            let idx = self.list.iter().rposition(|&x| x == -id).expect("done entry");
            self.list.remove(idx);
        }
        let idx = self.index_of(id);
        self.list.remove(idx);
        self.free_marker(id);
    }

    pub(crate) fn add_done(&mut self, id: i32, anchor_before: Option<i32>) {
        let idx = match anchor_before {
            None => self.list.len(),
            Some(anchor) => self.index_of(anchor),
        };
        self.list.insert(idx, -id);
    }

    /// `getStartMarkerAt`: the marker or error item that starts at production `index`.
    pub(crate) fn get_start_marker_at(&self, index: usize) -> Option<i32> {
        let id = self.list[index];
        (id > 0).then_some(id)
    }

    /// `getLexemeIndexAt`.
    pub(crate) fn get_lexeme_index_at(&self, index: usize) -> i32 {
        let id = self.list[index];
        self.marker(id).get_lexeme_index(id < 0)
    }

    /// Only the disposed flag matters: a freed id is never read again except by the asserts.
    fn free_marker(&mut self, id: i32) {
        self.marker_mut(id).lexeme = -1;
    }
}
