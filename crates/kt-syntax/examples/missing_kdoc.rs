//! A tiny lint: public functions without a KDoc comment.
//!
//! `cargo run -p kt-syntax --example missing_kdoc -- <file.kt>...` prints `path:line:col: message` per finding
//! and exits with 1 if there are any.

use std::process::ExitCode;
use std::{env, fs};

use kt_syntax::{Node, SourceFile, SyntaxKind, parse, parse_script};

fn main() -> ExitCode {
    let mut findings = 0;
    for path in env::args().skip(1) {
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("{path}: {e}");
                return ExitCode::from(2);
            }
        };
        let parsed = if path.ends_with(".kts") { parse_script(&text) } else { parse(&text) };
        let file = match parsed {
            Ok(file) => file,
            Err(e) => {
                eprintln!("{path}: {e}");
                return ExitCode::from(2);
            }
        };
        for (name, message) in undocumented_functions(&file) {
            // Offsets are bytes of the normalized text; editors want 1-based lines and UTF-16 columns.
            let at = file.line_col_utf16(name.range().start).unwrap();
            println!("{path}:{}:{}: {message}", at.line + 1, at.col + 1);
            findings += 1;
        }
    }
    if findings == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

/// The name token and a message for each public, non-local, non-override function without KDoc.
fn undocumented_functions(file: &SourceFile) -> Vec<(Node<'_>, String)> {
    file.root()
        .find_all(&[SyntaxKind::FUN])
        .filter(|function| function.doc_comment().is_none() && !is_local(*function) && !is_hidden_or_override(*function))
        .filter_map(|function| function.child(SyntaxKind::IDENTIFIER))
        .map(|name| (name, format!("function `{}` has no KDoc", name.text())))
        .collect()
}

fn is_local(function: Node) -> bool {
    function.ancestors().any(|node| matches!(node.kind(), SyntaxKind::BLOCK | SyntaxKind::FUNCTION_LITERAL))
}

fn is_hidden_or_override(function: Node) -> bool {
    let Some(modifiers) = function.child(SyntaxKind::MODIFIER_LIST) else { return false };
    modifiers.children().any(|modifier| {
        matches!(modifier.kind(), SyntaxKind::PRIVATE_KEYWORD | SyntaxKind::INTERNAL_KEYWORD | SyntaxKind::OVERRIDE_KEYWORD)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_public_member_and_top_level_functions_only() {
        let file = parse(
            "/** Documented. */\nfun a() {}\n\nfun b() {\n    fun local() {}\n    run { fun inLambda() {} }\n}\n\nclass C {\n    private fun c() {}\n    override fun d() {}\n    // not KDoc\n    fun e() {}\n}\n",
        )
        .unwrap();
        let found: Vec<(&str, u32)> =
            undocumented_functions(&file).iter().map(|(name, _)| (name.text(), file.line_col(name.range().start).unwrap().line)).collect();
        assert_eq!(found, [("b", 3), ("e", 12)]);
    }
}
