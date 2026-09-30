//! Port of `ParagraphList.kt`.

use std::fmt;

use super::kstring::to_string;
use super::paragraph::Paragraph;

/// A list of paragraphs, each starting on a new line; `separate` paragraphs get a blank line before.
/// Holds the builder's paragraph arena, since `prev`/`next` are arena indices.
pub struct ParagraphList {
    arena: Vec<Paragraph>,
    paragraphs: Vec<usize>,
}

impl ParagraphList {
    pub(super) fn new(arena: Vec<Paragraph>, paragraphs: Vec<usize>) -> Self {
        ParagraphList { arena, paragraphs }
    }

    pub fn is_single_paragraph(&self) -> bool {
        self.paragraphs.len() <= 1
    }

    pub fn iter(&self) -> impl Iterator<Item = &Paragraph> {
        self.paragraphs.iter().map(|&id| &self.arena[id])
    }
}

impl fmt::Display for ParagraphList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.iter().map(|p| to_string(&p.content)).collect();
        f.write_str(&parts.join(", "))
    }
}
