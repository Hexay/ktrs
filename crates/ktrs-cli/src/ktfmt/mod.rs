//! Port of ktfmt's `cli` package (v0.64): the `ktfmt` binary's flags, messages and exit codes.

pub use ktrs_fmt::editor_config_resolver;
pub mod main;
pub mod parsed_args;

pub use main::{Main, expand_args_to_file_names};
pub use parsed_args::{HELP_TEXT, KTFMT_VERSION, ParseResult, ParsedArgs, parse_options, process_args};

/// The `ktfmt` binary: `args` without the program name; returns the exit code.
pub fn main(args: &[String]) -> i32 {
    ktrs_syntax::caught_panic::silence_caught_panics();
    Main::new(std::io::stdin(), std::io::stdout(), std::io::stderr()).run(args)
}
