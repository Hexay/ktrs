//! Port of ktfmt's `cli` package (v0.64): the `ktfmt` binary's flags, messages and exit codes.

pub mod editor_config_resolver;
pub mod main;
pub mod parsed_args;

pub use main::{Main, expand_args_to_file_names};
pub use parsed_args::{HELP_TEXT, KTFMT_VERSION, ParseResult, ParsedArgs, parse_options, process_args};
