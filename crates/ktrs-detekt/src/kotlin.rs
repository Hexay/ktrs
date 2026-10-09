//! Kotlin stdlib and JDK behaviour the ported code depends on: `Regex`, case-insensitive `contains`, and
//! `java.net.URL(...).toURI()` acceptance.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use ktrs_lint::rules::internal::KotlinRegex;

/// Kotlin `Regex` (`String.toRegex()`): Java syntax, as far as `KotlinRegex` translates it.
// TODO: lookaround and backreferences (the `regex` crate has neither) need a backtracking engine; a pattern
// using them panics here like a `PatternSyntaxException`.
#[derive(Clone)]
pub struct Regex {
    pattern: String,
    compiled: Rc<KotlinRegex>,
}

thread_local! {
    // Upstream compiles a rule's patterns once per file (a rule instance per file); the cache keeps that from
    // costing a compilation per file here.
    static COMPILED: RefCell<HashMap<String, Rc<KotlinRegex>>> = RefCell::new(HashMap::new());
}

impl Regex {
    pub fn new(pattern: &str) -> Regex {
        let compiled = COMPILED.with(|cache| {
            if let Some(compiled) = cache.borrow().get(pattern) {
                return compiled.clone();
            }
            let compiled = Rc::new(KotlinRegex::new(pattern));
            cache.borrow_mut().insert(pattern.to_owned(), compiled.clone());
            compiled
        });
        Regex { pattern: pattern.to_owned(), compiled }
    }

    /// `Regex.matches(input)` / `String.matches(regex)`: the whole input.
    pub fn matches(&self, input: &str) -> bool {
        self.compiled.matches(input)
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

/// `Regex.toString()`: the pattern.
impl fmt::Display for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.pattern)
    }
}

/// `String.removePrefix(prefix)`.
pub fn remove_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

/// `String.removeSuffix(suffix)`.
pub fn remove_suffix<'a>(text: &'a str, suffix: &str) -> &'a str {
    text.strip_suffix(suffix).unwrap_or(text)
}

/// `String.contains(other, ignoreCase = true)`: Java's per-char `regionMatches(ignoreCase = true)`.
pub fn contains_ignore_case(text: &str, other: &str) -> bool {
    let text: Vec<char> = text.chars().collect();
    let other: Vec<char> = other.chars().collect();
    if other.is_empty() {
        return true;
    }
    text.windows(other.len()).any(|window| window.iter().zip(&other).all(|(&a, &b)| chars_equal_ignore_case(a, b)))
}

fn chars_equal_ignore_case(a: char, b: char) -> bool {
    if a == b {
        return true;
    }
    let (upper_a, upper_b) = (to_upper_case(a), to_upper_case(b));
    upper_a == upper_b || to_lower_case(upper_a) == to_lower_case(upper_b)
}

/// `Character.toUpperCase(char)`: the one-to-one mapping only.
fn to_upper_case(c: char) -> char {
    let mut upper = c.to_uppercase();
    match (upper.next(), upper.next()) {
        (Some(single), None) => single,
        _ => c,
    }
}

fn to_lower_case(c: char) -> char {
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(single), None) => single,
        _ => c,
    }
}

/// Whether `URL(spec).toURI()` succeeds (`runCatching { URL(lastArgument).toURI() }.isSuccess`).
// Approximates the JDK: the protocol must have a built-in handler and the rest must be legal RFC 2396 characters
// with a numeric port; unverified beyond the cases of tools/detekt-oracle/smoke.
pub fn is_url_with_uri(spec: &str) -> bool {
    let spec = spec.trim_matches(|c: char| c <= ' ');
    let Some(colon) = spec.find(':') else { return false };
    let protocol = &spec[..colon];
    let valid_protocol = protocol.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && protocol.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-'));
    if !valid_protocol || protocol.contains('/') {
        return false;
    }
    let rest = &spec[colon + 1..];
    match protocol.to_ascii_lowercase().as_str() {
        "http" | "https" | "ftp" | "file" | "mailto" | "jrt" | "jmod" => {}
        // JarURLConnection: "no !/ in spec"
        "jar" if rest.contains("!/") => {}
        _ => return false,
    }
    if let Some(after_slashes) = rest.strip_prefix("//") {
        let authority = &after_slashes[..after_slashes.find(['/', '?', '#']).unwrap_or(after_slashes.len())];
        // URI: an empty authority needs a path, query or fragment after it.
        if after_slashes.is_empty() {
            return false;
        }
        let host_port = authority.rsplit('@').next().unwrap_or(authority);
        if let Some((_, port)) = host_port.rsplit_once(':')
            && !host_port.ends_with(']')
            && !port.bytes().all(|b| b.is_ascii_digit())
        {
            return false;
        }
    }
    // URI: an opaque URI needs a scheme-specific part.
    if rest.is_empty() {
        return false;
    }
    has_only_uri_characters(rest)
}

fn has_only_uri_characters(text: &str) -> bool {
    let bytes = text.as_bytes();
    let (mut in_query_or_fragment, mut in_fragment) = (false, false);
    for (index, c) in text.char_indices() {
        let legal = match c {
            '%' => bytes.get(index + 1).is_some_and(u8::is_ascii_hexdigit) && bytes.get(index + 2).is_some_and(u8::is_ascii_hexdigit),
            c if c.is_ascii_alphanumeric() => true,
            '#' if in_fragment => false,
            '#' => {
                (in_query_or_fragment, in_fragment) = (true, true);
                true
            }
            '?' => {
                in_query_or_fragment = true;
                true
            }
            // java.net.URI counts the brackets as reserved, which the path does not take.
            '[' | ']' => in_query_or_fragment,
            '_' | '-' | '!' | '.' | '~' | '\'' | '(' | ')' | '*' | ';' | '/' | ':' | '@' | '&' | '=' | '+' | '$' | ',' => true,
            // java.net.URI "other": non-ASCII that is neither a space nor a control character
            c if !c.is_ascii() => !c.is_whitespace() && !c.is_control(),
            _ => false,
        };
        if !legal {
            return false;
        }
    }
    true
}
