//! Port of `KtlintSuppressionRuleTest.kt`: `Given an EOL comment with a ktlint-disable directive` and
//! `Given a top level block comment with a ktlint-disable directive`.

mod support;

use support::assert_that::trim_indent as t;
use support::suppression_rule::{DISABLE, ktlint_suppression_rule_assert_that as assert_that};

#[test]
fn eol_given_a_ktlint_disable_directive_without_rule_id() {
    assert_that(r#"val foo = "foo" // ktlint-disable"#)
        .has_lint_violation(1, 20, DISABLE)
        .is_formatted_as(&t(r#"
        @Suppress("ktlint")
        val foo = "foo"
        "#));
}

#[test]
fn eol_given_a_ktlint_disable_directive_with_rule_id_not_prefixed_with_a_rule_set_id() {
    assert_that(r#"val foo = "foo" // ktlint-disable foo"#)
        .has_lint_violation(1, 20, DISABLE)
        .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:foo")
        val foo = "foo"
        "#));
}

#[test]
fn eol_given_a_ktlint_disable_directive_with_rule_id_prefixed_with_a_rule_set_id() {
    assert_that(r#"val foo = "foo" // ktlint-disable standard:foo"#)
        .has_lint_violation(1, 20, DISABLE)
        .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:foo")
        val foo = "foo"
        "#));
}

#[test]
fn eol_given_a_ktlint_disable_directive_with_multiple_rule_ids() {
    for rule_ids in [
        "custom:foo standard:bar standard:foo",
        "custom:foo   standard:bar   standard:foo",
        "standard:bar standard:foo standard:bar custom:foo",
        "standard:foo custom:foo standard:bar",
    ] {
        assert_that(&format!(r#"val foo = "foo" // ktlint-disable {rule_ids}"#))
            .has_lint_violation(1, 20, DISABLE)
            .is_formatted_as(&t(r#"
                @Suppress("ktlint:custom:foo", "ktlint:standard:bar", "ktlint:standard:foo")
                val foo = "foo"
                "#));
    }
}

#[test]
fn eol_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_suppress_then_add_the_ktlint_suppression_and_sort_all_suppressions_alphabetically()
 {
    assert_that(
        r#"
        @Suppress("zzz", "aaa")
        val foo = "foo" // ktlint-disable standard:foo
        "#,
    )
    .has_lint_violation(2, 20, DISABLE)
    .is_formatted_as(&t(r#"
        @Suppress("aaa", "ktlint:standard:foo", "zzz")
        val foo = "foo"
        "#));
}

#[test]
fn eol_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_suppress_warnings_then_add_the_ktlint_suppression_and_sort_all_suppressions_alphabetically()
 {
    assert_that(
        r#"
        @SuppressWarnings("aaa", "zzz")
        val foo = "foo" // ktlint-disable standard:foo
        "#,
    )
    .has_lint_violation(2, 20, DISABLE)
    .is_formatted_as(&t(r#"
        @SuppressWarnings("aaa", "ktlint:standard:foo", "zzz")
        val foo = "foo"
        "#));
}

#[test]
fn eol_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_both_suppress_and_suppress_warnings_then_add_the_ktlint_suppression_to_the_suppress()
 {
    assert_that(
        r#"
        @Suppress("aaa", "zzz")
        @SuppressWarnings("bbb", "yyy")
        val foo = "foo" // ktlint-disable standard:foo
        "#,
    )
    .has_lint_violation(3, 20, DISABLE)
    .is_formatted_as(&t(r#"
        @Suppress("aaa", "ktlint:standard:foo", "zzz")
        @SuppressWarnings("bbb", "yyy")
        val foo = "foo"
        "#));
}

#[test]
fn top_level_given_a_ktlint_disable_directive_without_rule_id() {
    assert_that("/* ktlint-disable */")
        .has_lint_violation(1, 4, DISABLE)
        .is_formatted_as(r#"@file:Suppress("ktlint")"#);
}

#[test]
fn top_level_given_a_top_level_ktlint_disable_directive() {
    for (rule_ids, expected_suppression_id_string) in [
        ("foo", "ktlint:standard:foo"),
        ("standard:foo", "ktlint:standard:foo"),
        ("custom:foo", "ktlint:custom:foo"),
        (
            "custom:foo standard:bar",
            "ktlint:custom:foo,ktlint:standard:bar",
        ),
        (
            "custom:foo   standard:bar",
            "ktlint:custom:foo,ktlint:standard:bar",
        ),
        (
            "custom:foo standard:bar custom:foo",
            "ktlint:custom:foo,ktlint:standard:bar",
        ),
        (
            "standard:bar custom:foo",
            "ktlint:custom:foo,ktlint:standard:bar",
        ),
    ] {
        let ids: Vec<String> = expected_suppression_id_string
            .split(',')
            .map(|it| format!("\"{it}\""))
            .collect();
        assert_that(&format!("/* ktlint-disable {rule_ids} */"))
            .has_lint_violation(1, 4, DISABLE)
            .is_formatted_as(&format!("@file:Suppress({})", ids.join(", ")));
    }
}

#[test]
fn top_level_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_suppress_then_add_the_ktlint_suppression_and_sort_all_suppressions_alphabetically()
 {
    assert_that(
        r#"
        @file:Suppress("zzz", "aaa")
        /* ktlint-disable standard:foo */
        "#,
    )
    .has_lint_violation(2, 4, DISABLE)
    .is_formatted_as(r#"@file:Suppress("aaa", "ktlint:standard:foo", "zzz")"#);
}

#[test]
fn top_level_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_suppress_warnings_then_add_the_ktlint_suppression_and_sort_all_suppressions_alphabetically()
 {
    assert_that(
        r#"
        @file:SuppressWarnings("aaa", "zzz")
        /* ktlint-disable standard:foo */
        "#,
    )
    .has_lint_violation(2, 4, DISABLE)
    .is_formatted_as(r#"@file:SuppressWarnings("aaa", "ktlint:standard:foo", "zzz")"#);
}

#[test]
fn top_level_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_both_suppress_and_suppress_warnings_then_add_the_ktlint_suppression_to_the_suppress()
 {
    assert_that(
        r#"
        @file:Suppress("aaa", "zzz")
        @file:SuppressWarnings("bbb", "yyy")

        /* ktlint-disable standard:foo */
        "#,
    )
    .has_lint_violation(4, 4, DISABLE)
    .is_formatted_as(&t(r#"
        @file:Suppress("aaa", "ktlint:standard:foo", "zzz")
        @file:SuppressWarnings("bbb", "yyy")
        "#));
}

#[test]
fn top_level_given_a_ktlint_disable_directive_for_which_the_target_element_is_already_annotated_with_both_suppress_warnings_and_suppress_then_add_the_ktlint_suppression_to_the_suppress()
 {
    assert_that(
        r#"
        @file:SuppressWarnings("bbb", "yyy")
        @file:Suppress("aaa", "zzz")

        /* ktlint-disable standard:foo */
        "#,
    )
    .has_lint_violation(4, 4, DISABLE)
    .is_formatted_as(&t(r#"
        @file:SuppressWarnings("bbb", "yyy")
        @file:Suppress("aaa", "ktlint:standard:foo", "zzz")
        "#));
}
