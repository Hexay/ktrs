//! Port of `ImportSorter.kt`: orders imports by layout pattern, then alphabetically.

use ktrs_ast::psi::ImportPath;

use super::pattern_entry::PatternEntry;

pub struct ImportSorter {
    pub patterns: Vec<PatternEntry>,
}

impl ImportSorter {
    pub fn new(patterns: Vec<PatternEntry>) -> ImportSorter {
        ImportSorter { patterns }
    }

    /// `compare(import1, import2)` is the order of these keys: `compareValuesBy(findImportIndex,
    /// toString().replace("`", ""))`, the strings compared by UTF-16 unit like `String.compareTo`. Computed
    /// once per import, since both parts render the path.
    pub fn sort_key(&self, import_path: &ImportPath) -> (i32, Vec<u16>) {
        (self.find_import_index(import_path), import_path.to_string().replace('`', "").encode_utf16().collect())
    }

    pub fn find_import_index(&self, path: &ImportPath) -> i32 {
        let mut best_index: i32 = -1;
        let mut best_entry_match: Option<&PatternEntry> = None;
        let mut all_other_alias_index = -1;
        let mut all_other_index = -1;
        let (has_alias, path_str) = (path.has_alias(), path.path_str());

        for (index, entry) in self.patterns.iter().enumerate() {
            let index = index as i32;
            if *entry == PatternEntry::all_other_alias_imports_entry() {
                all_other_alias_index = index;
            }
            if *entry == PatternEntry::all_other_imports_entry() {
                all_other_index = index;
            }
            if entry.is_better_match_for_package_than(best_entry_match, has_alias, &path_str) {
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
