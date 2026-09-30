//! Port of `KtlintSuppressionRuleTest.kt`: `Given a pair of matching ktlint directives in block comments within
//! the same parent node`, `Given a ktlint-enable directive` and the redundant-directive case.

mod support;

use support::assert_that::{trim_indent as t, v};
use support::suppression_rule::{
    DISABLE, ENABLE, ktlint_suppression_rule_assert_that as assert_that,
};

fn pair(rule_ids: &str) -> String {
    format!(
        "fun foo() {{\n    /* ktlint-disable {rule_ids} */\n    doSomething()\n    /* ktlint-enable {rule_ids} */\n}}"
    )
}

#[test]
fn pair_given_a_ktlint_disable_directive_without_rule_id() {
    assert_that(
        &pair("")
            .replace("disable  */", "disable */")
            .replace("enable  */", "enable */"),
    )
    .has_lint_violations(&[v(2, 8, DISABLE), v(4, 8, ENABLE)])
    .is_formatted_as(&t(r#"
            @Suppress("ktlint")
            fun foo() {
                doSomething()
            }
            "#));
}

#[test]
fn pair_given_a_ktlint_disable_directive_with_rule_id_not_prefixed_with_a_rule_set_id() {
    assert_that(&pair("foo"))
        .has_lint_violations(&[v(2, 8, DISABLE), v(4, 8, ENABLE)])
        .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:foo")
        fun foo() {
            doSomething()
        }
        "#));
}

#[test]
fn pair_given_a_ktlint_disable_directive_with_rule_id_prefixed_with_a_rule_set_id() {
    assert_that(&pair("standard:foo"))
        .has_lint_violations(&[v(2, 8, DISABLE), v(4, 8, ENABLE)])
        .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:foo")
        fun foo() {
            doSomething()
        }
        "#));
}

#[test]
fn pair_given_a_ktlint_disable_directive_with_multiple_rule_ids() {
    for rule_ids in [
        "standard:bar standard:foo",
        "standard:bar   standard:foo",
        "standard:bar standard:foo standard:bar",
        "standard:foo standard:bar",
    ] {
        assert_that(&pair(rule_ids))
            .has_lint_violations(&[v(2, 8, DISABLE), v(4, 8, ENABLE)])
            .is_formatted_as(&t(r#"
            @Suppress("ktlint:standard:bar", "ktlint:standard:foo")
            fun foo() {
                doSomething()
            }
            "#));
    }
}

#[test]
fn pair_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_suppress_then_add_the_ktlint_suppression_and_sort_all_suppressions_alphabetically()
 {
    assert_that(&format!(
        "@Suppress(\"zzz\", \"aaa\")\n{}",
        pair("standard:foo")
    ))
    .has_lint_violations(&[v(3, 8, DISABLE), v(5, 8, ENABLE)])
    .is_formatted_as(&t(r#"
            @Suppress("aaa", "ktlint:standard:foo", "zzz")
            fun foo() {
                doSomething()
            }
            "#));
}

#[test]
fn pair_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_suppress_warnings_then_add_the_ktlint_suppression_and_sort_all_suppressions_alphabetically()
 {
    assert_that(&format!(
        "@SuppressWarnings(\"aaa\", \"zzz\")\n{}",
        pair("standard:foo")
    ))
    .has_lint_violations(&[v(3, 8, DISABLE), v(5, 8, ENABLE)])
    .is_formatted_as(&t(r#"
            @SuppressWarnings("aaa", "ktlint:standard:foo", "zzz")
            fun foo() {
                doSomething()
            }
            "#));
}

#[test]
fn pair_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_both_suppress_and_suppress_warnings_then_add_the_ktlint_suppression_to_the_suppress()
 {
    assert_that(&format!(
        "@Suppress(\"aaa\", \"zzz\")\n@SuppressWarnings(\"bbb\", \"yyy\")\n{}",
        pair("standard:foo")
    ))
    .has_lint_violations(&[v(4, 8, DISABLE), v(6, 8, ENABLE)])
    .is_formatted_as(&t(r#"
            @Suppress("aaa", "ktlint:standard:foo", "zzz")
            @SuppressWarnings("bbb", "yyy")
            fun foo() {
                doSomething()
            }
            "#));
}

#[test]
fn given_a_ktlint_enable_directive_matching_with_a_ktlint_disable_directive() {
    assert_that(&pair("standard:foo"))
        .has_lint_violations(&[v(2, 8, DISABLE), v(4, 8, ENABLE)])
        .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:foo")
        fun foo() {
            doSomething()
        }
        "#));
}

#[test]
fn given_a_ktlint_enable_directive_not_matching_with_a_ktlint_disable_directive() {
    assert_that(
        r#"
        fun foo() {
            doSomething()
            /* ktlint-enable standard:foo */
        }
        "#,
    )
    .has_lint_violation(3, 8, ENABLE)
    .is_formatted_as(&t(r#"
        fun foo() {
            doSomething()
        }
        "#));
}

#[test]
fn given_a_ktlint_disable_directive_for_a_specific_rule_on_a_declaration_which_already_has_suppression_annotation_for_all_ktlint_rules()
 {
    assert_that(
        r#"
        @Suppress("ktlint")
        fun foo() {
            bar() // ktlint-disable standard:bar

            /* ktlint-disable standard:foo */
            /* ktlint-disable custom:foo */
            bar()
            /* ktlint-enable custom:foo */
            /* ktlint-enable standard:foo */
        }
        "#,
    )
    .has_lint_violations(&[
        v(3, 14, DISABLE),
        v(5, 8, DISABLE),
        v(6, 8, DISABLE),
        v(8, 8, ENABLE),
        v(9, 8, ENABLE),
    ])
    .is_formatted_as(&t(r#"
        @Suppress("ktlint")
        fun foo() {
            bar()

            bar()
        }
        "#));
}
