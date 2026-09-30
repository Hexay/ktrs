//! Port of the ktlint 2.0.0-ALPHA-4 CLI (`ktlint-cli`): the `ktlint` binary's flags, messages, reporters
//! and exit codes, over ktrs-lint's engine and standard rules. Differential test: tools/ktlint-oracle/cli-diff.sh.
//!
//! Deviations: rule sets (`-R`) and reporter JARs (`artifact=`) can't be loaded (a JAR declaring the
//! service exits with 6, as a JAR without one does upstream); `--log-level=debug|trace` prints only the
//! CLI's own messages, not the engine's; help text is wrapped for 80 columns whatever the terminal; a rule
//! crash shows the panic, not a JVM stack trace.

pub mod args;
pub mod baseline;
mod clikt;
pub mod command_line;
pub mod console;
pub mod file_utils;
mod jar_providers;
pub mod java_glob;
pub mod jpath;
pub mod logger;
mod parallel;
mod patterns;
mod process;
pub mod reporter;
mod reporter_aggregator;
mod sha256;
mod subcommands;

pub use command_line::{ExitCode, KtlintCli};

/// The `ktlint` binary: `args` without the program name; returns the exit code.
pub fn main(args: &[String]) -> i32 {
    KtlintCli::from_env().run(args)
}
