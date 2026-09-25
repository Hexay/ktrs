#![allow(dead_code)]

use std::path::{Path, PathBuf};

use ktrs_lexer::Token;

pub fn testdata(sub: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/kotlin")
        .join(sub)
}

/// Files under `dir` (recursive) with one of `exts`, sorted.
pub fn walk(dir: &Path, exts: &[&str]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| exts.contains(&e))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// `StringUtil.convertLineSeparators`: `\r\n` and lone `\r` become `\n`.
pub fn convert_line_separators(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// `(kind, text)` per token; panics unless the tokens cover `text` exactly.
pub fn spans<'t>(text: &'t str, tokens: &[Token]) -> Vec<(ktrs_syntax::SyntaxKind, &'t str)> {
    let mut pos = 0usize;
    let out = tokens
        .iter()
        .map(|t| {
            let end = pos + t.len as usize;
            let span = (t.kind, &text[pos..end]);
            pos = end;
            span
        })
        .collect();
    assert_eq!(pos, text.len(), "tokens do not cover the input");
    out
}
