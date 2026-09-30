//! Port of ktlint-rule-engine `internal/SuppressionLocatorTest.kt` (annotations and directives; the
//! formatter-tag cases are in `engine_suppression_locator_tags.rs`).

mod engine_common;

use engine_common::{
    NO_FOO_MESSAGE, STANDARD_NO_FOO_IDENTIFIER_RULE_ID, lint_no_foo, no_foo_errors,
};
use ktrs_lint::engine::internal_rules::KTLINT_SUPPRESSION_RULE_ID;
use ktrs_lint::{EditorConfigOverride, LintError};

fn lint(code: &str) -> Vec<LintError> {
    lint_no_foo(code, EditorConfigOverride::empty(), Vec::new(), true)
}

fn suppression_error(
    line: usize,
    col: usize,
    detail: &str,
    can_be_auto_corrected: bool,
) -> LintError {
    LintError {
        line,
        col,
        rule_id: KTLINT_SUPPRESSION_RULE_ID,
        detail: detail.to_owned(),
        can_be_auto_corrected,
    }
}

#[test]
fn given_that_no_foo_identifier_rule_finds_a_violation() {
    let code = "val foo = \"foo\"\nval fooWithSuffix = \"fooWithSuffix\"";
    assert_eq!(
        lint(code),
        [no_foo_errors(1, 5), no_foo_errors(2, 5)].concat()
    );
}

#[test]
fn given_an_eol_comment_with_a_ktlint_directive_to_disable_all_rules_then_do_not_suppress_the_violation_anymore()
 {
    let code = "val foo = \"foo\" // ktlint-disable";
    let expected = LintError {
        line: 1,
        col: 5,
        rule_id: STANDARD_NO_FOO_IDENTIFIER_RULE_ID,
        detail: NO_FOO_MESSAGE.to_owned(),
        can_be_auto_corrected: false,
    };
    assert!(lint(code).contains(&expected));
}

#[test]
fn given_a_violation_suppressed_with_suppress_at_statement_level_for_all_rules() {
    let code = "@Suppress(\"ktlint\")\nval fooNotReported = \"foo\"\n\nval fooReported = \"foo\"";
    assert_eq!(lint(code), no_foo_errors(4, 5));
}

#[test]
fn given_a_violation_suppressed_with_suppress_at_statement_level_for_a_specific_rule() {
    let code = "@Suppress(\"ktlint:standard:no-foo-identifier-standard\", \"ktlint:custom:no-foo-identifier\")\n\
                val fooNotReported = \"foo\"\n\nval fooReported = \"foo\"";
    assert_eq!(lint(code), no_foo_errors(4, 5));
}

#[test]
fn given_a_violation_suppressed_with_suppress_at_function_level() {
    let code = "@Suppress(\"ktlint:standard:no-foo-identifier-standard\", \"ktlint:custom:no-foo-identifier\")\n\
                fun foo() {\n    val fooNotReported = \"foo\"\n}\n\nval fooReported = \"foo\"";
    assert_eq!(lint(code), no_foo_errors(6, 5));
}

#[test]
fn given_a_violation_suppressed_with_suppress_at_function_level_for_all_rules() {
    let code = "@Suppress(\"ktlint\")\nfun foo() {\n    val fooNotReported = \"foo\"\n}\n\nval fooReported = \"foo\"";
    assert_eq!(lint(code), no_foo_errors(6, 5));
}

#[test]
fn given_a_violation_suppressed_with_suppress_at_class_level() {
    let code = "@Suppress(\"ktlint:standard:no-foo-identifier-standard\", \"ktlint:custom:no-foo-identifier\")\n\
                class Foo {\n    fun foo() {\n        val fooNotReported = \"foo\"\n    }\n\n    val foo = \"foo\"\n}";
    assert_eq!(lint(code), []);
}

#[test]
fn given_a_violation_suppressed_with_suppress_for_all_rules_at_class_level() {
    let code = "@Suppress(\"ktlint\")\nclass Foo {\n    fun foo() {\n        val fooNotReported = \"foo\"\n    }\n\n    val foo = \"foo\"\n}";
    assert_eq!(lint(code), []);
}

#[test]
fn given_that_the_no_foo_identifier_rule_is_suppressed_in_the_entire_file_with_file_suppress() {
    let code = "@file:Suppress(\"ktlint:standard:no-foo-identifier-standard\", \"ktlint:custom:no-foo-identifier\")\n\n\
                class Foo {\n    fun foo() {\n        val fooNotReported = \"foo\"\n    }\n}\n\nval fooNotReported = \"foo\"";
    assert_eq!(lint(code), []);
}

#[test]
fn given_that_all_rules_are_suppressed_in_the_entire_file_with_file_suppress() {
    let code = "@file:Suppress(\"ktlint\")\n\nclass Foo {\n    fun foo() {\n        val fooNotReported = \"foo\"\n    }\n}\n\n\
                val fooNotReported = \"foo\"";
    assert_eq!(lint(code), []);
}

#[test]
fn given_code_that_tries_to_disable_the_ktlint_suppression_rule_itself_given_a_file_annotation() {
    let code = "@file:Suppress(\"ktlint:internal:ktlint-suppression\")";
    let actual = lint_no_foo(code, EditorConfigOverride::empty(), Vec::new(), false);
    assert_eq!(
        actual,
        [suppression_error(
            1,
            17,
            "Ktlint rule with id 'ktlint:internal:ktlint-suppression' is unknown or not loaded",
            false
        )]
    );
}

#[test]
fn given_code_that_tries_to_disable_the_ktlint_suppression_rule_itself_given_a_block_comment_with_a_ktlint_disable_directive()
 {
    let code = "/* ktlint-disable internal:ktlint-suppression */";
    let actual = lint_no_foo(code, EditorConfigOverride::empty(), Vec::new(), false);
    assert_eq!(
        actual,
        [
            suppression_error(
                1,
                4,
                "Directive 'ktlint-disable' is deprecated. Replace with @Suppress annotation",
                true
            ),
            suppression_error(
                1,
                19,
                "Ktlint rule with id 'internal:ktlint-suppression' is unknown or not loaded",
                false
            ),
        ]
    );
}

#[test]
fn given_code_that_tries_to_disable_the_ktlint_suppression_rule_itself_given_an_eol_comment_with_a_ktlint_disable_directive()
 {
    let code = "val foo = \"foo\" // ktlint-disable internal:ktlint-suppression";
    let actual = lint_no_foo(code, EditorConfigOverride::empty(), Vec::new(), false);
    let mut expected = no_foo_errors(1, 5).to_vec();
    expected.push(suppression_error(
        1,
        20,
        "Directive 'ktlint-disable' is deprecated. Replace with @Suppress annotation",
        true,
    ));
    expected.push(suppression_error(
        1,
        35,
        "Ktlint rule with id 'internal:ktlint-suppression' is unknown or not loaded",
        false,
    ));
    assert_eq!(actual, expected);
}

// skipped: needs IndentationRule — `Given a suppression of a rule which alphabetically comes before rule id ktlint-suppression`
// skipped: needs IndentationRule — `Issue 2695 - ` (format of a ktlint-disable standard:indent block)
// skipped: needs NoUnusedImportsRule — `Issue 2696 - Given an import which is only used in a block that is suppressed ...`
