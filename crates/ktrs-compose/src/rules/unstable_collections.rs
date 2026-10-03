//! Port of `rules/UnstableCollections.kt`.

use std::sync::LazyLock;

use ktrs_ast::Ast;
use ktrs_ast::psi::KtFunction;
use regex::Regex;

use crate::core::compose_kt_config::ComposeKtConfig;
use crate::core::compose_kt_visitor::ComposeKtVisitor;
use crate::core::emitter::Emitter;
use crate::core::util::kt_callable_declarations::is_type_unstable_collection;

pub struct UnstableCollections;

impl ComposeKtVisitor for UnstableCollections {
    fn is_opt_in(&self) -> bool {
        true
    }

    fn visit_composable(&self, ast: &mut Ast, function: KtFunction, emitter: &mut dyn Emitter, _config: &dyn ComposeKtConfig) {
        let params: Vec<_> =
            function.value_parameters(ast).into_iter().filter(|it| is_type_unstable_collection(ast, it.node())).collect();
        for param in params {
            let variable_name = param.name_as_safe_name(ast);
            let type_reference = param.type_reference(ast);
            let ty = type_reference.map_or_else(|| "List/Set/Map".to_owned(), |t| t.text(ast));
            let raw_type = DIAMOND_REGEX.replace_all(&ty, "");
            let message = create_error_message(&ty, &raw_type, &variable_name);
            emitter.report(ast, type_reference.map_or(param.node(), |t| t.node()), &message, false);
        }
    }
}

/// `Regex("<.*>\\??")`, with Java's `.` (no line terminator).
static DIAMOND_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("<[^\n\r\u{85}\u{2028}\u{2029}]*>\\??").unwrap());

/// `String.capitalized`: `replaceFirstChar { if (it.isLowerCase()) it.titlecase(Locale.getDefault()) else .. }`.
fn capitalized(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else { return String::new() };
    if !first.is_lowercase() {
        return s.to_owned();
    }
    titlecase(first) + chars.as_str()
}

/// Kotlin `Char.titlecase()`: the uppercase with its tail lowercased when it expands, else `toTitleCase`.
fn titlecase(c: char) -> String {
    let upper: String = c.to_uppercase().collect();
    if upper.chars().count() > 1 {
        if c == '\u{149}' {
            return upper;
        }
        let mut chars = upper.chars();
        let head = chars.next().unwrap();
        return format!("{head}{}", chars.as_str().to_lowercase());
    }
    // The Latin digraphs are the only lowercase letters whose title case is not their upper case.
    match c {
        '\u{1C6}' => '\u{1C5}'.to_string(),
        '\u{1C9}' => '\u{1C8}'.to_string(),
        '\u{1CC}' => '\u{1CB}'.to_string(),
        '\u{1F3}' => '\u{1F2}'.to_string(),
        _ => upper,
    }
}

pub fn create_error_message(ty: &str, raw_type: &str, variable: &str) -> String {
    format!(
        "The Compose Compiler cannot infer the stability of a parameter if a {ty} is used in it, even if the item type is stable.
You should use Kotlinx Immutable Collections instead: `{variable}: Immutable{ty}` or create an `@Immutable` wrapper for this class: `@Immutable data class {cap}{raw_type}(val items: {ty})`
See https://mrmans0n.github.io/compose-rules/rules/#avoid-using-unstable-collections for more information.",
        cap = capitalized(variable),
    )
}
