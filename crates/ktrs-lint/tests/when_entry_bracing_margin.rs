//! ktlint 2.0.0-ALPHA-4's `when-entry-bracing` rebuilds the entry from `"""|when {|$entry|}""".trimMargin()`, so `|`
//! margins inside the entry's own raw strings are stripped too (held-out corpus: wire's Roots.kt, kotest's
//! CommutativeEquality.kt). Expected: the jar's `-F` output for this input (ktlint_official).

use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::{AutocorrectDecision, Code, KtLintRuleEngine};

const INPUT: &str = r#"fun f(x: Int): Int =
    when (x) {
        1 -> {
            1
        }
        else -> throw IllegalArgumentException(
            """
            |a
            |b
            """.trimMargin(),
        )
    }
"#;

const EXPECTED: &str = r#"fun f(x: Int): Int =
    when (x) {
        1 -> {
            1
        }

        else -> {
            throw IllegalArgumentException(
                """
a
b
                """.trimMargin(),
            )
        }
    }
"#;

#[test]
fn format_strips_margins_inside_the_braced_entry_like_the_jar() {
    let engine = KtLintRuleEngine::new(standard_rule_providers());
    let code = Code::from_snippet(INPUT, false);
    let formatted = engine.format(&code, &mut |_| AutocorrectDecision::AllowAutocorrect).expect("format");
    assert_eq!(formatted, EXPECTED);
}
