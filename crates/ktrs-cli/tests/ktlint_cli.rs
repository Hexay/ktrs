//! Port of ktlint-cli's `SimpleCLITest`, in process. Upstream's `too-many-empty-lines` project violates
//! `no-consecutive-blank-lines`/`no-empty-first-line-in-method-block`, not ported yet: its file here violates
//! `no-semi` instead.

mod ktlint_support;

use ktlint_support::{Project, contains_line};

const NO_CODE_STYLE_ERROR: &[(&str, &str)] = &[("Main.kt", "fun main() {\n    println(\"Hello world!\")\n}\n")];
const WITH_ERROR: &[(&str, &str)] = &[("Main.kt.test", "fun main() {\n    println(\"Hello world!\");\n}\n")];
const WITH_ERROR_FORMATTED: &str = "fun main() {\n    println(\"Hello world!\")\n}\n";

fn with_error() -> Project {
    Project::new("too-many-empty-lines", WITH_ERROR)
}

#[test]
fn given_help_then_return_the_help_output() {
    let run = Project::new("no-code-style-error", NO_CODE_STYLE_ERROR).run(&["--help"]);
    run.assert_normal_exit_code();
    assert!(contains_line(&run.out, "An anti-bikeshedding Kotlin linter with built-in formatter."));
    assert!(contains_line(&run.out, "Usage:"));
    assert!(contains_line(&run.out, "EXAMPLES"));
}

#[test]
fn given_version_then_return_the_version_information_output() {
    for version in ["-v", "--version"] {
        let run = Project::new("no-code-style-error", NO_CODE_STYLE_ERROR).run(&[version]);
        run.assert_normal_exit_code();
        assert!(run.out.contains("ktlint version 2.0.0-ALPHA-4"), "{run:?}");
    }
}

#[test]
fn given_code_without_errors_then_normal_exit_code_and_no_error_output() {
    let run = Project::new("no-code-style-error", NO_CODE_STYLE_ERROR).run(&["--log-level=debug"]);
    run.assert_normal_exit_code().assert_error_output_is_empty();
}

#[test]
fn given_code_with_an_error_then_error_exit_code_and_error_output() {
    let run = with_error().run(&["**/*.test"]);
    run.assert_error_exit_code();
    assert_line!(run.out, ".*Main.kt.test:2:28: Unnecessary semicolon \\(standard:no-semi\\)");
}

#[test]
fn given_code_with_an_error_then_warning_to_use_format() {
    let run = with_error().run(&["**/*.test"]);
    run.assert_error_exit_code();
    assert_line!(run.out, ".* WARN .* Lint has found errors than can be autocorrected using 'ktlint --format'");
}

#[test]
fn given_code_with_an_error_but_a_glob_which_does_not_select_the_file() {
    let run = with_error().run(&["some-pattern"]);
    run.assert_normal_exit_code();
    assert!(contains_line(&run.out, "No files matched [some-pattern]"), "{run:?}");
}

#[test]
fn given_a_default_kotlin_extension_then_it_is_picked_up_without_patterns() {
    let run = Project::new("no-code-style-error", NO_CODE_STYLE_ERROR).run(&["--log-level=debug"]);
    run.assert_normal_exit_code();
    assert!(contains_line(&run.out, "Enable default patterns"), "{run:?}");
    assert!(contains_line(&run.out, "1 file(s) scanned / 0 error(s)"), "{run:?}");
}

#[test]
fn given_code_with_an_error_which_can_be_autocorrected_then_normal_exit_code() {
    let project = with_error();
    let run = project.run(&["-F", "**/*.test"]);
    run.assert_normal_exit_code();
    assert_eq!(project.read("Main.kt.test"), WITH_ERROR_FORMATTED);
}

#[test]
fn given_patterns_from_stdin_which_do_not_select_the_file_then_no_files_matched_warning() {
    for option in ["--patterns-from-stdin", "--patterns-from-stdin="] {
        let run = with_error().run_with_stdin(&[option], b"path/to/file/1.kt\0path/to/file/2.kts\0");
        run.assert_normal_exit_code();
        assert!(contains_line(&run.out, "No files matched [path/to/file/1.kt, path/to/file/2.kts]"), "{run:?}");
    }
}

