//! Port of `java/Replacement.java`: replace a byte range of the input with a string.

use super::range::Range;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Replacement {
    replace_range: Range,
    replacement_string: String,
}

impl Replacement {
    pub fn create(start_position: i32, end_position: i32, replace_with: String) -> Replacement {
        assert!(start_position >= 0, "startPosition must be non-negative");
        assert!(
            start_position <= end_position,
            "startPosition cannot be after endPosition"
        );
        Replacement {
            replace_range: Range::closed_open(start_position, end_position),
            replacement_string: replace_with,
        }
    }

    pub fn get_replace_range(&self) -> Range {
        self.replace_range
    }

    pub fn get_replacement_string(&self) -> &str {
        &self.replacement_string
    }
}
