//! Port of ktfmt's `cli` package (v0.65): the `ktfmt` binary's flags, messages and exit codes.
//!
//! Deviations:
//! - `--experimental-engine` (undocumented upstream, absent from `--help`) prints upstream's warning and is then
//!   rejected with exit code 1: the experimental kotlinlang formatters (`format/visitor/kotlinlang`) are not ported.
//! - Each file's messages are printed in file order, not in completion order (`main.rs`).

pub use ktrs_fmt::editor_config_resolver;
mod files;
mod java_int;
pub mod main;
pub mod parsed_args;

pub use files::expand_args_to_file_names;
pub use main::Main;
pub use parsed_args::{
    ArgsException, HELP_TEXT, KTFMT_VERSION, ParseResult, ParsedArgs, parse_options, process_args,
};

/// The `ktfmt` binary: `args` without the program name; returns the exit code.
pub fn main(args: &[String]) -> i32 {
    ktrs_syntax::caught_panic::silence_caught_panics();
    Main::new(std::io::stdin(), std::io::stdout(), std::io::stderr()).run(args)
}
