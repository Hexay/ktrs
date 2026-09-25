//! KDoc PSI (`KDocName`); section/tag/link traversal uses `get_children_of_type`.

use ktrs_syntax::SyntaxKind::DOT;

use crate::types::*;

impl KDocName {
    pub fn qualifier(&self) -> Option<KDocName> {
        self.get_child_of_type()
    }

    /// `getNameTextRange()` relative to this element: the part after the dot, backticks stripped.
    pub fn name_text_range(&self) -> std::ops::Range<usize> {
        let text = self.text();
        let start = self.start_offset();
        let mut name_start = self.node().find_child_by_type(DOT).map_or(0, |dot| usize::from(dot.text_range().end()) - start);
        let mut name_end = text.len();
        if name_end - name_start >= 2 && text.as_bytes()[name_start] == b'`' && text.as_bytes()[name_end - 1] == b'`' {
            name_start += 1;
            name_end -= 1;
        }
        name_start..name_end
    }

    pub fn name_text(&self) -> String {
        self.text()[self.name_text_range()].to_owned()
    }

    pub fn qualified_name(&self) -> Vec<String> {
        let mut names = self.qualifier().map(|q| q.qualified_name()).unwrap_or_default();
        names.push(self.name_text());
        names
    }
}
