//! Port of `core/util/KotlinUtils.kt`. The collection helpers (`runIf`, `runIfNotNull`, `mapIf`, `mapFirst`,
//! `mapSecond`, `uniquePairs`) are plain iterator adapters in Rust and are not ported; write them inline.

use ktrs_lint::rules::internal::KotlinRegex;
use ktrs_psi::FqName;

/// `FqName + String`: `FqName(asString() + "." + other)`.
pub fn fq_name_plus(fq_name: &FqName, other: &str) -> FqName {
    FqName::new(&format!("{}.{other}", fq_name.as_string()))
}

/// `String?.matchesAnyOf(patterns)`: a full match of any pattern; never for null or empty.
pub fn matches_any_of(text: Option<&str>, patterns: &[KotlinRegex]) -> bool {
    let Some(text) = text.filter(|t| !t.is_empty()) else { return false };
    patterns.iter().any(|regex| regex.matches(text))
}

pub fn join_to_regex_or_null(set: &[String]) -> Option<KotlinRegex> {
    if set.is_empty() { None } else { Some(join_to_regex(set)) }
}

/// `Set<String>.joinToRegex()`: `(a|b|...)`.
pub fn join_to_regex(set: &[String]) -> KotlinRegex {
    KotlinRegex::new(&format!("({})", set.join("|")))
}

/// `String.toCamelCase()`: `_`-separated words, each with its first char titlecased.
pub fn to_camel_case(text: &str) -> String {
    text.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) if first.is_lowercase() => first.to_uppercase().chain(chars).collect(),
                Some(first) => std::iter::once(first).chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

/// `String.toSnakeCase()`: `_` before every uppercase char that follows another char (`(?<=.)(?=\p{Upper})`;
/// `.` does not match a line terminator), then lowercased.
pub fn to_snake_case(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 4);
    let mut previous: Option<char> = None;
    for c in text.chars() {
        if c.is_uppercase() && previous.is_some_and(|p| !matches!(p, '\n' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}')) {
            out.push('_');
        }
        out.push(c);
        previous = Some(c);
    }
    out.to_lowercase()
}

pub const KOTLIN_SCOPE_FUNCTIONS: &[&str] = &["with", "apply", "run", "also", "let"];
pub const KOTLIN_IT_OBJECT_SCOPE_FUNCTIONS: &[&str] = &["let", "also"];