#[test]
fn issue_1793_given_no_patterns_read_from_stdin_then_return_nothing() {
    let run = with_error().run_with_stdin(&["--patterns-from-stdin"], b"");
    run.assert_normal_exit_code();
    assert!(!contains_line(&run.out, "Enable default patterns"), "{run:?}");
    assert!(contains_line(&run.out, "No files matched []"), "{run:?}");
}

#[test]
fn issue_1608_relative_and_sarif_play_well_together() {
    let run = with_error().run(&["**/*.test", "--relative", "--reporter=sarif"]);
    run.assert_error_exit_code();
    assert!(!contains_line(&run.err, "this and base files have different roots"), "{run:?}");
}

#[test]
fn issue_2576_single_reporter() {
    let run = with_error().run(&["**/*.test", "--reporter=plain"]);
    run.assert_error_exit_code();
    assert!(contains_line(&run.out, "Main.kt.test:2:28: Unnecessary semicolon"), "{run:?}");
}

#[test]
fn issue_2576_multiple_reporters() {
    let args = ["**/*.test", "--reporter=plain", "--reporter=json,output=ktlint-violations.json", "--log-level=debug"];
    let run = with_error().run(&args);
    run.assert_error_exit_code();
    assert!(contains_line(&run.out, "Initializing \"plain\" reporter"), "{run:?}");
    assert_line!(run.out, ".*Initializing \"json\" reporter with .*, output=ktlint-violations.json");
    assert!(contains_line(&run.out, "Main.kt.test:2:28: Unnecessary semicolon"), "{run:?}");
    assert_line!(run.out, ".*ReporterAggregator -- \"json\" report written to .*ktlint-violations.json");
}

#[test]
fn given_a_custom_reporter_which_does_not_exist() {
    let run = with_error().run(&["**/*.test", "--reporter=custom,artifact=custom-reporter.jar"]);
    run.assert_error_exit_code();
    assert!(contains_line(&run.out, "File 'custom-reporter.jar' does not exist"), "{run:?}");
}

#[test]
fn generate_git_hooks_outside_a_git_repository() {
    for subcommand in ["installGitPreCommitHook", "installGitPrePushHook"] {
        let run = with_error().run(&[subcommand]);
        run.assert_error_exit_code();
        assert!(contains_line(&run.err, "git directory not found. Are you sure you are inside project directory?"), "{run:?}");
    }
}

#[test]
fn generate_editorconfig_given_the_code_style() {
    // Upstream passes "--code-style ktlint_official" through a shell, which splits it.
    let run = with_error().run(&["generateEditorConfig", "--code-style", "ktlint_official"]);
    run.assert_normal_exit_code();
    assert!(contains_line(&run.out, "ktlint_code_style = ktlint_official"), "{run:?}");
}

#[test]
fn generate_editorconfig_without_the_code_style_fails() {
    let run = with_error().run(&["generateEditorConfig"]);
    run.assert_error_exit_code();
    assert!(contains_line(&run.err, "Error: missing option --code-style"), "{run:?}");
}

#[test]
fn issue_1742_disable_the_filename_rule_when_stdin_is_used() {
    // Upstream asserts the engine's trace dump `ktlint_standard_filename: disabled`; the CLI's own debug line instead.
    let run = with_error().run_with_stdin(&["--stdin", "--log-level=debug"], b"fun foo() = 42");
    assert!(contains_line(&run.out, "Add editor config override to disable 'filename' rule"), "{run:?}");
}

#[test]
fn issue_1123_enable_filename_rule_when_stdin_and_stdin_path_are_used() {
    let run = with_error().run_with_stdin(&["--stdin", "--stdin-path", "/foo/Foo.kt", "--log-level=debug"], b"fun foo() = 42");
    assert!(!contains_line(&run.out, "Add editor config override to disable 'filename' rule"), "{run:?}");
}

#[test]
fn issue_3296_merge_editorconfig_from_subdirectory_with_root_when_relative_stdin_path_is_used() {
    let project = Project::new(
        "nested-editorconfig",
        &[
            (".editorconfig", "root = true\n\n[*.{kt,kts}]\nindent_size = 3\nend_of_line = crlf\nktlint_code_style = ktlint_official\n"),
            ("foo/.editorconfig", "[*.{kt,kts}]\nindent_size = 6\nktlint_code_style = intellij_idea\n"),
        ],
    );
    // Upstream asserts the engine's trace dump of the merged properties; the indent they cause instead.
    let run = project.run_with_stdin(&["--stdin", "--stdin-path", "foo/Foo.kt"], b"fun foo() {\n  val x = 42\n}\n");
    assert!(contains_line(&run.err, "Unexpected indentation (2) (should be 6)"), "{run:?}");
}

