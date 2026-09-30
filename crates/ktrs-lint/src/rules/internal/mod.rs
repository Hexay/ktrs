//! Ports of ktlint-ruleset-standard `rules/internal/`, plus the Kotlin/Java text semantics (`Regex`, `Char`
//! predicates, `trim`) the rules depend on.

pub mod importordering;
pub mod kotlin_string;
mod reg_ex_ignoring_diacritics_and_strokes_on_letters;

pub use reg_ex_ignoring_diacritics_and_strokes_on_letters::{KotlinRegex, reg_ex_ignoring_diacritics_and_strokes_on_letters};

use ktrs_parser::kt_tokens::{KEYWORDS, SOFT_KEYWORDS};

/// The `KEYWORDS` companion value of the naming rules: the debug names of `KtTokens.KEYWORDS` and
/// `KtTokens.SOFT_KEYWORDS`, minus those with an uppercase character (`AS_SAFE`, `NOT_IN`, ...).
pub fn is_keyword(text: &str) -> bool {
    KEYWORDS
        .types()
        .chain(SOFT_KEYWORDS.types())
        .map(|k| k.debug_name())
        .filter(|keyword| !keyword.chars().any(char::is_uppercase))
        .any(|keyword| keyword == text)
}
