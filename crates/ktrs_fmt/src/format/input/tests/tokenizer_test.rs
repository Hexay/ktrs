//! Port of ktfmt's `TokenizerTest.kt`.

use ktrs_parser::FileKind;

use crate::doc::Tok;
use crate::format::input::whitespace_tombstones::SPACE_TOMBSTONE;
use crate::format::input::{KotlinTok, ParseError, Tokenizer};

fn tokenize(code: &str) -> Result<Vec<KotlinTok>, ParseError> {
    let parse = ktrs_parser::parse_file(code, FileKind::Script);
    assert!(
        !parse.has_errors(),
        "parse errors: {:?}",
        parse.error_messages
    );
    let mut tokenizer = Tokenizer::new(code);
    tokenizer.visit_file(&ktrs_psi::PsiElement::root(parse.tree.clone()))?;
    Ok(tokenizer.toks)
}

fn texts(toks: &[KotlinTok]) -> Vec<&str> {
    toks.iter().map(|t| t.get_original_text()).collect()
}

fn indices(toks: &[KotlinTok]) -> Vec<i32> {
    toks.iter().map(|t| t.get_index()).collect()
}

#[test]
fn psi_white_space_are_split_to_newlines_and_maximal_length_whitespaces() {
    let code = ["val  a = ", "", "     ", "     15"].join("\n");
    let toks = tokenize(&code).unwrap();
    assert_eq!(
        texts(&toks),
        [
            "val", "  ", "a", " ", "=", " ", "\n", "\n", "     ", "\n", "     ", "15"
        ]
    );
}

#[test]
fn strings_are_returned_as_a_single_token() {
    let code = [
        "val a=\"\"\"",
        "  ",
        "   ",
        "    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do ",
        "    Lorem",
        "    ",
        "     ",
        "      \"\"\"",
        "val b=\"lorem ipsum\"",
        "      ",
        "    ",
    ]
    .join("\n");
    let toks = tokenize(&code).unwrap();
    let t = SPACE_TOMBSTONE;
    let string = [
        "\"\"\"".to_string(),
        format!(" {t}"),
        format!("  {t}"),
        format!("    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do{t}"),
        "    Lorem".to_string(),
        format!("   {t}"),
        format!("    {t}"),
        "      \"\"\"".to_string(),
    ]
    .join("\n");
    let expected = [
        "val",
        " ",
        "a",
        "=",
        &string,
        "\n",
        "val",
        " ",
        "b",
        "=",
        "\"lorem ipsum\"",
        "\n",
        "      ",
        "\n",
        "    ",
    ];
    assert_eq!(texts(&toks), expected);
}

#[test]
fn token_index_is_advanced_after_a_string_token() {
    let toks = tokenize("val b=\"a\"\nval a=5\n").unwrap();
    assert_eq!(
        texts(&toks),
        [
            "val", " ", "b", "=", "\"a\"", "\n", "val", " ", "a", "=", "5", "\n"
        ]
    );
    assert_eq!(indices(&toks), [0, -1, 1, 2, 3, -1, 4, -1, 5, 6, 7, -1]);
}

#[test]
fn context_receivers_are_parsed_correctly() {
    let code = "context(Something)\nclass A {\n  context(\n  // Test comment.\n  Logger, Raise<Error>)\n  fun test() {}\n}\n";
    let toks = tokenize(code).unwrap();
    #[rustfmt::skip]
    let expected = [
        "context", "(", "Something", ")", "\n", "class", " ", "A", " ", "{", "\n", "  ", "context", "(",
        "\n", "  ", "// Test comment.", "\n", "  ", "Logger", ",", " ", "Raise", "<", "Error", ">", ")",
        "\n", "  ", "fun", " ", "test", "(", ")", " ", "{", "}", "\n", "}", "\n",
    ];
    assert_eq!(texts(&toks), expected);
    #[rustfmt::skip]
    let expected_indices = [
        0, 1, 2, 3, -1, 4, -1, 5, -1, 6, -1, -1, 7, 8, -1, -1, 9, -1, -1, 10, 11, -1, 12, 13, 14, 15, 16,
        -1, -1, 17, -1, 18, 19, 20, -1, 21, 22, -1, 23, -1,
    ];
    assert_eq!(indices(&toks), expected_indices);
}

#[test]
fn guard_conditions_with_subject_are_parsed_correctly() {
    let code = "fun feedAnimal(animal: Animal) {\n    when (animal) {\n        is Animal.Cat if !animal.mouseHunter -> animal.feedCat()\n        else if animal.eatsPlants -> animal.giveLettuce()\n    }\n}\n";
    let toks = tokenize(code).unwrap();
    #[rustfmt::skip]
    let expected = [
        "fun", " ", "feedAnimal", "(", "animal", ":", " ", "Animal", ")", " ", "{", "\n", "    ", "when",
        " ", "(", "animal", ")", " ", "{", "\n", "        ", "is", " ", "Animal", ".", "Cat", " ", "if",
        " ", "!", "animal", ".", "mouseHunter", " ", "->", " ", "animal", ".", "feedCat", "(", ")", "\n",
        "        ", "else", " ", "if", " ", "animal", ".", "eatsPlants", " ", "->", " ", "animal", ".",
        "giveLettuce", "(", ")", "\n", "    ", "}", "\n", "}", "\n",
    ];
    assert_eq!(texts(&toks), expected);
    #[rustfmt::skip]
    let expected_indices = [
        0, -1, 1, 2, 3, 4, -1, 5, 6, -1, 7, -1, -1, 8, -1, 9, 10, 11, -1, 12, -1, -1, 13, -1, 14, 15, 16,
        -1, 17, -1, 18, 19, 20, 21, -1, 22, -1, 23, 24, 25, 26, 27, -1, -1, 28, -1, 29, -1, 30, 31, 32, -1,
        33, -1, 34, 35, 36, 37, 38, -1, -1, 39, -1, 40, -1,
    ];
    assert_eq!(indices(&toks), expected_indices);
}