const SCRIPT: &[u8] = b"pluginManagement {\n    repositories {\n        mavenCentral();\n    }\n}\n";

#[test]
fn issue_1832_stdin_not_parsable_as_kotlin_is_linted_as_kotlin_script() {
    // Upstream's script has "Needless blank line"s (no-consecutive-blank-lines, not ported): a semicolon here.
    let run = with_error().run_with_stdin(&["--stdin"], SCRIPT);
    assert!(contains_line(&run.out, "Can not parse input from <stdin> as Kotlin, due to error below:"), "{run:?}");
    assert!(contains_line(&run.err, "Unnecessary semicolon"), "{run:?}");
}

#[test]
fn issue_1832_stdin_not_parsable_as_kotlin_is_formatted_as_kotlin_script() {
    let run = with_error().run_with_stdin(&["--stdin", "--format"], SCRIPT);
    assert!(contains_line(&run.out, "Can not parse input from <stdin> as Kotlin, due to error below:"), "{run:?}");
    assert!(contains_line(&run.out, "Now, trying to read the input as Kotlin Script."), "{run:?}");
    assert!(!contains_line(&run.out, "Can not parse input from <stdin> as Kotlin script, due to error below:"), "{run:?}");
}

#[test]
fn issue_2379_stdin_not_parsable_as_kotlin_nor_as_kotlin_script() {
    let run = with_error().run_with_stdin(&["--stdin", "--format"], b"fun foo() =");
    assert_eq!(run.exit_code, 3, "{run:?}");
    assert!(contains_line(&run.out, "Can not parse input from <stdin> as Kotlin, due to error below:"), "{run:?}");
    assert!(contains_line(&run.out, "Can not parse input from <stdin> as Kotlin script, due to error below:"), "{run:?}");
    assert!(contains_line(&run.out, "Not a valid Kotlin file"), "{run:?}");
}

#[test]
fn issue_3096_unicode_from_stdin_is_written_to_stdout_in_utf8() {
    let code = "// Preserve \u{a7} symbol when formatting the file with ktlint\nval sectionSign = { Char(167) }";
    let run = with_error().run_with_stdin(&["--stdin", "--format"], code.as_bytes());
    assert!(run.out.contains(code), "{run:?}");
}

#[test]
fn issue_3278_percent_from_stdin_is_not_interpreted_as_format_instruction() {
    // Upstream's `class Foo { val foo = "%"; }` is also wrapped by statement-wrapping (not ported): a property only.
    let run = with_error().run_with_stdin(&["--stdin", "--format"], b"val foo = \"%\";\n");
    assert!(run.out.contains("val foo = \"%\"\n"), "{run:?}");
}

const IGNORE_AUTOCORRECT_FAILURES: &[(&str, &str)] = &[(
    "Foo.kt",
    "// This file is name incorrectly as it is not named after the only class in it. Incorrectly named files cannot be autocorrected\nclass Bar",
)];

#[test]
fn issue_3150_report_violations_that_cannot_be_autocorrected() {
    let run = Project::new("ignore-autocorrect-failures", IGNORE_AUTOCORRECT_FAILURES).run(&["--format"]);
    run.assert_error_exit_code();
    assert_line!(run.out, ".*\\(cannot be auto-corrected\\) \\(standard:filename\\).*");
}

#[test]
fn issue_3150_ignore_violations_that_cannot_be_autocorrected() {
    let run = Project::new("ignore-autocorrect-failures", IGNORE_AUTOCORRECT_FAILURES).run(&["--format", "--ignore-autocorrect-failures"]);
    run.assert_normal_exit_code();
    assert_no_line!(run.out, ".*\\(cannot be auto-corrected\\) \\(standard:filename\\).*");
}

#[test]
fn do_not_print_kotlin_logging_startup_message_to_stdout() {
    let run = with_error().run(&["**/*.test", "--reporter=plain"]);
    assert!(!contains_line(&run.out, "kotlin-logging: initializing..."));
    let run = with_error().run_with_stdin(&["--stdin"], b"   fun foo() = 42");
    assert!(!contains_line(&run.out, "kotlin-logging: initializing..."));
}
