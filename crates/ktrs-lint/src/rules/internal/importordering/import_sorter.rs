//! Port of `ImportSorter.kt`: orders imports by layout pattern, then alphabetically.

use std::cmp::Ordering;

use ktrs_ast::psi::ImportPath;

use super::pattern_entry::PatternEntry;

pub struct ImportSorter {
    pub patterns: Vec<PatternEntry>,
}

impl ImportSorter {
    pub fn new(patterns: Vec<PatternEntry>) -> ImportSorter {
        ImportSorter { patterns }
    }

    /// `compare(import1, import2)`: `compareValuesBy(findImportIndex, toString().replace("`", ""))`, the
    /// strings compared by UTF-16 unit like `String.compareTo`.
    pub fn compare(&self, import_path1: &ImportPath, import_path2: &ImportPath) -> Ordering {
        self.find_import_index(import_path1)
            .cmp(&self.find_import_index(import_path2))
            .then_with(|| sort_text(import_path1).encode_utf16().cmp(sort_text(import_path2).encode_utf16()))
    }

    pub fn find_import_index(&self, path: &ImportPath) -> i32 {
        let mut best_index: i32 = -1;
        let mut best_entry_match: Option<&PatternEntry> = None;
        let mut all_other_alias_index = -1;
        let mut all_other_index = -1;

        for (index, entry) in self.patterns.iter().enumerate() {
            let index = index as i32;
            if *entry == PatternEntry::all_other_alias_imports_entry() {
                all_other_alias_index = index;
            }
            if *entry == PatternEntry::all_other_imports_entry() {
                all_other_index = index;
            }
            if entry.is_better_match_for_package_than(best_entry_match, path) {
                best_entry_match = Some(entry);
                best_index = index;
            }
        }

        if best_index == -1 && path.has_alias() && all_other_alias_index == -1 && all_other_index != -1 {
            // if no layout for alias imports specified, put them among all others
            best_index = all_other_index;
        }
        best_index
    }
}

fn sort_text(import_path: &ImportPath) -> String {
    import_path.to_string().replace('`', "")
}
