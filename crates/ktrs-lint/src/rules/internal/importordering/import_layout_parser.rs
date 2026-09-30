//! Port of `ImportLayoutParser.kt`.

use super::pattern_entry::PatternEntry;
use crate::rules::internal::kotlin_string::trim;

pub const BLANK_LINE_CHAR: &str = "|";
pub const WILDCARD_CHAR: &str = "*";
pub const ALIAS_CHAR: &str = "^";

/// `parseImportsLayout`; `Err` is the `IllegalArgumentException` message.
pub fn parse_imports_layout(imports_layout: &str) -> Result<Vec<PatternEntry>, String> {
    let imports_list: Vec<&str> = imports_layout.split(',').map(trim).collect();

    if imports_list.first() == Some(&BLANK_LINE_CHAR) || imports_list.last() == Some(&BLANK_LINE_CHAR) {
        return Err("Blank lines are not supported in the beginning or end of import list".to_owned());
    }

    if !imports_list.contains(&WILDCARD_CHAR) {
        return Err("<all other imports> symbol (\"*\") must be present in the custom imports layout".to_owned());
    }

    Ok(imports_list
        .into_iter()
        .map(|it| {
            let mut import = it;
            if import == BLANK_LINE_CHAR {
                return PatternEntry::blank_line_entry();
            }
            let mut has_alias = false;
            let mut with_subpackages = false;
            if let Some(rest) = import.strip_prefix(ALIAS_CHAR) {
                import = trim(rest);
                has_alias = true;
            }
            if import.ends_with("**") {
                // java.**
                import = &import[..import.len() - 1];
                with_subpackages = true;
            }
            if import == WILDCARD_CHAR {
                PatternEntry::all_other_imports_entry()
            } else if import.is_empty() && has_alias {
                PatternEntry::all_other_alias_imports_entry()
            } else {
                PatternEntry::new(import, with_subpackages, has_alias)
            }
        })
        .collect())
}
