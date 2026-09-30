//! Port of `KtlintSuppressionRuleTest.kt`: `Given a suppression annotation missing the rule set id prefix`,
//! the dangling EOL directive and `Given an import statement`.

mod support;

use support::assert_that::{trim_indent as t, v};
use support::suppression_rule::{
    DANGLING, DISABLE, QUALIFY, ktlint_suppression_rule_assert_that as assert_that,
};

#[test]
fn given_a_file_suppress_annotation() {
    assert_that(r#"@file:Suppress("ktlint:bar", "ktlint:standard:foo", "ktlint:custom:foo")"#)
        .has_lint_violation(1, 17, QUALIFY)
        .is_formatted_as(
            r#"@file:Suppress("ktlint:standard:bar", "ktlint:standard:foo", "ktlint:custom:foo")"#,
        );
}

#[test]
fn given_a_file_suppress_warnings_annotation() {
    assert_that(r#"@file:SuppressWarnings("ktlint:bar", "ktlint:standard:foo", "ktlint:custom:foo")"#)
        .has_lint_violation(1, 25, QUALIFY)
        .is_formatted_as(r#"@file:SuppressWarnings("ktlint:standard:bar", "ktlint:standard:foo", "ktlint:custom:foo")"#);
}

#[test]
fn given_a_file_array_annotation_with_suppress_and_suppress_warnings_annotations() {
    assert_that(r#"@file:[Suppress("ktlint:bar", "ktlint:custom:foo") SuppressWarnings("ktlint:foo")]"#)
        .has_lint_violations(&[v(1, 18, QUALIFY), v(1, 70, QUALIFY)])
        .is_formatted_as(r#"@file:[Suppress("ktlint:standard:bar", "ktlint:custom:foo") SuppressWarnings("ktlint:standard:foo")]"#);
}

#[test]
fn given_an_array_annotation_with_suppress_and_suppress_warnings_annotations() {
    assert_that(
        r#"
        @[Suppress("ktlint:bar", "ktlint:custom:foo") SuppressWarnings("ktlint:foo")]
        val foo = "foo"
        "#,
    )
    .has_lint_violations(&[v(1, 13, QUALIFY), v(1, 65, QUALIFY)])
    .is_formatted_as(&t(r#"
        @[Suppress("ktlint:standard:bar", "ktlint:custom:foo") SuppressWarnings("ktlint:standard:foo")]
        val foo = "foo"
        "#));
}

#[test]
fn given_a_suppress_annotation_with_a_named_argument_with_an_array_of_initialization() {
    assert_that(
        r#"
        @Suppress(names = arrayOf("ktlint:bar", "ktlint:standard:foo", "ktlint:custom:foo"))
        val foo = "foo"
        "#,
    )
    .has_lint_violation(1, 28, QUALIFY)
    .is_formatted_as(&t(r#"
        @Suppress(names = arrayOf("ktlint:standard:bar", "ktlint:standard:foo", "ktlint:custom:foo"))
        val foo = "foo"
        "#));
}

#[test]
fn given_a_suppress_annotation_with_a_named_argument_with_an_array_squared_brackets_initialization()
{
    assert_that(
        r#"
        @Suppress(names = ["ktlint:bar", "ktlint:standard:foo", "ktlint:custom:foo"])
        val foo = "foo"
        "#,
    )
    .has_lint_violation(1, 21, QUALIFY)
    .is_formatted_as(&t(r#"
        @Suppress(names = ["ktlint:standard:bar", "ktlint:standard:foo", "ktlint:custom:foo"])
        val foo = "foo"
        "#));
}

#[test]
fn given_a_suppress_annotation_on_a_declaration() {
    assert_that(
        r#"
        @Suppress("ktlint:bar", "ktlint:standard:foo", "ktlint:custom:foo")
        val foo = "foo"
        "#,
    )
    .has_lint_violation(1, 12, QUALIFY)
    .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:bar", "ktlint:standard:foo", "ktlint:custom:foo")
        val foo = "foo"
        "#));
}

#[test]
fn given_a_suppress_warnings_annotation_on_a_declaration() {
    assert_that(
        r#"
        @SuppressWarnings("ktlint:bar", "ktlint:standard:foo", "ktlint:custom:foo")
        val foo = "foo"
        "#,
    )
    .has_lint_violation(1, 20, QUALIFY)
    .is_formatted_as(&t(r#"
        @SuppressWarnings("ktlint:standard:bar", "ktlint:standard:foo", "ktlint:custom:foo")
        val foo = "foo"
        "#));
}

#[test]
fn given_an_eol_comment_with_a_ktlint_disable_directive_not_preceded_by_code_leaf_on_same_line() {
    assert_that(
        r#"
        val foo = "foo"
        // ktlint-disable
        val bar = "bar"
        "#,
    )
    .has_lint_violation(2, 4, DANGLING)
    .is_formatted_as(&t(r#"
        val foo = "foo"
        val bar = "bar"
        "#));
}

#[test]
fn given_an_eol_comment_with_a_ktlint_disable_directive_on_an_import() {
    assert_that(
        r#"
        import foo.bar
        import foobar.* // ktlint-disable no-wildcard-imports
        "#,
    )
    .has_lint_violation(2, 20, DISABLE)
    .is_formatted_as(&t(r#"
        @file:Suppress("ktlint:standard:no-wildcard-imports")

        import foo.bar
        import foobar.*
        "#));
}

#[test]
fn given_an_eol_comment_with_a_ktlint_disable_directive_on_an_import_and_an_existing_file_suppress_annotation()
 {
    assert_that(
        r#"
        @file:Suppress("aaa", "zzz")

        import foo.bar
        import foobar.* // ktlint-disable no-wildcard-imports
        "#,
    )
    .has_lint_violation(4, 20, DISABLE)
    .is_formatted_as(&t(r#"
        @file:Suppress("aaa", "ktlint:standard:no-wildcard-imports", "zzz")

        import foo.bar
        import foobar.*
        "#));
}

#[test]
fn given_an_eol_comment_with_a_ktlint_disable_directive_on_an_import_and_an_existing_file_suppress_annotation_without_parameters()
 {
    assert_that(
        r#"
        @file:Suppress

        import foobar.* // ktlint-disable no-wildcard-imports
        "#,
    )
    .has_lint_violation(3, 20, DISABLE)
    .is_formatted_as(&t(r#"
        @file:Suppress("ktlint:standard:no-wildcard-imports")

        import foobar.*
        "#));
}

#[test]
fn given_an_eol_comment_with_a_ktlint_disable_directive_on_an_import_on_a_file_starting_with_a_copyright_comment_before_the_package_statement()
 {
    assert_that(
        r#"
        /* Some copyright notice before package statement */
        package foo

        import foo.bar
        import foobar.* // ktlint-disable no-wildcard-imports
        "#,
    )
    .has_lint_violation(5, 20, DISABLE)
    .is_formatted_as(&t(r#"
        /* Some copyright notice before package statement */
        @file:Suppress("ktlint:standard:no-wildcard-imports")

        package foo

        import foo.bar
        import foobar.*
        "#));
}

#[test]
fn given_an_eol_comment_with_a_ktlint_disable_directive_on_an_import_and_an_existing_file_suppress_annotation_on_a_file_starting_with_a_copyright_comment()
 {
    assert_that(
        r#"
        /* Some copyright notice before package statement */
        @file:Suppress("aaa", "zzz")
        package foo

        import foo.bar
        import foobar.* // ktlint-disable no-wildcard-imports
        "#,
    )
    .has_lint_violation(6, 20, DISABLE)
    .is_formatted_as(&t(r#"
        /* Some copyright notice before package statement */
        @file:Suppress("aaa", "ktlint:standard:no-wildcard-imports", "zzz")
        package foo

        import foo.bar
        import foobar.*
        "#));
}
