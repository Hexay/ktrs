//! `java.nio.file.Path` semantics that ktlint's file handling depends on, over `/`-separated strings (on
//! Windows, `\` is read as `/`): parsing drops redundant separators, `resolve` keeps `.`/`..`, `normalize`
//! is lexical, `startsWith` compares whole names.

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JPath {
    /// `""` (relative), `"/"` or, on Windows, `"C:/"`.
    root: String,
    names: Vec<String>,
}

pub const ON_WINDOWS: bool = cfg!(windows);

impl JPath {
    /// `Paths.get(s)`; `None` where Windows throws `InvalidPathException` (wildcards and the like).
    pub fn parse(s: &str) -> Option<JPath> {
        if s.contains('\0') {
            return None;
        }
        let s = if ON_WINDOWS { s.replace('\\', "/") } else { s.to_owned() };
        let (root, rest) = split_root(&s);
        if ON_WINDOWS && rest.chars().any(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*')) {
            return None;
        }
        let names = rest.split('/').filter(|n| !n.is_empty()).map(str::to_owned).collect();
        Some(JPath { root, names })
    }

    pub fn from_path(path: &std::path::Path) -> JPath {
        JPath::parse(&path.to_string_lossy()).unwrap_or(JPath { root: String::new(), names: Vec::new() })
    }

    pub fn is_absolute(&self) -> bool {
        !self.root.is_empty()
    }

    /// `resolve(other)`: `other` when absolute, else appended (no normalization).
    pub fn resolve(&self, other: &str) -> Option<JPath> {
        let other = JPath::parse(other)?;
        if other.is_absolute() {
            return Some(other);
        }
        let mut names = self.names.clone();
        names.extend(other.names);
        Some(JPath { root: self.root.clone(), names })
    }

    pub fn normalize(&self) -> JPath {
        let mut names: Vec<String> = Vec::new();
        for name in &self.names {
            match name.as_str() {
                "." => {}
                ".." if names.last().is_some_and(|n| n != "..") => {
                    names.pop();
                }
                ".." if self.is_absolute() => {}
                _ => names.push(name.clone()),
            }
        }
        JPath { root: self.root.clone(), names }
    }

    pub fn starts_with(&self, other: &JPath) -> bool {
        eq_root(&self.root, &other.root)
            && self.names.len() >= other.names.len()
            && self.names.iter().zip(&other.names).all(|(a, b)| eq_name(a, b))
    }

    pub fn parent(&self) -> Option<JPath> {
        if self.names.is_empty() || (self.root.is_empty() && self.names.len() == 1) {
            return None;
        }
        Some(JPath { root: self.root.clone(), names: self.names[..self.names.len() - 1].to_vec() })
    }

    pub fn file_name(&self) -> Option<&str> {
        self.names.last().map(String::as_str)
    }

    /// Kotlin's `relativeToOrSelf(base)` (`PathRelativizer`): `None` where it would throw.
    pub fn relative_to(&self, base: &JPath) -> Option<JPath> {
        let (bn, pn) = (base.normalize(), self.normalize());
        if !eq_root(&bn.root, &pn.root) {
            return None;
        }
        let common = bn.names.iter().zip(&pn.names).take_while(|(a, b)| eq_name(a, b)).count();
        if bn.names[common..].iter().any(|n| n == "..") {
            return None;
        }
        let mut names: Vec<String> = bn.names[common..].iter().map(|_| "..".to_owned()).collect();
        names.extend(pn.names[common..].iter().cloned());
        Some(JPath { root: String::new(), names })
    }

    pub fn relative_to_or_self(&self, base: &JPath) -> JPath {
        self.relative_to(base).unwrap_or_else(|| self.clone())
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }
}

impl std::fmt::Display for JPath {
    /// `pathString` with `/` separators (ktlint replaces `File.separatorChar` with `/` before showing one).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.root, self.names.join("/"))
    }
}

fn split_root(s: &str) -> (String, &str) {
    let bytes = s.as_bytes();
    if ON_WINDOWS && bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        let rest = &s[2..];
        return (format!("{}/", &s[..2].to_ascii_uppercase()), rest.trim_start_matches('/'));
    }
    if s.starts_with('/') { ("/".to_owned(), s) } else { (String::new(), s) }
}

fn eq_root(a: &str, b: &str) -> bool {
    if ON_WINDOWS { a.eq_ignore_ascii_case(b) } else { a == b }
}

fn eq_name(a: &str, b: &str) -> bool {
    if ON_WINDOWS { a.eq_ignore_ascii_case(b) } else { a == b }
}