#[test]
fn long_binary_expressions_are_parsed_correctly() {
    let code = "//////////////////////////////////////\nfun foo() {\n  val sentence =\n      \"The\" +\n          \"quick\" +\n          (\"brown\" + \"fox\") +\n          \"jumps\" +\n          \"over\" +\n          \"the\" +\n          \"lazy\" +\n          \"dog\"\n}\n";
    let toks = tokenize(code).unwrap();
    let pad = "          ";
    #[rustfmt::skip]
    let expected = [
        "//////////////////////////////////////", "\n", "fun", " ", "foo", "(", ")", " ", "{", "\n", "  ",
        "val", " ", "sentence", " ", "=", "\n", "      ", "\"The\"", " ", "+", "\n", pad, "\"quick\"", " ",
        "+", "\n", pad, "(", "\"brown\"", " ", "+", " ", "\"fox\"", ")", " ", "+", "\n", pad, "\"jumps\"",
        " ", "+", "\n", pad, "\"over\"", " ", "+", "\n", pad, "\"the\"", " ", "+", "\n", pad, "\"lazy\"",
        " ", "+", "\n", pad, "\"dog\"", "\n", "}", "\n",
    ];
    assert_eq!(texts(&toks), expected);
    #[rustfmt::skip]
    let expected_indices = [
        0, -1, 1, -1, 2, 3, 4, -1, 5, -1, -1, 6, -1, 7, -1, 8, -1, -1, 9, -1, 10, -1, -1, 11, -1, 12, -1,
        -1, 13, 14, -1, 15, -1, 16, 17, -1, 18, -1, -1, 19, -1, 20, -1, -1, 21, -1, 22, -1, -1, 23, -1, 24,
        -1, -1, 25, -1, 26, -1, -1, 27, -1, 28, -1,
    ];
    assert_eq!(indices(&toks), expected_indices);
}

#[test]
fn context_parameters_are_parsed_correctly() {
    let code = "context(something: Something)\nclass A {\n  context(\n  // Test comment.\n  logger: Logger, raise: Raise<Error>, _: Ignored)\n  fun test() {}\n}\n";
    let toks = tokenize(code).unwrap();
    #[rustfmt::skip]
    let expected = [
        "context", "(", "something", ":", " ", "Something", ")", "\n", "class", " ", "A", " ", "{", "\n",
        "  ", "context", "(", "\n", "  ", "// Test comment.", "\n", "  ", "logger", ":", " ", "Logger", ",",
        " ", "raise", ":", " ", "Raise", "<", "Error", ">", ",", " ", "_", ":", " ", "Ignored", ")", "\n",
        "  ", "fun", " ", "test", "(", ")", " ", "{", "}", "\n", "}", "\n",
    ];
    assert_eq!(texts(&toks), expected);
    #[rustfmt::skip]
    let expected_indices = [
        0, 1, 2, 3, -1, 4, 5, -1, 6, -1, 7, -1, 8, -1, -1, 9, 10, -1, -1, 11, -1, -1, 12, 13, -1, 14, 15,
        -1, 16, 17, -1, 18, 19, 20, 21, 22, -1, 23, 24, -1, 25, 26, -1, -1, 27, -1, 28, 29, 30, -1, 31, 32,
        -1, 33, -1,
    ];
    assert_eq!(indices(&toks), expected_indices);
}

fn assert_parse_error(code: &str, message: Option<&str>) {
    let result = tokenize(code);
    match message {
        None => assert!(result.is_ok(), "unexpected {:?}", result.err()),
        Some(message) => assert_eq!(result.unwrap_err().to_string(), message),
    }
}

#[test]
fn unclosed_comment_obvious() {
    assert_parse_error(
        "package a.b\n/*\nclass A {}\n",
        Some("2:1: error: Unclosed comment"),
    );
}

#[test]
fn unclosed_comment_too_short() {
    assert_parse_error(
        "package a.b\n/*/\nclass A {}\n",
        Some("2:1: error: Unclosed comment"),
    );
}

#[test]
fn unclosed_comment_nested() {
    assert_parse_error(
        "package a.b\n/* /* */\nclass A {}\n",
        Some("2:1: error: Unclosed comment"),
    );
}

#[test]
fn unclosed_comment_nested_eof() {
    // TODO: https://youtrack.jetbrains.com/issue/KT-72887 - This should be an error.
    assert_parse_error("package a.b\nclass A {}\n/* /* */", None);
}
