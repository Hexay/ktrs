//! `KtlintCommandLine`'s pattern sources: the arguments, `--patterns-from-stdin`, or the default patterns.

use crate::ktlint::args::KtlintArgs;
use crate::ktlint::console::Console;
use crate::ktlint::file_utils::{DEFAULT_PATTERNS, java_list};
use crate::ktlint::logger::{KTLINT_COMMAND_LINE, Logger};

pub fn replace_with_patterns_from_stdin_or_default_patterns_when_empty(
    args: &KtlintArgs,
    console: &Console,
    logger: &Logger,
) -> Vec<String> {
    let arguments = &args.arguments;
    if let Some(delimiter) = &args.patterns_from_stdin {
        let stdin_patterns = read_patterns_from_stdin(console, delimiter);
        if !stdin_patterns.is_empty() {
            if arguments.is_empty() {
                logger.debug(KTLINT_COMMAND_LINE, || {
                    format!("Patterns read from 'stdin' due to flag '--patterns-from-stdin': {}", java_list(&stdin_patterns))
                });
            } else {
                logger.warn(KTLINT_COMMAND_LINE, || {
                    format!(
                        "Patterns specified at command line ({}) and patterns from 'stdin' due to flag '--patterns-from-stdin' ({}) are merged",
                        java_list(arguments),
                        java_list(&stdin_patterns)
                    )
                });
            }
        }
        return arguments.iter().cloned().chain(stdin_patterns).collect();
    }
    if arguments.is_empty() {
        logger.info(KTLINT_COMMAND_LINE, || format!("Enable default patterns {}", java_list(&DEFAULT_PATTERNS)));
        return DEFAULT_PATTERNS.iter().map(|p| p.to_string()).collect();
    }
    arguments.clone()
}

/// Split by `delimiter` (the NUL byte when empty, as `git diff --name-only -z` writes), deduplicated.
fn read_patterns_from_stdin(console: &Console, delimiter: &str) -> Vec<String> {
    let text = String::from_utf8_lossy(&console.read_stdin()).into_owned();
    let mut patterns: Vec<String> = Vec::new();
    for pattern in text.split(if delimiter.is_empty() { "\0" } else { delimiter }).filter(|p| !p.is_empty()) {
        if !patterns.iter().any(|p| p == pattern) {
            patterns.push(pattern.to_owned());
        }
    }
    patterns
}
