//! Port of the ktlint 2.0.0-ALPHA-4 CLI (`ktlint-cli`): the `ktlint` binary's flags, messages, reporters
//! and exit codes, over ktrs-lint's engine and standard rules. Differential test: tools/ktlint-oracle/cli-diff.sh.
//! With `--ktlint-version=1.8` (or `ktrs_ktlint_version = 1.8`) it is the 1.8.0 CLI instead: [`version`].
//!
//! A run that loads a rule set (`-R`) or reporter (`artifact=`) JAR with JVM code is handed to the real ktlint
//! jar ([`ktlint_jar`]). Deviations: the engine logs only its WARNs, so
//! `--log-level=debug|trace` adds only the CLI's own messages; help text is wrapped for 80 columns whatever the
//! terminal; a rule crash's `KtLintRuleException` has no stack frames (`\tat ...` lines), its cause named by the
//! JVM class (ktrs-lint `engine/rule_panic.rs`). Log lines name file workers `pool-1-thread-N` like the JVM, but
//! which worker takes which file is our own scheduling.

pub mod args;
pub mod baseline;
mod clikt;
pub mod command_line;
pub(crate) use ktrs_compose::jar as compose_jar;
pub mod console;
pub mod file_utils;
pub mod gradle;
mod hand_off_args;
pub(crate) mod jar_providers;
mod java_printf;
pub use ktrs_editorconfig::java_glob;
pub mod jpath;
pub mod ktrs_only;
pub mod ktlint_jar;
mod legacy_rule_set;
pub mod logger;
mod parallel;
mod patterns;
mod process;
pub mod reporter;
mod reporter_aggregator;
mod run;
pub(crate) use ktrs_compose::jar::sha256;
mod subcommands;
pub mod version;

pub use command_line::{ExitCode, KtlintCli};

/// The `ktlint` binary: `args` without the program name; returns the exit code.
pub fn main(args: &[String]) -> i32 {
    ktrs_lint::engine::silence_caught_rule_panics();
    KtlintCli::from_env().run(args)
}
