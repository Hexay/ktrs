//! Port of `KtlintRuleEngineSuppressionKtTest.kt` (`KtLintRuleEngine.insertSuppression`).

mod support;

use ktrs_lint::RuleId;
use ktrs_lint::engine::KtlintSuppression;
use support::assert_that::trim_indent as t;
use support::insert::{SOME_RULE_ID, insert};

const EXPRESSION_OPERAND_WRAPPING_RULE_ID: RuleId = RuleId("standard:expression-operand-wrapping");
const NO_CONSECUTIVE_BLANK_LINES_RULE_ID: RuleId = RuleId("standard:no-consecutive-blank-lines");
const NO_LINE_BREAK_BEFORE_ASSIGNMENT_RULE_ID: RuleId =
    RuleId("standard:no-line-break-before-assignment");

fn at(code: &str, line: usize, col: usize, rule_id: RuleId) -> String {
    insert(&t(code), KtlintSuppression::AtOffset { line, col, rule_id })
}

#[test]
fn given_a_file_suppression_then_add_the_suppression_at_file_level() {
    let actual = insert(
        "import foo.Foo",
        KtlintSuppression::ForFile {
            rule_id: SOME_RULE_ID,
        },
    );
    assert_eq!(
        actual,
        "@file:Suppress(\"ktlint:standard:some-rule-id\")\n\nimport foo.Foo"
    );
}

#[test]
fn given_an_offset_suppression_at_an_import_statement_then_add_the_suppression_at_file_level() {
    assert_eq!(
        at("import foo.Foo", 1, 1, SOME_RULE_ID),
        "@file:Suppress(\"ktlint:standard:some-rule-id\")\n\nimport foo.Foo"
    );
}

#[test]
fn given_an_offset_suppression_in_a_top_level_declaration_before_the_assignment_then_add_the_suppression_on_the_declaration()
 {
    assert_eq!(
        at(r#"val foo = "Foo""#, 1, 1, SOME_RULE_ID),
        "@Suppress(\"ktlint:standard:some-rule-id\")\nval foo = \"Foo\""
    );
}

#[test]
fn given_an_offset_suppression_in_a_string_template_then_add_the_suppression_on_top_of_the_string_template()
 {
    for index in [11, 12, 13, 14, 15] {
        assert_eq!(
            at(r#"val foo = "Foo""#, 1, index, SOME_RULE_ID),
            "val foo = @Suppress(\"ktlint:standard:some-rule-id\")\n\"Foo\"",
            "index {index}"
        );
    }
}

#[test]
fn given_an_offset_suppression_inside_a_string_template_value_argument_then_add_the_suppression_to_the_value_argument()
 {
    let code = r#"
        fun foo() {
            bar("Foo")
        }
        "#;
    assert_eq!(
        at(code, 2, 10, SOME_RULE_ID),
        t(r#"
        fun foo() {
            bar(@Suppress("ktlint:standard:some-rule-id")
            "Foo")
        }
        "#)
    );
}

#[test]
fn given_an_offset_suppression_on_a_value_argument_then_add_the_suppression_to_the_parent_of_the_value_argument_list()
 {
    let code = r#"
        fun foo(): String {
            bar(
                "Foo",
            )
        }
        "#;
    assert_eq!(
        at(code, 3, 14, SOME_RULE_ID),
        t(r#"
        fun foo(): String {
            @Suppress("ktlint:standard:some-rule-id")
            bar(
                "Foo",
            )
        }
        "#)
    );
}

#[test]
fn given_an_offset_suppression_in_a_string_template_which_is_part_of_a_return_statement_then_add_the_suppression_on_top_of_the_return_statement()
 {
    let code = r#"
        fun foo(): String {
            return "Foo"
        }
        "#;
    assert_eq!(
        at(code, 2, 13, SOME_RULE_ID),
        t(r#"
        fun foo(): String {
            @Suppress("ktlint:standard:some-rule-id")
            return "Foo"
        }
        "#)
    );
}

#[test]
fn given_a_value_argument_with_a_multiline_condition_to_which_a_suppression_is_to_be_added() {
    let code = r#"
        fun foo() {
            require(
                true && false ||
                    true,
            )
        }
        "#;
    assert_eq!(
        at(code, 3, 17, EXPRESSION_OPERAND_WRAPPING_RULE_ID),
        t(r#"
        fun foo() {
            @Suppress("ktlint:standard:expression-operand-wrapping")
            require(
                true && false ||
                    true,
            )
        }
        "#)
    );
}

#[test]
fn given_a_property_assignment_with_a_multiline_condition_to_which_a_suppression_is_to_be_added() {
    let code = r#"
        val bar =
            true && false ||
                true
        "#;
    assert_eq!(
        at(code, 2, 17, EXPRESSION_OPERAND_WRAPPING_RULE_ID),
        t(r#"
        @Suppress("ktlint:standard:expression-operand-wrapping")
        val bar =
            true && false ||
                true
        "#)
    );
}

#[test]
fn given_an_if_statement_with_a_multiline_condition_to_which_a_suppression_is_to_be_added() {
    let code = r#"
        fun foo() {
            if (true && false ||
                true
            ) {}
        }
        "#;
    assert_eq!(
        at(code, 2, 17, EXPRESSION_OPERAND_WRAPPING_RULE_ID),
        t(r#"
        fun foo() {
            @Suppress("ktlint:standard:expression-operand-wrapping")
            if (true && false ||
                true
            ) {}
        }
        "#)
    );
}

#[test]
fn given_a_violation_with_offset_exactly_at_the_eol_of_a_line() {
    let code = r#"
        fun foo(): String // Some comment
            = "some-result"
        "#;
    assert_eq!(
        at(code, 1, 34, NO_LINE_BREAK_BEFORE_ASSIGNMENT_RULE_ID),
        t(r#"
        @Suppress("ktlint:standard:no-line-break-before-assignment")
        fun foo(): String // Some comment
            = "some-result"
        "#)
    );
}

#[test]
fn given_some_consecutive_blank_lines_at_top_level() {
    let code = "val foo = \"foo\"\n\n\nval bar = \"bar\"";
    assert_eq!(
        insert(
            code,
            KtlintSuppression::AtOffset {
                line: 3,
                col: 1,
                rule_id: NO_CONSECUTIVE_BLANK_LINES_RULE_ID
            }
        ),
        format!("@file:Suppress(\"ktlint:standard:no-consecutive-blank-lines\")\n\n{code}")
    );
}

#[test]
fn given_some_consecutive_blank_lines_inside_a_function() {
    let code = "fun foobar() {\n    val foo = \"foo\"\n\n\n    val bar = \"bar\"\n}";
    assert_eq!(
        insert(
            code,
            KtlintSuppression::AtOffset {
                line: 3,
                col: 1,
                rule_id: NO_CONSECUTIVE_BLANK_LINES_RULE_ID
            }
        ),
        format!("@Suppress(\"ktlint:standard:no-consecutive-blank-lines\")\n{code}")
    );
}
