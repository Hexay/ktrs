//! `GlobToRegex.kt`.

use crate::kotlin::Regex;

/// Convert a simple path pattern String to a Regex: `*` matches any zero or more characters, `?` any one.
pub fn path_glob_to_regex(pattern: &str) -> Regex {
    Regex::new(&pattern.replace('.', "\\.").replace('*', ".*").replace('?', "."))
}

// TODO: fullyQualifiedNameGlobToRegex (lookahead; used by the annotation suppressor and unported rules)
