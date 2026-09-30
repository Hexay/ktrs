//! Port of `KtlintSuppressionRuleTest.kt`: merging into `@file` annotations, init blocks, sibling directives,
//! `Given ktlint-disable directive in block comment not having a ktlint-enable directive ...`, package
//! statements and unknown rule ids.

mod support;

use support::assert_that::{trim_indent as t, v, v_manual};
use support::suppression_rule::{
    DISABLE, ENABLE, NO_MATCHING_ENABLE, ktlint_suppression_rule_assert_that as assert_that,
};

// skipped: documentation_example needs standard:argument-list-wrapping

#[test]
fn given_multiple_ktlint_disable_directives_which_have_to_merged_into_an_existing_file_suppress_annotation()
 {
    for annotation_name in ["Suppress", "SuppressWarnings"] {
        let code = t(r#"
            @file:ANNOTATION("ktlint:standard:bar")

            import bar // ktlint-disable standard:no-wildcard-imports

            /* ktlint-disable standard:foo */

            val someFoo = foo.TEST
            val someBar = bar.TEST

            /* ktlint-disable custom:foo */
            "#)
        .replace("ANNOTATION", annotation_name);
        let formatted_code = t(r#"
            @file:ANNOTATION("ktlint:custom:foo", "ktlint:standard:bar", "ktlint:standard:foo", "ktlint:standard:no-wildcard-imports")

            import bar

            val someFoo = foo.TEST
            val someBar = bar.TEST
            "#)
        .replace("ANNOTATION", annotation_name);
        assert_that(&code)
            .has_lint_violations(&[v(3, 15, DISABLE), v(5, 4, DISABLE), v(10, 4, DISABLE)])
            .is_formatted_as(&formatted_code);
    }
}

#[test]
fn given_a_block_comment_containing_a_ktlint_disable_directive_inside_an_init_block() {
    assert_that(
        r#"
        class Foo() {
            var foo: String
            var bar: String

            init {
                /* ktlint-disable standard:foo */
                foo = "foo"
                /* ktlint-enable standard:foo */
            }

            init { // ktlint-disable standard:bar
                bar = "bar"
            }
        }
        "#,
    )
    .has_lint_violations(&[v(6, 12, DISABLE), v(8, 12, ENABLE), v(11, 15, DISABLE)])
    .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:bar", "ktlint:standard:foo")
        class Foo() {
            var foo: String
            var bar: String

            init {
                foo = "foo"
            }

            init {
                bar = "bar"
            }
        }
        "#));
}

#[test]
fn given_a_pair_of_matching_ktlint_directives_in_block_comments_as_siblings_in_same_parent_node() {
    assert_that(
        r#"
        fun foobar(
            /* ktlint-disable standard:foo */
            foo: Int,
            bar: Int,
            /* ktlint-enable standard:foo */
        ) {}
        "#,
    )
    .has_lint_violations(&[v(2, 8, DISABLE), v(5, 8, ENABLE)])
    .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:foo")
        fun foobar(
            foo: Int,
            bar: Int,
        ) {}
        "#));
}

#[test]
fn given_a_ktlint_disable_directive_root_level_not_related_to_an_declaration_or_expression_then_move_to_file_annotation()
 {
    assert_that("/* ktlint-disable standard:foo */")
        .has_lint_violation(1, 4, DISABLE)
        .is_formatted_as(r#"@file:Suppress("ktlint:standard:foo")"#);
}

#[test]
fn given_ktlint_disable_directive_in_last_block_comment_before_class_but_not_having_a_ktlint_enable_directive()
 {
    assert_that(
        r#"
        /* ktlint-disable standard:foo */
        class Foo
        "#,
    )
    .has_lint_violation(1, 4, DISABLE)
    .is_formatted_as(&t(r#"
        @file:Suppress("ktlint:standard:foo")

        class Foo
        "#));
}

#[test]
fn given_ktlint_disable_directive_in_last_block_comment_before_property_but_not_having_a_ktlint_enable_directive()
 {
    assert_that(
        r#"
        /* ktlint-disable standard:foo */
        val foo = "foo"
        "#,
    )
    .has_lint_violation(1, 4, DISABLE)
    .is_formatted_as(&t(r#"
        @file:Suppress("ktlint:standard:foo")

        val foo = "foo"
        "#));
}

#[test]
fn given_a_pair_of_matching_ktlint_directives_in_block_comments_but_not_as_siblings_in_same_parent_node()
 {
    assert_that(
        r#"
        fun foobar(
            /* ktlint-disable standard:foo */
            foo: Int,
            bar: Int,
        ) {
            /* ktlint-enable standard:foo */
            doSomething()
        }
        "#,
    )
    .has_lint_violations(&[v_manual(2, 8, NO_MATCHING_ENABLE), v(6, 8, ENABLE)])
    .is_formatted_as(&t(r#"
        fun foobar(
            /* ktlint-disable standard:foo */
            foo: Int,
            bar: Int,
        ) {
            doSomething()
        }
        "#));
}

#[test]
fn given_ktlint_disable_directive_on_a_package_statement() {
    assert_that("package foo.foo_bar // ktlint-disable standard:package-name")
        .has_lint_violation(1, 24, DISABLE)
        .is_formatted_as(&t(r#"
            @file:Suppress("ktlint:standard:package-name")

            package foo.foo_bar
            "#));
}

#[test]
fn given_an_invalid_rule_id_then_ignore_it_without_throwing_an_exception() {
    assert_that(
        r#"
        @file:Suppress("ktlint:standard:SOME-INVALID-RULE-ID-1")

        @Suppress("ktlint:standard:SOME-INVALID-RULE-ID-2")
        class Foo {
            /* ktlint-disable standard:SOME-INVALID-RULE-ID-3 */
            fun bar() {
                val bar = "bar" // ktlint-disable standard:SOME-INVALID-RULE-ID-4
            }
            /* ktlint-enable standard:SOME-INVALID-RULE-ID-3 */
        }
        "#,
    )
    .has_lint_violations(&[
        v_manual(
            1,
            17,
            "Ktlint rule with id 'ktlint:standard:SOME-INVALID-RULE-ID-1' is unknown or not loaded",
        ),
        v_manual(
            3,
            12,
            "Ktlint rule with id 'ktlint:standard:SOME-INVALID-RULE-ID-2' is unknown or not loaded",
        ),
        v(5, 8, DISABLE),
        v_manual(
            5,
            23,
            "Ktlint rule with id 'standard:SOME-INVALID-RULE-ID-3' is unknown or not loaded",
        ),
        v(7, 28, DISABLE),
        v_manual(
            7,
            43,
            "Ktlint rule with id 'standard:SOME-INVALID-RULE-ID-4' is unknown or not loaded",
        ),
        v(9, 8, ENABLE),
    ]);
}

#[test]
fn given_an_unknown_ktlint_rule_id_then_do_not_create_an_empty_suppress_annotation() {
    assert_that(r#"val foo = "foo" // ktlint-disable standard:unknown-rule-id"#)
        .has_lint_violations(&[
            v(1, 20, DISABLE),
            v_manual(
                1,
                35,
                "Ktlint rule with id 'standard:unknown-rule-id' is unknown or not loaded",
            ),
        ])
        .is_formatted_as(r#"val foo = "foo""#);
}
