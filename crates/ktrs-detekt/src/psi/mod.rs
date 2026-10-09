//! `detekt-psi-utils`: the helpers syntax-only rules share. Ported so far: what the rules in `crate::rules` call.

mod allowed_exception_name_pattern;
mod glob_to_regex;
mod is_part_of_utils;
mod kt_annotated_extensions;
mod kt_modifier_list;
mod method_signature;
mod string_extensions;

pub use allowed_exception_name_pattern::is_allowed_exception_name;
pub use glob_to_regex::path_glob_to_regex;
pub use is_part_of_utils::is_part_of;
pub use kt_annotated_extensions::has_annotation;
pub use kt_modifier_list::*;
pub use method_signature::is_hash_code_function;
pub use string_extensions::{
    last_argument_matches_kotlin_reference_url_syntax, last_argument_matches_markdown_url_syntax, last_argument_matches_url,
};
