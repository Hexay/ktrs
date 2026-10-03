//! The `ktlint` CLI on fuzzer finds (research/24-fuzzing.md); expected stderr is the 2.0.0-ALPHA-4 jar's
//! without stack frames.

mod ktlint_support;

use ktlint_support::Project;
use ktrs_cli::ktlint::console::LINE_SEPARATOR as LS;

const MISSED: &str = "Tokens [RPAR] were not inserted into the tree. Language: kotlin";

fn missed_tokens_log() -> String {
    format!("ERROR: {MISSED}\nDetails:\nmissedTokensFragment.txt\n{{fun<)]<T:@( {{}}){LS}")
}

/// Finding 4: the parser's `AssertionError`, logged by IntelliJ's `DefaultLogger`, ends the run (wrapped by the
/// thread pool for a file); it beats the parse error before it.
#[test]
fn missed_tokens_end_the_run_with_the_assertion_error() {
    for text in ["{fun<)]<T:@( {})", "val a = )\nfun f() = {fun<)]<T:@( {})"] {
        let project = Project::new("fuzz", &[("t.kt", text)]);
        let run = project.run(&["t.kt"]);
        assert_eq!((run.exit_code, run.out.as_str()), (1, ""), "{run:?}");
        let crash = format!("Exception in thread \"main\" java.util.concurrent.ExecutionException: java.lang.AssertionError: {MISSED}{LS}");
        assert_eq!(run.err, format!("{}{crash}", missed_tokens_log()));

        let run = project.run_with_stdin(&["--stdin", "-F"], text.as_bytes());
        assert_eq!(run.exit_code, 1, "{run:?}");
        assert_eq!(run.err, format!("{}Exception in thread \"main\" java.lang.AssertionError: {MISSED}{LS}", missed_tokens_log()));
    }
}
