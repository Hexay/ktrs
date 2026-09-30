//! Upstream lexer fixtures (`AbstractKotlinLexerTest` / `AbstractKDocLexerTest`): the whole file
//! is fed to one lexer and printed as `KIND ('text')` with `\n` escaped.

mod common;

use common::{convert_line_separators, spans, testdata, walk};
use ktrs_lexer::{Token, tokenize, tokenize_kdoc};

fn print_tokens(text: &str, tokens: &[Token]) -> String {
    spans(text, tokens)
        .into_iter()
        .map(|(kind, t)| format!("{} ('{}')\n", kind.debug_name(), t.replace('\n', "\\n")))
        .collect()
}

fn run_dir(sub: &str, lex: fn(&str) -> Vec<Token>) -> usize {
    let files = walk(&testdata(sub), &["kt"]);
    let mut failures = Vec::new();
    for kt in &files {
        let text = convert_line_separators(&std::fs::read_to_string(kt).unwrap());
        let actual = print_tokens(&text, &lex(&text));
        let expected =
            convert_line_separators(&std::fs::read_to_string(kt.with_extension("txt")).unwrap());
        if actual.trim_end() != expected.trim_end() {
            failures.push(format!(
                "{}\n--- expected\n{}\n--- actual\n{}",
                kt.display(),
                expected,
                actual
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failing fixture(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    files.len()
}

#[test]
fn kotlin_lexer_fixtures() {
    assert_eq!(run_dir("lexer/kotlin", tokenize), 18);
}

#[test]
fn kdoc_lexer_fixtures() {
    assert_eq!(run_dir("lexer/kdoc", tokenize_kdoc), 2);
}
