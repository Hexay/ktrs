//! The ktlint 2.0 expectations of a case recorded on 1.8: `<case>.2_0.{lint,format,error}` replace the 1.8 rows when
//! present (an empty file: 2.0 has none of that kind), `<case>.2_0.expected.kt|kts` the formatted text.

use std::path::Path;

use crate::case::{self, Case};

pub fn ktlint_2_0(root: &Path, case_18: &Case) -> Case {
    let base = root.join(&case_18.name);
    let mut case = case_18.clone();
    if let Some(lint) = case::read(&base, ".2_0.lint") {
        case.lint = case::rows(Some(lint));
    }
    if let Some(format) = case::read(&base, ".2_0.format") {
        case.format = case::rows(Some(format));
    }
    if let Some(error) = case::read(&base, ".2_0.error") {
        case.errors = case::rows(Some(error))
            .iter()
            .filter_map(|l| l.split_once('\t').map(|(stage, e)| (stage.to_owned(), e.to_owned())))
            .collect();
    }
    let ext = if case.options.script { "kts" } else { "kt" };
    if let Some(expected) = case::read(&base, &format!(".2_0.expected.{ext}")) {
        case.expected = expected;
    }
    case
}
