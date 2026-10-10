//! A codemod with text edits: renames every call of an unqualified function.
//!
//! `cargo run -p kt-syntax --example rename_call -- <old> <new> <file.kt>... [--write]` prints the rewritten text
//! of each file, or rewrites the files in place with `--write`.

use std::process::ExitCode;
use std::{env, fs};

use kt_syntax::{SourceFile, SyntaxKind, TextEdit, parse};

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let write = args.iter().any(|arg| arg == "--write");
    args.retain(|arg| arg != "--write");
    let [old, new, paths @ ..] = &args[..] else {
        eprintln!("usage: rename_call <old> <new> <file.kt>... [--write]");
        return ExitCode::from(2);
    };
    for path in paths {
        let result = fs::read_to_string(path).map_err(|e| e.to_string()).and_then(|text| rename_calls(&text, old, new));
        match result {
            Ok(Some(rewritten)) if write => fs::write(path, rewritten).unwrap(),
            Ok(Some(rewritten)) => print!("{rewritten}"),
            Ok(None) => {}
            Err(e) => {
                eprintln!("{path}: {e}");
                return ExitCode::from(2);
            }
        }
    }
    ExitCode::SUCCESS
}

/// `text` with calls of `old` renamed to `new`, in the input's own line endings; `None` if nothing changes.
fn rename_calls(text: &str, old: &str, new: &str) -> Result<Option<String>, String> {
    let file = parse(text).map_err(|e| e.to_string())?;
    let edits = call_renames(&file, old, new);
    if edits.is_empty() {
        return Ok(None);
    }
    let mut rewritten = file.apply_edits(edits).map_err(|e| e.to_string())?;
    // The tree is over the normalized text: put back what `parse` took away.
    if file.had_crlf() {
        rewritten = rewritten.replace('\n', "\r\n");
    }
    if file.had_bom() {
        rewritten.insert(0, '\u{feff}');
    }
    Ok(Some(rewritten))
}

/// One edit per `old(..)` or `old { .. }` call whose callee is a plain name (not `x.old()`).
fn call_renames(file: &SourceFile, old: &str, new: &str) -> Vec<TextEdit> {
    file.root()
        .find_all(&[SyntaxKind::CALL_EXPRESSION])
        .filter(|call| call.prev_sibling().is_none_or(|before| !matches!(before.kind(), SyntaxKind::DOT | SyntaxKind::SAFE_ACCESS)))
        .filter_map(|call| call.first_child())
        .filter(|callee| callee.kind() == SyntaxKind::REFERENCE_EXPRESSION && callee.text() == old)
        .map(|callee| callee.replace(new))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_unqualified_calls_only() {
        let text = "fun f() {\n    old(1)\n    old { old() }\n    x.old()\n    val old = \"old()\" // old()\n}\n";
        let expected = "fun f() {\n    new(1)\n    new { new() }\n    x.old()\n    val old = \"old()\" // old()\n}\n";
        assert_eq!(rename_calls(text, "old", "new").unwrap().as_deref(), Some(expected));
        assert_eq!(rename_calls(expected, "old", "new").unwrap(), None);
    }

    #[test]
    fn keeps_the_byte_order_mark_and_crlf() {
        let rewritten = rename_calls("\u{feff}val a = old()\r\nval b = 1\r\n", "old", "renamed").unwrap().unwrap();
        assert_eq!(rewritten, "\u{feff}val a = renamed()\r\nval b = 1\r\n");
    }
}
