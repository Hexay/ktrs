//! `ImportPath.pathStr`/`hasAlias`/`toString` and the `FqName` rendering behind `pathStr`
//! (`NameRenderingUtils.render(FqNameUnsafe)`), for ktlint's import rules.

use std::fmt;
use std::hash::{Hash, Hasher};

use ktrs_parser::kt_tokens::KEYWORDS;

use super::file::{FqName, ImportPath};

impl Hash for FqName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_string().hash(state);
    }
}

impl FqName {
    /// `pathSegments()`: `parent().pathSegments() + shortName()`, root first.
    pub fn path_segments(&self) -> Vec<String> {
        let mut segments = Vec::new();
        let mut current = self.clone();
        while let (Some(short_name), Some(parent)) = (current.short_name(), current.parent()) {
            segments.push(short_name);
            current = parent;
        }
        segments.reverse();
        segments
    }

    /// `NameRenderingUtils.render(toUnsafe())`: the segments joined by `.`, each backticked when needed.
    pub fn render(&self) -> String {
        self.path_segments().iter().map(|s| render_name(s)).collect::<Vec<_>>().join(".")
    }
}

/// `NameRenderingUtils.render(Name)`.
pub fn render_name(name: &str) -> String {
    if should_be_escaped(name) { format!("`{name}`") } else { name.to_owned() }
}

fn should_be_escaped(name: &str) -> bool {
    KEYWORDS.types().any(|k| k.keyword_text() == Some(name))
        || name.encode_utf16().any(|c| !is_letter_or_digit(c) && c != u16::from(b'_'))
        || name.is_empty()
        || !name.chars().next().is_some_and(is_java_identifier_start)
}

// Gotcha: Java's per-UTF-16-unit `isLetterOrDigit` (L*, Nd); surrogate halves are never letters.
fn is_letter_or_digit(unit: u16) -> bool {
    char::from_u32(u32::from(unit)).is_some_and(|c| c.is_alphabetic() || c.is_numeric())
}

fn is_java_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

impl ImportPath {
    /// `getPathStr()`.
    pub fn path_str(&self) -> String {
        let mut path = self.fq_name.render();
        if self.is_all_under {
            path.push_str(".*");
        }
        path
    }

    pub fn has_alias(&self) -> bool {
        self.alias.is_some()
    }
}

/// `toString()`: `pathStr`, then ` as <alias>`.
impl fmt::Display for ImportPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.path_str())?;
        if let Some(alias) = &self.alias {
            write!(f, " as {alias}")?;
        }
        Ok(())
    }
}
