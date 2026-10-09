//! The row format of the corpus oracle (tools/detekt-oracle/src/DetektProbe.kt is the spec), shared by
//! `examples/detekt_probe`, the tests and `cargo detekt-diff`.

use std::path::{Path, PathBuf};
use std::{fs, panic};

use crate::api::Issue;
use crate::engine::Analyzer;

/// `line  col  endLine  endCol  start  end  rule  severity  signature  message`: a `rows.tsv` line without its
/// leading file column.
pub fn row(issue: &Issue) -> String {
    let location = issue.location();
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        location.source.line,
        location.source.column,
        location.end_source.line,
        location.end_source.column,
        location.text.start,
        location.text.end,
        issue.rule_instance.id,
        issue.severity.name(),
        escape(&issue.entity.signature),
        escape(&issue.message),
    )
}

pub fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n").replace('\r', "\\r")
}

/// The rows of one file in analyzer order, or the message of the panic that upstream would abort the run with.
pub fn rows_of_file(analyzer: &Analyzer, path: &Path) -> Result<Vec<String>, String> {
    let raw = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let raw = String::from_utf8_lossy(&raw);
    let issues = panic::catch_unwind(panic::AssertUnwindSafe(|| analyzer.analyze_text(path, &raw))).map_err(|payload| {
        let message = payload.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| payload.downcast_ref::<String>().cloned());
        message.unwrap_or_else(|| "<non-string panic>".to_owned())
    })?;
    Ok(issues.iter().map(row).collect())
}

/// The `.kt`/`.kts` files under `dir`, sorted.
pub fn kotlin_files(dir: &Path) -> Vec<PathBuf> {
    fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect(&path, out);
            } else if path.extension().is_some_and(|e| e == "kt" || e == "kts") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    collect(dir, &mut files);
    files.sort();
    files
}
