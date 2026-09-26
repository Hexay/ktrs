//! `PsiElement` text accessors. They walk the green tree directly: a cursor walk
//! (`SyntaxNode::text`) allocates a node per step, and ktfmt reads the text of large elements often.

use std::ops::ControlFlow::{self, Break, Continue};

use ktrs_syntax::SyntaxElement;

use crate::element::PsiElement;

impl PsiElement {
    pub fn text(&self) -> String {
        let mut text = String::with_capacity(self.text_length());
        let _ = self.try_for_each_text_chunk(|chunk| {
            text.push_str(chunk);
            Continue::<()>(())
        });
        text
    }

    pub fn text_contains(&self, c: char) -> bool {
        self.try_for_each_text_chunk(|chunk| if chunk.contains(c) { Break(()) } else { Continue(()) }).is_break()
    }

    /// `text().chars().all(f)`, stopping at the first failing char without building the text.
    pub fn text_all(&self, mut f: impl FnMut(char) -> bool) -> bool {
        self.try_for_each_text_chunk(|chunk| if chunk.chars().all(&mut f) { Continue(()) } else { Break(()) })
            .is_continue()
    }

    /// Feeds the text to `f` token by token until `f` breaks: prefix checks without building the text.
    pub fn try_for_each_text_chunk<B>(&self, mut f: impl FnMut(&str) -> ControlFlow<B>) -> ControlFlow<B> {
        match self.syntax() {
            SyntaxElement::Node(n) => green_text_chunks(&n.green(), &mut f),
            SyntaxElement::Token(t) => f(t.text()),
        }
    }
}

fn green_text_chunks<B>(node: &rowan::GreenNodeData, f: &mut impl FnMut(&str) -> ControlFlow<B>) -> ControlFlow<B> {
    for child in node.children() {
        match child {
            rowan::NodeOrToken::Node(n) => green_text_chunks(n, f)?,
            rowan::NodeOrToken::Token(t) => f(t.text())?,
        }
    }
    Continue(())
}
