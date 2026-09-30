//! Port of ktlint-rule-engine `internal/SuppressionLocatorTest.kt`, nested class
//! `Given that formatter tags are enabled`.

mod engine_common;

use engine_common::{lint_no_foo, no_foo_errors};
use ktrs_lint::editorconfig::PropertyRef;
use ktrs_lint::engine::{
    FORMATTER_TAG_OFF_ENABLED_PROPERTY, FORMATTER_TAG_ON_ENABLED_PROPERTY,
    FORMATTER_TAGS_ENABLED_PROPERTY,
};
use ktrs_lint::{EditorConfigOverride, LintError};

fn lint_with_tags(code: &str, custom_tags: bool) -> Vec<LintError> {
    let mut properties = vec![(
        PropertyRef::from(&*FORMATTER_TAGS_ENABLED_PROPERTY),
        Some("true".to_owned()),
    )];
    if custom_tags {
        properties.push((
            PropertyRef::from(&*FORMATTER_TAG_OFF_ENABLED_PROPERTY),
            Some("custom-formatter-tag-off".to_owned()),
        ));
        properties.push((
            PropertyRef::from(&*FORMATTER_TAG_ON_ENABLED_PROPERTY),
            Some("custom-formatter-tag-on".to_owned()),
        ));
    }
    lint_no_foo(
        code,
        EditorConfigOverride::from(properties),
        Vec::new(),
        true,
    )
}

#[test]
fn given_a_violation_suppressed_with_the_default_formatter_tags_in_a_block_comment() {
    let code = "/* @formatter:off */\nval fooNotReported = \"foo\"\n/* @formatter:on */\nval fooReported = \"foo\"";
    assert_eq!(lint_with_tags(code, false), no_foo_errors(4, 5));
}

#[test]
fn given_a_violation_suppressed_with_the_default_formatter_tags_in_eol_comments() {
    let code = "// @formatter:off\nval fooNotReported = \"foo\"\n// @formatter:on\nval fooReported = \"foo\"";
    assert_eq!(lint_with_tags(code, false), no_foo_errors(4, 5));
}

// skipped: needs IndentationRule — `Issue 2695 - Given that the formatter-on tag is not found in a block containing the formatter-off tag ...`
// skipped: needs IndentationRule — `Issue 2695 - Given that the formatter-on tag is applied on a non-block element ...`

/// Not upstream: the two issue-2695 cases above without IndentationRule, so the unclosed-tag ranges are
/// still checked (up to the containing `}`, and the next sibling when there is none).
#[test]
fn given_an_unclosed_formatter_off_tag_the_suppression_ends_at_the_containing_block_or_next_sibling()
 {
    let code = "val fooReported1 = \"foo\"\nfun bar() {\n    // @formatter:off\n    val fooNotReported1 = \"foo\"\n    \
                val fooNotReported2 = \"foo\"\n}\nval fooReported2 = \"foo\"";
    assert_eq!(
        lint_with_tags(code, false),
        [no_foo_errors(1, 5), no_foo_errors(7, 5)].concat()
    );
    let code = "val bar1 = fooReported()\nfun bar() =\n    // @formatter:off\n    fooNotReported()\nval bar2 = fooReported()";
    assert_eq!(
        lint_with_tags(code, false),
        [no_foo_errors(1, 12), no_foo_errors(5, 12)].concat()
    );
}

#[test]
fn given_a_violation_suppressed_with_custom_formatter_tags_in_block_comments() {
    let code = "/* custom-formatter-tag-off */\nval fooNotReported = \"foo\"\n/* custom-formatter-tag-on */\nval fooReported = \"foo\"";
    assert_eq!(lint_with_tags(code, true), no_foo_errors(4, 5));
}

#[test]
fn given_a_violation_suppressed_with_custom_formatter_tags_in_eol_comments() {
    let code = "// custom-formatter-tag-off\nval fooNotReported = \"foo\"\n// custom-formatter-tag-on\nval fooReported = \"foo\"";
    assert_eq!(lint_with_tags(code, true), no_foo_errors(4, 5));
}
