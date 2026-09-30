//! `PsiElement` text accessors. An element's text is one contiguous slice of the file text.

use std::ops::ControlFlow;

use crate::element::PsiElement;

impl PsiElement {
    pub fn text(&self) -> String {
        self.text_slice().to_owned()
    }

    /// `getText()` without copying.
    pub fn text_slice(&self) -> &str {
        self.tree().text_of(self.id())
    }

    pub fn text_contains(&self, c: char) -> bool {
        self.text_slice().contains(c)
    }

    /// `text().chars().all(f)`, stopping at the first failing char without building the text.
    pub fn text_all(&self, f: impl FnMut(char) -> bool) -> bool {
        self.text_slice().chars().all(f)
    }

    /// Feeds the text to `f` (in one chunk: the text is contiguous).
    pub fn try_for_each_text_chunk<B>(&self, mut f: impl FnMut(&str) -> ControlFlow<B>) -> ControlFlow<B> {
        f(self.text_slice())
    }
}
