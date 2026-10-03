//! Port of the ktlint 2.0.0-ALPHA-4 CLI (`ktlint-cli`): the `ktlint` binary's flags, messages, reporters
//! and exit codes, over ktrs-lint's engine and standard rules. Differential test: tools/ktlint-oracle/cli-diff.sh.
//! With `--ktlint-version=1.8` (or `ktrs_ktlint_version = 1.8`) it is the 1.8.0 CLI instead: [`version`].
//!
//! A run that loads a rule set (`-R`) or reporter (`artifact=`) JAR with JVM code is handed to the real ktlint
//! jar ([`ktlint_jar`]). Deviations: `--log-level=debug|trace` prints only the
//! CLI's own messages, not the engine's; help text is wrapped for 80 columns whatever the terminal; a rule
//! crash shows the panic, not a JVM stack trace.

pub mod args;
pub mod baseline;
mod clikt;
pub mod command_line;
pub(crate) mod compose_jar;
pub mod console;
pub mod file_utils;
pub mod gradle;
mod hand_off_args;
pub(crate) mod jar_providers;
mod java_printf;
pub mod java_glob;
pub mod jpath;
pub mod ktlint_jar;
mod legacy_rule_set;
pub mod logger;
mod parallel;
mod patterns;
mod process;
pub mod reporter;
mod reporter_aggregator;
mod run;
mod sha256;
mod subcommands;
pub mod version;
mod zip_directory;

pub use command_line::{ExitCode, KtlintCli};

/// The `ktlint` binary: `args` without the program name; returns the exit code.
pub fn main(args: &[String]) -> i32 {
    ktrs_lint::engine::silence_caught_rule_panics();
    KtlintCli::from_env().run(args)
}
