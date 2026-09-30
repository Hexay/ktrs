//! Port of `KtlintSuppressionRuleTest.kt`: directives on setters, constructors, parameters, declarations,
//! property delegates and nested expressions.

mod support;

use support::assert_that::{trim_indent as t, v};
use support::suppression_rule::{
    DISABLE, ENABLE, ktlint_suppression_rule_assert_that as assert_that,
};

#[test]
fn given_a_setter_with_multiple_ktlint_directives() {
    assert_that(
        r#"
        class Foo {
            var foo: Int = 1
                set(value) { // ktlint-disable standard:foo
                    field = value // ktlint-disable standard:bar
                    field = value
                }
        }
        "#,
    )
    .has_lint_violations(&[v(3, 25, DISABLE), v(4, 30, DISABLE)])
    .is_formatted_as(&t(r#"
        class Foo {
            var foo: Int = 1
                @Suppress("ktlint:standard:bar", "ktlint:standard:foo")
                set(value) {
                    field = value
                    field = value
                }
        }
        "#));
}

#[test]
fn given_a_primary_constructor_with_multiple_ktlint_directives() {
    assert_that(
        r#"
        class Foo constructor(bar: Bar) {
            /* ktlint-disable standard:bar standard:foo */

            /* ktlint-enable standard:bar standard:foo */
        }
        "#,
    )
    .has_lint_violations(&[v(2, 8, DISABLE), v(4, 8, ENABLE)])
    .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:bar", "ktlint:standard:foo")
        class Foo constructor(bar: Bar) {
        }
        "#));
}

#[test]
fn given_a_class_with_a_single_parameter_wrapped_between_ktlint_disable_and_ktlint_enable_directives()
 {
    assert_that(
        r#"
        class Foo(
            /* ktlint-disable standard:bar standard:foo */
            val bar: Bar
            /* ktlint-enable standard:bar standard:foo */
        )
        "#,
    )
    .has_lint_violations(&[v(2, 8, DISABLE), v(4, 8, ENABLE)])
    .is_formatted_as(&t(r#"
        class Foo(
            @Suppress("ktlint:standard:bar", "ktlint:standard:foo")
            val bar: Bar
        )
        "#));
}

#[test]
fn given_a_class_with_multiple_parameters_wrapped_between_ktlint_disable_and_ktlint_enable_directives()
 {
    assert_that(
        r#"
        class Foo(
            /* ktlint-disable standard:bar standard:foo */
            val bar1: Bar,
            val bar2: Bar,
            /* ktlint-enable standard:bar standard:foo */
        )
        "#,
    )
    .has_lint_violations(&[v(2, 8, DISABLE), v(5, 8, ENABLE)])
    .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:bar", "ktlint:standard:foo")
        class Foo(
            val bar1: Bar,
            val bar2: Bar,
        )
        "#));
}

#[test]
fn given_a_ktlint_disable_block_directive_around_a_single_declaration_then_place_the_suppress_on_the_declaration()
 {
    assert_that(
        r#"
        class Foobar {
            val bar = "bar"

            /* ktlint-disable standard:bar standard:foo */
            fun foo() {}
            /* ktlint-enable standard:bar standard:foo */
        }
        "#,
    )
    .has_lint_violations(&[v(4, 8, DISABLE), v(6, 8, ENABLE)])
    .is_formatted_as(&t(r#"
        class Foobar {
            val bar = "bar"

            @Suppress("ktlint:standard:bar", "ktlint:standard:foo")
            fun foo() {}
        }
        "#));
}

#[test]
fn given_a_ktlint_disable_block_directive_around_multiple_declarations_then_place_the_suppress_on_the_declaration()
 {
    assert_that(
        r#"
        class Foobar {
            /* ktlint-disable standard:bar standard:foo */
            val bar = "bar"

            fun foo() {}
            /* ktlint-enable standard:bar standard:foo */
        }
        "#,
    )
    .has_lint_violations(&[v(2, 8, DISABLE), v(6, 8, ENABLE)])
    .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:bar", "ktlint:standard:foo")
        class Foobar {
            val bar = "bar"

            fun foo() {}
        }
        "#));
}

#[test]
fn given_a_declaration_with_a_suppress_annotation_using_a_named_argument_and_a_ktlint_disable_directive()
 {
    assert_that(
        r#"
        @Suppress(names = ["unused"])
        val foo = "foo" // ktlint-disable standard:foo
        "#,
    )
    .has_lint_violation(2, 20, DISABLE)
    .is_formatted_as(&t(r#"
        @Suppress("ktlint:standard:foo", "unused")
        val foo = "foo"
        "#));
}

#[test]
fn given_a_property_delegate_with_a_ktlint_disable_directive() {
    assert_that(
        r#"
        val foo by lazy(LazyThreadSafetyMode.PUBLICATION) { // ktlint-disable standard:foo
            // do something
        }
        "#,
    )
    .has_lint_violation(1, 56, DISABLE)
    .is_formatted_as(&t(r#"
        val foo by @Suppress("ktlint:standard:foo")
        lazy(LazyThreadSafetyMode.PUBLICATION) {
            // do something
        }
        "#));
}

#[test]
fn given_a_nested_expression_with_a_ktlint_disable_directive() {
    assert_that(
        r#"
        val foo =
            setOf("a")
                .map {
                    bar(it) // ktlint-disable standard:foo
                }
        "#,
    )
    .has_lint_violation(4, 24, DISABLE)
    .is_formatted_as(&t(r#"
        val foo =
            @Suppress("ktlint:standard:foo")
            setOf("a")
                .map {
                    bar(it)
                }
        "#));
}

// skipped: given_an_expression_containing_a_ktlint_block_directive_then_change_the_expression_to_an_annotated_expression_with_an_suppress_annotation
// needs standard:argument-list-wrapping (and max-line-length)
