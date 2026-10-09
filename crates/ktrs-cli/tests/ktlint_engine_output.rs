//! The engine's part of the `ktlint` output, as the 2.0.0-ALPHA-4 jar prints it: a rule crash's cause by its
//! JVM class name, and the engine's WARN log lines with their thread. Inputs are the corpus files that
//! tools/ktlint-oracle/cli-diff.sh runs against the jar.

mod ktlint_support;

use ktlint_support::Project;

const INDENT_CRASH: &str = include_str!("../../../tools/ktlint-oracle/cli-diff-data/example-flow-09.kt");
const NOT_CONVERGING: &str = include_str!("../../../tools/ktlint-oracle/cli-diff-data/AndroidDnsTest.kt");
const TIME: &str = r"\d{2}:\d{2}:\d{2}\.\d{3}";

#[test]
fn format_crash_names_the_cause_by_its_jvm_class() {
    let run = Project::new("crash", &[("Crash.kt", INDENT_CRASH)]).run(&["-F", "--relative"]);
    assert_eq!(run.exit_code, 1, "{run:#?}");
    assert_line!(
        run.out,
        r"Crash\.kt:0:0: Internal Error \(rule 'standard:indent'\) in Crash\.kt at position '0:0\. Please create a ticket at https://github\.com/ktlint/ktlint/issues and provide the source code that triggered an error\."
    );
    assert_line!(run.out, r"io\.github\.ktlint\.core\.rule\.engine\.api\.KtLintRuleException: Rule 'standard:indent' throws exception in file '.*Crash\.kt' at position \(0:0\)");
    assert_line!(run.out, r"Caused by: java\.lang\.IllegalArgumentException: Stack should be empty:");
    assert_line!(run.out, r" \(\)");
    run.assert_error_output_is_empty();
}

#[test]
fn format_crash_on_stdin_logs_the_exception_as_logback_renders_it() {
    let run = Project::new("crash", &[]).run_with_stdin(&["--stdin", "-F"], INDENT_CRASH.as_bytes());
    assert_eq!(run.exit_code, 4, "{run:#?}");
    assert_line!(run.out, &format!(r"{TIME} \[main\] ERROR io\.github\.ktlint\.core\.cli\.internal\.KtlintCommandLine -- kotlin\.Unit"));
    assert_line!(run.out, r"io\.github\.ktlint\.core\.rule\.engine\.api\.KtLintRuleException: Rule 'standard:indent' throws exception in file '<stdin>' at position \(0:0\)");
    assert_line!(run.out, r"Caused by: java\.lang\.IllegalArgumentException: Stack should be empty:");
    assert_no_line!(run.out, r"Internal Error.*");
}

#[test]
fn lint_does_not_crash_on_the_indent_crash_file() {
    let run = Project::new("crash", &[("Crash.kt", INDENT_CRASH)]).run(&["--relative"]);
    assert_no_line!(run.out, r".*Internal Error.*");
    run.assert_error_output_is_empty();
}

#[test]
fn format_crash_in_the_json_reporter_names_the_cause_by_its_jvm_class() {
    let run = Project::new("crash", &[("Crash.kt", INDENT_CRASH)]).run(&["-F", "--relative", "--reporter=json"]);
    assert!(run.out.contains(r"\nCaused by: java.lang.IllegalArgumentException: Stack should be empty:\n"), "{run:#?}");
}

#[test]
fn format_that_does_not_converge_logs_a_warning_from_the_worker_thread() {
    let run = Project::new("converge", &[("AndroidDnsTest.kt", NOT_CONVERGING)]).run(&["-F", "--relative"]);
    assert_line!(
        run.out,
        &format!(
            r"{TIME} \[pool-1-thread-1\] WARN io\.github\.ktlint\.core\.rule\.engine\.internal\.CodeFormatter -- Format was not able to resolve all violations which \(theoretically\) can be autocorrected in file .*AndroidDnsTest\.kt in 3 consecutive runs of format\."
        )
    );
}

#[test]
fn format_from_stdin_that_does_not_converge_logs_a_warning_from_the_main_thread() {
    let run = Project::new("converge", &[]).run_with_stdin(&["--stdin", "-F"], NOT_CONVERGING.as_bytes());
    assert_line!(
        run.out,
        &format!(r"{TIME} \[main\] WARN io\.github\.ktlint\.core\.rule\.engine\.internal\.CodeFormatter -- .* in file <stdin> in 3 consecutive runs of format\.")
    );
}

#[test]
fn disabling_the_suppression_rule_logs_a_warning() {
    let files = [("A.kt", "val a = 1\n"), (".editorconfig", "root = true\n\n[*]\nktlint_internal_ktlint-suppression = disabled\n")];
    let run = Project::new("suppression", &files).run(&["--relative"]);
    assert_line!(
        run.out,
        &format!(
            r"{TIME} \[pool-1-thread-1\] WARN io\.github\.ktlint\.core\.rule\.engine\.internal\.rulefilter\.RuleExecutionRuleFilter -- Rule 'internal:ktlint-suppression' can not be disabled via the '\.editorconfig'"
        )
    );
}
