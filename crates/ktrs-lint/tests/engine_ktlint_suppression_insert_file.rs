//! Port of `KtlintSuppressionKtTest.kt`: `Given a file suppression to be inserted` (with `Given that all rules
//! for entire file have to be disabled`), the top level declaration and `@SuppressWarnings` cases.

mod support;

use ktrs_lint::RuleId;
use ktrs_lint::engine::KtlintSuppression;
use support::assert_that::trim_indent as t;
use support::insert::{at_offset, insert};

const NO_WILDCARD: &str = "standard:no-wildcard-imports";

#[test]
fn given_a_suppression_to_be_inserted_on_a_package_statement() {
    assert_eq!(
        at_offset("package foo.foo_bar", 1, 1, "standard:package-name"),
        t(r#"
        @file:Suppress("ktlint:standard:package-name")

        package foo.foo_bar
        "#)
    );
}

#[test]
fn given_a_suppression_to_be_inserted_on_an_import_statement() {
    assert_eq!(
        at_offset("import foo.*", 1, 1, NO_WILDCARD),
        t(r#"
        @file:Suppress("ktlint:standard:no-wildcard-imports")

        import foo.*
        "#)
    );
}

#[test]
fn given_code_with_a_file_suppression_but_no_suppression_ids_and_an_import_statement_then_insert_a_file_suppression()
 {
    let code = "@file:Suppress\n\nimport foo.*";
    assert_eq!(
        at_offset(code, 3, 1, NO_WILDCARD),
        "@file:Suppress(\"ktlint:standard:no-wildcard-imports\")\n\nimport foo.*"
    );
}

#[test]
fn given_code_with_a_file_suppression_already_containing_the_suppression_id_then_not_add_that_same_suppression_id_again()
 {
    let code = "@file:Suppress(\"ktlint:standard:no-wildcard-imports\")\n\nimport foo.*";
    assert_eq!(at_offset(code, 3, 1, NO_WILDCARD), code);
}

#[test]
fn given_code_with_a_file_suppression_not_containing_any_suppression_id_lexicograhically_bigger_than_the_new_id_then_add_it_as_last_element()
 {
    let code = "@file:Suppress(\"aaa\")\n\nimport foo.*";
    assert_eq!(
        at_offset(code, 3, 1, NO_WILDCARD),
        "@file:Suppress(\"aaa\", \"ktlint:standard:no-wildcard-imports\")\n\nimport foo.*"
    );
}

#[test]
fn given_code_with_a_file_suppression_not_containing_any_suppression_id_lexicograhically_smaller_than_the_new_id_then_add_it_as_first_element()
 {
    let code = "@file:Suppress(\"zzz\")\n\nimport foo.*";
    assert_eq!(
        at_offset(code, 3, 1, NO_WILDCARD),
        "@file:Suppress(\"ktlint:standard:no-wildcard-imports\", \"zzz\")\n\nimport foo.*"
    );
}

#[test]
fn given_code_with_a_file_suppression_having_unsorted_ids_and_the_new_suppression_id_is_lexicograhically_in_between_other_elements()
 {
    let code = "@file:Suppress(\"zzz\", \"aaa\")\n\nimport foo.*";
    assert_eq!(
        at_offset(code, 3, 1, NO_WILDCARD),
        "@file:Suppress(\"aaa\", \"ktlint:standard:no-wildcard-imports\", \"zzz\")\n\nimport foo.*"
    );
}

#[test]
fn given_code_with_a_copyright_comment_before_the_package_statement_then_insert_the_suppression_below_the_copyright_comment()
 {
    let code = r#"
        /* Some copyright notice before package statement */
        package foobar

        import foo.*
        "#;
    assert_eq!(
        at_offset(code, 4, 1, NO_WILDCARD),
        t(r#"
        /* Some copyright notice before package statement */
        @file:Suppress("ktlint:standard:no-wildcard-imports")

        package foobar

        import foo.*
        "#)
    );
}

/// `insertKtlintRuleSuppression(setOf("ktlint"), forceFileAnnotation = true)`: on the file node.
fn suppress_all_rules(code: &str) -> String {
    insert(
        code,
        KtlintSuppression::ForFile {
            rule_id: RuleId("ktlint"),
        },
    )
}

#[test]
fn given_that_no_file_annotation_is_defined() {
    assert_eq!(
        suppress_all_rules("import foo.*"),
        "@file:Suppress(\"ktlint\")\n\nimport foo.*"
    );
}

#[test]
fn given_that_a_file_annotation_is_defined_then_remove_ktlint_suppression_ids_only_as_they_become_redundant()
 {
    let code =
        "@file:Suppress(\"ktlint:standard:no-wildcard-imports\", \"unused\")\n\nimport foo.*";
    assert_eq!(
        suppress_all_rules(code),
        "@file:Suppress(\"ktlint\", \"unused\")\n\nimport foo.*"
    );
}

#[test]
fn given_a_top_level_declaration_at_which_a_suppression_is_to_be_added() {
    assert_eq!(
        at_offset(r#"val foo = "Foo""#, 1, 1, "standard:some-rule-id"),
        t(r#"
        @Suppress("ktlint:standard:some-rule-id")
        val foo = "Foo"
        "#)
    );
}

#[test]
fn given_a_ktlint_suppression_then_add_it_to_the_existing_suppress_warnings_and_sort_all_suppressions_alphabetically()
 {
    let code = r#"
        @SuppressWarnings("zzz", "aaa")
        val foo = "foo"
        "#;
    assert_eq!(
        at_offset(code, 2, 1, "standard:foo"),
        t(r#"
        @SuppressWarnings("aaa", "ktlint:standard:foo", "zzz")
        val foo = "foo"
        "#)
    );
}

#[test]
fn given_the_target_element_is_already_annotated_with_both_suppress_and_suppress_warnings_then_add_the_ktlint_suppression_to_the_suppress()
 {
    let code = r#"
        @Suppress("aaa", "zzz")
        @SuppressWarnings("bbb", "yyy")
        val foo = "foo"
        "#;
    assert_eq!(
        at_offset(code, 3, 1, "standard:foo"),
        t(r#"
        @Suppress("aaa", "ktlint:standard:foo", "zzz")
        @SuppressWarnings("bbb", "yyy")
        val foo = "foo"
        "#)
    );
}
