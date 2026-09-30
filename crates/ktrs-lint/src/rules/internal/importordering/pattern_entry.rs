//! Port of `PatternEntry.kt`: an entry of the imports layout, with the matching logic for imports.

use std::fmt;

use ktrs_ast::psi::ImportPath;

use super::import_layout_parser::{ALIAS_CHAR, BLANK_LINE_CHAR, WILDCARD_CHAR};
use crate::editorconfig::PropertyValueType;

/// Equality (`equals`) is over all three fields.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PatternEntry {
    package_name: String,
    pub with_subpackages: bool,
    pub has_alias: bool,
}

impl PatternEntry {
    pub fn new(package_name: &str, with_subpackages: bool, has_alias: bool) -> PatternEntry {
        PatternEntry {
            package_name: package_name.strip_suffix(".*").unwrap_or(package_name).to_owned(),
            with_subpackages,
            has_alias,
        }
    }

    pub fn blank_line_entry() -> PatternEntry {
        PatternEntry::new(BLANK_LINE_CHAR, true, false)
    }

    pub fn all_other_imports_entry() -> PatternEntry {
        PatternEntry::new(WILDCARD_CHAR, true, false)
    }

    pub fn all_other_alias_imports_entry() -> PatternEntry {
        PatternEntry::new(ALIAS_CHAR, true, true)
    }

    fn is_all_other_imports_entry(&self) -> bool {
        self.package_name == WILDCARD_CHAR && self.with_subpackages && !self.has_alias
    }

    fn is_all_other_alias_imports_entry(&self) -> bool {
        self.package_name == ALIAS_CHAR && self.with_subpackages && self.has_alias
    }

    pub fn is_blank_line_entry(&self) -> bool {
        self.package_name == BLANK_LINE_CHAR && self.with_subpackages && !self.has_alias
    }

    fn matches_package_name(&self, other_package_name: &str) -> bool {
        if self.is_all_other_imports_entry() || self.is_all_other_alias_imports_entry() {
            return true;
        }
        if self.is_blank_line_entry() {
            return false;
        }
        if other_package_name.starts_with(&self.package_name) {
            if other_package_name.len() == self.package_name.len() {
                return true;
            }
            if self.with_subpackages && other_package_name.as_bytes()[self.package_name.len()] == b'.' {
                return true;
            }
        }
        false
    }

    pub fn matches(&self, import: &ImportPath) -> bool {
        let path_str = import.path_str();
        self.matches_package_name(path_str.strip_suffix(".*").unwrap_or(&path_str))
    }

    pub fn is_better_match_for_package_than(&self, entry: Option<&PatternEntry>, import: &ImportPath) -> bool {
        if self.has_alias != import.has_alias() || !self.matches_package_name(&import.path_str()) {
            return false;
        }
        let Some(entry) = entry else { return true };
        if entry.has_alias != self.has_alias {
            return false;
        }
        // Any matched package is better than ALL_OTHER_IMPORTS_ENTRY
        if self.is_all_other_imports_entry() {
            return false;
        }
        if entry.is_all_other_imports_entry() {
            return true;
        }
        if entry.with_subpackages != self.with_subpackages {
            return !self.with_subpackages;
        }
        entry.package_name.matches('.').count() < self.package_name.matches('.').count()
    }
}

impl fmt::Display for PatternEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_all_other_imports_entry() {
            f.write_str(WILDCARD_CHAR)
        } else if self.is_all_other_alias_imports_entry() {
            f.write_str(ALIAS_CHAR)
        } else {
            let subpackages = if self.with_subpackages { WILDCARD_CHAR } else { "" };
            write!(f, "{}.{WILDCARD_CHAR}{subpackages}", self.package_name)
        }
    }
}

/// `List<PatternEntry>.toString()`.
impl PropertyValueType for Vec<PatternEntry> {
    fn to_value_string(&self) -> String {
        format!("[{}]", self.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "))
    }
}
