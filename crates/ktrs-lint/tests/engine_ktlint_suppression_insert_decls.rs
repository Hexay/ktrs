//! Port of `KtlintSuppressionKtTest.kt`: init blocks, setters, constructors, the type argument / type
//! parameter / value argument / value parameter lists, named arguments, property delegates and nested expressions.

mod support;

use support::assert_that::trim_indent as t;
use support::insert::{at_offset, char_at};

const FOO: &str = "standard:foo";

#[test]
fn given_an_init_block_comment_to_which_an_suppression_is_being_added() {
    let code = r#"
        class Foo() {
            var foo: String
            var bar: String

            init {
                foo = "foo"
            }
        }
        "#;
    assert_eq!(
        at_offset(code, 6, 1, FOO),
        format!("@Suppress(\"ktlint:standard:foo\")\n{}", t(code))
    );
}

#[test]
fn given_a_setter_on_which_a_suppression_is_added() {
    let code = r#"
        class Foo {
            var foo: Int = 1
                set(value) {
                    field = value
                }
        }
        "#;
    assert_eq!(
        at_offset(code, 4, 1, FOO),
        t(r#"
        class Foo {
            var foo: Int = 1
                @Suppress("ktlint:standard:foo")
                set(value) {
                    field = value
                }
        }
        "#)
    );
}

#[test]
fn given_a_primary_constructor_on_which_a_suppression_is_added() {
    let code = r#"
        class Foo constructor(bar: Bar) {
            // foo
        }
        "#;
    assert_eq!(
        at_offset(code, 2, 8, FOO),
        format!("@Suppress(\"ktlint:standard:foo\")\n{}", t(code))
    );
}

const TYPE_ARGUMENTS: &str = r#"
    fun FooBar<
        in FOO,
        in BAR
        >.foo(foo: FOO, bar: BAR) {}
    "#;

#[test]
fn given_a_type_argument_on_which_a_suppression_is_added() {
    assert_eq!(
        at_offset(TYPE_ARGUMENTS, 2, 5, FOO),
        t(r#"
        fun FooBar<
            @Suppress("ktlint:standard:foo")
            in FOO,
            in BAR
            >.foo(foo: FOO, bar: BAR) {}
        "#)
    );
}

#[test]
fn given_a_type_argument_on_which_a_suppression_is_added_on_the_comma_or_non_code_leaf_in_the_type_argument_list()
 {
    assert_eq!(char_at(TYPE_ARGUMENTS, 2, 11), ',');
    assert_eq!(
        at_offset(TYPE_ARGUMENTS, 2, 11, FOO),
        t(r#"
        fun FooBar<
            in FOO,
            @Suppress("ktlint:standard:foo")
            in BAR
            >.foo(foo: FOO, bar: BAR) {}
        "#)
    );
}

const TYPE_PARAMETERS: &str = r#"
    fun <
        FOO,
        BAR,
        > foobar(foo: FOO, bar: BAR) = "foo"
    "#;

#[test]
fn given_a_type_parameter_on_which_a_suppression_is_added() {
    assert_eq!(
        at_offset(TYPE_PARAMETERS, 2, 5, FOO),
        t(r#"
        fun <
            @Suppress("ktlint:standard:foo")
            FOO,
            BAR,
            > foobar(foo: FOO, bar: BAR) = "foo"
        "#)
    );
}

#[test]
fn given_a_type_parameter_on_which_a_suppression_is_added_on_the_comma_or_non_code_leaf_in_the_type_parameter_list()
 {
    assert_eq!(char_at(TYPE_PARAMETERS, 2, 8), ',');
    assert_eq!(
        at_offset(TYPE_PARAMETERS, 2, 8, FOO),
        t(r#"
        fun <
            FOO,
            @Suppress("ktlint:standard:foo")
            BAR,
            > foobar(foo: FOO, bar: BAR) = "foo"
        "#)
    );
}

const VALUE_ARGUMENTS: &str = r#"
    val foobar = foobar(
        foo = "foo",
        bar = "bar",
    )
    "#;

#[test]
fn given_a_value_argument_on_which_a_suppression_is_added() {
    assert_eq!(
        at_offset(VALUE_ARGUMENTS, 2, 5, FOO),
        t(r#"
        val foobar = foobar(
            @Suppress("ktlint:standard:foo")
            foo = "foo",
            bar = "bar",
        )
        "#)
    );
}

#[test]
fn given_a_value_argument_on_which_a_suppression_is_added_on_the_comma_or_non_code_leaf_in_the_value_argument_list()
 {
    assert_eq!(char_at(VALUE_ARGUMENTS, 2, 16), ',');
    assert_eq!(
        at_offset(VALUE_ARGUMENTS, 2, 16, FOO),
        t(r#"
        val foobar = foobar(
            foo = "foo",
            @Suppress("ktlint:standard:foo")
            bar = "bar",
        )
        "#)
    );
}

#[test]
fn given_a_class_parameter_on_which_a_suppression_is_added() {
    let code = r#"
        class Foobar(
            val foo: Foo,
        )
        "#;
    assert_eq!(
        at_offset(code, 2, 9, FOO),
        t(r#"
        class Foobar(
            @Suppress("ktlint:standard:foo")
            val foo: Foo,
        )
        "#)
    );
}

#[test]
fn given_a_value_parameter_on_which_a_suppression_is_added_on_the_comma_or_non_code_leaf_in_the_value_parameter_list()
 {
    let code = r#"
        class Foobar(
            val foo: Foo,
            val bar: Bar
        )
        "#;
    assert_eq!(char_at(code, 2, 17), ',');
    assert_eq!(
        at_offset(code, 2, 17, FOO),
        t(r#"
        class Foobar(
            val foo: Foo,
            @Suppress("ktlint:standard:foo")
            val bar: Bar
        )
        "#)
    );
}

#[test]
fn given_a_declaration_with_a_suppress_annotation_using_a_named_argument_and_a_suppression() {
    let code = r#"
        @Suppress(names = ["unused"])
        val foo = "foo"
        "#;
    assert_eq!(
        at_offset(code, 2, 5, FOO),
        t(r#"
        @Suppress("ktlint:standard:foo", "unused")
        val foo = "foo"
        "#)
    );
}

#[test]
fn given_a_suppression_which_is_added_on_a_property_delegate() {
    let code = r#"
        val foo by lazy(LazyThreadSafetyMode.PUBLICATION) {
            // do something
        }
        "#;
    assert_eq!(
        at_offset(code, 1, 12, FOO),
        t(r#"
        val foo by @Suppress("ktlint:standard:foo")
        lazy(LazyThreadSafetyMode.PUBLICATION) {
            // do something
        }
        "#)
    );
}

#[test]
fn given_a_nested_expression_on_which_a_suppression_is_added() {
    let code = r#"
        val foo =
            setOf("a")
                .map {
                    bar(it)
                }
        "#;
    assert_eq!(
        at_offset(code, 4, 13, FOO),
        t(r#"
        val foo =
            setOf("a")
                .map {
                    @Suppress("ktlint:standard:foo")
                    bar(it)
                }
        "#)
    );
}
