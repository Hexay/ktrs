//! The open documents, and `file:` URIs as paths.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use ktrs_lint::LintError;
use lsp_types::Uri;

pub(crate) struct Document {
    pub text: String,
    pub version: i32,
    /// `None` for a document that is not a file (e.g. `untitled:`).
    pub path: Option<PathBuf>,
    /// The lint errors of `text`, once linted.
    pub lint_errors: Option<Vec<LintError>>,
}

#[derive(Default)]
pub(crate) struct Documents {
    open: HashMap<Uri, Document>,
    /// Documents whose diagnostics are out of date, in the order they changed.
    stale: Vec<Uri>,
}

impl Documents {
    pub(crate) fn open(&mut self, uri: Uri, text: String, version: i32) {
        let path = uri_to_path(&uri);
        self.open.insert(uri.clone(), Document { text, version, path, lint_errors: None });
        self.mark_stale(uri);
    }

    pub(crate) fn change(&mut self, uri: &Uri, text: String, version: i32) {
        if let Some(document) = self.open.get_mut(uri) {
            (document.text, document.version, document.lint_errors) = (text, version, None);
            self.mark_stale(uri.clone());
        }
    }

    pub(crate) fn close(&mut self, uri: &Uri) {
        self.open.remove(uri);
        self.stale.retain(|u| u != uri);
    }

    pub(crate) fn get(&self, uri: &Uri) -> Option<&Document> {
        self.open.get(uri)
    }

    pub(crate) fn get_mut(&mut self, uri: &Uri) -> Option<&mut Document> {
        self.open.get_mut(uri)
    }

    pub(crate) fn mark_stale(&mut self, uri: Uri) {
        if !self.stale.contains(&uri) {
            self.stale.push(uri);
        }
    }

    /// Marks every open document stale, dropping its lint errors (the configuration changed).
    pub(crate) fn invalidate_all(&mut self) {
        let uris: Vec<Uri> = self.open.keys().cloned().collect();
        for uri in uris {
            if let Some(document) = self.open.get_mut(&uri) {
                document.lint_errors = None;
            }
            self.mark_stale(uri);
        }
    }

    pub(crate) fn take_stale(&mut self) -> Vec<Uri> {
        std::mem::take(&mut self.stale)
    }
}

/// The path of a `file:` URI (percent-decoded; `/C:/x` is `C:/x` on Windows).
pub fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    let rest = uri.as_str().strip_prefix("file://")?;
    let path = &rest[rest.find('/')?..];
    let decoded = percent_decode(path.split(['?', '#']).next().unwrap_or(path))?;
    let decoded = match decoded.as_bytes() {
        [b'/', drive, b':', ..] if cfg!(windows) && drive.is_ascii_alphabetic() => decoded[1..].to_owned(),
        _ => decoded,
    };
    Some(PathBuf::from(decoded))
}

/// The `file:` URI of an absolute path.
pub fn path_to_uri(path: &Path) -> Uri {
    let path = path.to_string_lossy().replace('\\', "/");
    let path = if path.starts_with('/') { path } else { format!("/{path}") };
    let mut encoded = String::from("file://");
    for b in path.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'.' | b'_' | b'~' | b':' => encoded.push(b as char),
            _ => encoded.push_str(&format!("%{b:02X}")),
        }
    }
    Uri::from_str(&encoded).expect("a percent-encoded path is a valid URI")
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            decoded.push(u8::from_str_radix(s.get(i + 1..i + 3)?, 16).ok()?);
            i += 3;
        } else {
            decoded.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uris_round_trip() {
        let path = if cfg!(windows) { PathBuf::from("C:/a b/ü#.kt") } else { PathBuf::from("/a b/ü#.kt") };
        assert_eq!(uri_to_path(&path_to_uri(&path)), Some(path));
        let vscode = Uri::from_str("file:///c%3A/x/A.kt").unwrap();
        let expected = if cfg!(windows) { "c:/x/A.kt" } else { "/c:/x/A.kt" };
        assert_eq!(uri_to_path(&vscode), Some(PathBuf::from(expected)));
        assert_eq!(uri_to_path(&Uri::from_str("untitled:Untitled-1").unwrap()), None);
    }
}
