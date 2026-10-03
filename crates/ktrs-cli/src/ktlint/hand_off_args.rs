//! The argv a run hands to the ktlint jar: the run's own, without ktrs's `--ktlint-version` (the jar rejects it),
//! also where it came from an `@argfile`. Such an argfile is replaced by a temporary one holding its expansion
//! minus that option, so the jar's Clikt still reads those tokens from a file: no shell or `java` launcher
//! (Windows wildcard expansion) handling applies to them, and every other argfile is passed through untouched.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ktlint::clikt::expand_argument_files;
use crate::ktlint::version::KTLINT_VERSION_OPTION;

/// The jar's argv; the temporary argfiles it names are deleted on drop.
#[derive(Debug)]
pub struct HandOffArgs {
    pub args: Vec<String>,
    temp_files: Vec<PathBuf>,
}

impl HandOffArgs {
    /// Whether the argv names temporary files, which must outlive the jar (so no `exec`).
    pub fn has_temp_files(&self) -> bool {
        !self.temp_files.is_empty()
    }

    fn push_argfile(&mut self, tokens: &[&String]) -> Result<(), String> {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let file = std::env::temp_dir().join(format!("ktrs-ktlint-args-{}-{n}.txt", std::process::id()));
        let text: String = tokens.iter().map(|t| argfile_token(t) + "\n").collect();
        std::fs::write(&file, text).map_err(|e| format!("'{}' can not be written ({e}).", file.display()))?;
        self.args.push(format!("@{}", file.display()));
        self.temp_files.push(file);
        Ok(())
    }
}

impl Drop for HandOffArgs {
    fn drop(&mut self) {
        self.temp_files.iter().for_each(|file| drop(std::fs::remove_file(file)));
    }
}

/// `argv` without `--ktlint-version` and its value (before `--`), argfiles expanded as the drop-in's parse does.
pub fn hand_off_args(argv: &[String], working_dir: &Path) -> Result<HandOffArgs, String> {
    let mut filter = KtlintVersionFilter::default();
    let mut out = HandOffArgs { args: Vec::new(), temp_files: Vec::new() };
    for token in argv {
        if !is_argfile(token) {
            if filter.keeps(token) {
                out.args.push(token.clone());
            }
            continue;
        }
        let expanded = expand_argument_files(std::slice::from_ref(token), working_dir)?;
        let kept: Vec<&String> = expanded.iter().filter(|t| filter.keeps(t)).collect();
        if kept.len() == expanded.len() {
            out.args.push(token.clone());
        } else {
            out.push_argfile(&kept)?;
        }
    }
    Ok(out)
}

/// `@x` reads file `x`; `@@x` is the literal `@x` ([`expand_argument_files`]).
fn is_argfile(token: &str) -> bool {
    token.starts_with('@') && !token.starts_with("@@")
}

/// One token for Clikt's argfile tokenizer: double-quoted, `\` and `"` escaped, a leading `@` doubled.
fn argfile_token(token: &str) -> String {
    let token = if token.starts_with('@') { format!("@{token}") } else { token.to_owned() };
    format!("\"{}\"", token.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Drops `--ktlint-version <v>` and `--ktlint-version=<v>` until `--`, over a stream of tokens.
#[derive(Default)]
struct KtlintVersionFilter {
    after_separator: bool,
    skip_value: bool,
}

impl KtlintVersionFilter {
    fn keeps(&mut self, token: &str) -> bool {
        if self.after_separator {
            return true;
        }
        if std::mem::take(&mut self.skip_value) {
            return false;
        }
        if token == "--" {
            self.after_separator = true;
            return true;
        }
        if token == KTLINT_VERSION_OPTION {
            self.skip_value = true;
            return false;
        }
        !token.strip_prefix(KTLINT_VERSION_OPTION).is_some_and(|rest| rest.starts_with('='))
    }
}

#[cfg(test)]
mod tests;
