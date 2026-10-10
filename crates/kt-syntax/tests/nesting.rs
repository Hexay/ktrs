//! The stack guard of `parse` (research/36, "Very deep nesting"): every case runs on a thread whose stack is far
//! too small for the parser's recursion, so a missing or undersized stack budget aborts the test binary.

use std::thread;

use kt_syntax::{MAX_NESTING_DEPTH, ParseError, SyntaxKind, parse};
use ktrs_parser::{FileKind, parse_file};
use ktrs_syntax::psi_dump;

const SMALL_STACK: usize = 256 << 10;
/// For the unguarded parser and the recursive `psi_dump` the facade's output is compared with.
const HUGE_STACK: usize = 512 << 20;

fn on_stack<R: Send + 'static>(size: usize, f: impl FnOnce() -> R + Send + 'static) -> R {
    thread::Builder::new().stack_size(size).spawn(f).unwrap().join().unwrap()
}

/// Bracket shapes: `depth` levels of one bracket kind each.
fn bracket_shapes(depth: usize) -> Vec<(&'static str, String)> {
    let nest = |prefix: &str, open: &str, inner: &str, close: &str| format!("{prefix}{}{inner}{}\n", open.repeat(depth), close.repeat(depth));
    vec![
        ("lambdas", nest("val x = ", "run { ", "1", " }")),
        ("bare lambdas", nest("val x = ", "{", "", "}")),
        ("parentheses", nest("val x = ", "(", "1", ")")),
        ("call arguments", nest("val x = ", "f(", "1", ")")),
        ("indexing", nest("val x = ", "a[", "1", "]")),
        ("if blocks", nest("val x = ", "if (a) { ", "", "}")),
        ("classes", nest("", "class A { ", "", "}")),
        ("templates", nest("val x = ", "\"${", "1", "}\"")),
        ("when branches", nest("val x = ", "when (a) { else -> ", "1", " }")),
        // Not here: parenthesized function types, `((Int) -> Unit) -> Unit`. The parser's time explodes with their depth (research/36).
    ]
}

fn facade_dump_on_small_stack(text: String) -> Result<String, ParseError> {
    on_stack(SMALL_STACK, move || parse(&text).map(|file| file.dump()))
}

#[test]
fn nesting_up_to_the_limit_parses_on_a_small_stack_with_the_parser_s_tree() {
    let mixed = format!("val x = {}1{}\n", "f(a[{ ".repeat(MAX_NESTING_DEPTH / 3), " }])".repeat(MAX_NESTING_DEPTH / 3));
    for (name, text) in bracket_shapes(MAX_NESTING_DEPTH).into_iter().chain([("mixed", mixed)]) {
        let dump = facade_dump_on_small_stack(text.clone()).unwrap_or_else(|e| panic!("{name}: {e}"));
        let raw = on_stack(HUGE_STACK, move || psi_dump(&parse_file(&text, FileKind::Source), ""));
        assert!(dump == raw, "{name}: the guard changed the tree");
    }
}

#[test]
fn nesting_past_the_limit_is_an_error_not_an_abort() {
    for (name, text) in bracket_shapes(MAX_NESTING_DEPTH + 1) {
        let result = on_stack(SMALL_STACK, move || parse(&text).map(|_| ()));
        assert!(matches!(result, Err(ParseError::TooDeeplyNested { .. })), "{name}: {result:?}");
    }
    let far_past = format!("val x = {}", "(".repeat(1_000_000));
    let result = on_stack(SMALL_STACK, move || parse(&far_past).map(|_| ()));
    assert_eq!(result, Err(ParseError::TooDeeplyNested { offset: 8 + MAX_NESTING_DEPTH }));
    assert_eq!(result.unwrap_err().to_string(), format!("brackets nested deeper than 1000 at offset {}", 8 + MAX_NESTING_DEPTH));
}

#[test]
fn many_shallow_brackets_are_not_nesting() {
    let text = "val x = f(a[0]) { it }\n".repeat(5 * MAX_NESTING_DEPTH);
    let functions = on_stack(SMALL_STACK, move || parse(&text).unwrap().root().find_all(&[SyntaxKind::PROPERTY]).count());
    assert_eq!(functions, 5 * MAX_NESTING_DEPTH);
}

/// Recursion that opens no bracket is not counted by the limit; the stack budget grows with the input instead.
#[test]
fn bracket_free_chains_parse_on_a_small_stack() {
    let chain = |prefix: &str, link: &str, count: usize, suffix: &str| format!("{prefix}{}{suffix}\n", link.repeat(count));
    let shapes = [
        ("else if", chain("fun f() { if (a) {}", " else if (a) {}", 3000, " }")),
        ("if without braces", chain("fun f() { ", "if (a) ", 3000, "x }")),
        ("generic types", format!("val x: {}Int{} = y\n", "List<".repeat(3000), ">".repeat(3000))),
        ("prefix minus", chain("val x = ", "-", 60_000, "1")),
        ("prefix not", chain("val x = ", "!", 60_000, "a")),
        ("labels", chain("fun f() { ", "a@ ", 3000, "x }")),
        ("annotations", chain("fun f() { ", "@A ", 3000, "x }")),
        ("elvis return", chain("fun f() { ", "return a ?: ", 3000, "x }")),
        ("assignments", chain("fun f() { ", "a = ", 3000, "x }")),
        ("function type arrows", chain("val x: ", "(A) -> ", 3000, "A = y")),
        ("qualified calls", chain("val x = a", ".b()", 5000, "")),
    ];
    for (name, text) in shapes {
        let len = text.len();
        let spelled = on_stack(SMALL_STACK, move || parse(&text).map(|file| file.root().tokens().map(|t| t.text().len()).sum::<usize>()));
        assert_eq!(spelled, Ok(len), "{name}");
    }
}
