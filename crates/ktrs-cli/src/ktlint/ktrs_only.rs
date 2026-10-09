//! What `ktrs lint` adds to a ktlint run; the `ktlint` drop-in leaves these at their defaults.

use crate::changed_since::ChangedFiles;

#[derive(Clone, Debug, Default)]
pub struct KtrsLintOptions {
    /// `--changed-since`: the ref, and the files it resolved to (the run keeps those of its patterns).
    pub changed_since: Option<String>,
    pub changed_files: Option<ChangedFiles>,
    /// `--reporter` knows `github` (`reporter/github.rs`).
    pub github_reporter: bool,
}
