//! ktlint fuzz crashes (research/24-fuzzing.md), minimized; expectations are the 2.0.0-ALPHA-4 jar's `-F`.

use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::{AutocorrectDecision, Code, KtLintException, KtLintRuleEngine};

fn format(text: &str) -> Result<String, KtLintException> {
    let engine = KtLintRuleEngine::new(standard_rule_providers());
    engine.format(&Code::from_snippet(text, false), &mut |_| AutocorrectDecision::AllowAutocorrect)
}

fn assert_rule_throws(text: &str, rule_id: &str, cause: &str) {
    match format(text) {
        Err(KtLintException::Rule(e)) => {
            assert_eq!(e.rule_id, rule_id);
            assert_eq!(e.cause, cause);
        }
        other => panic!("expected {rule_id} to throw, got {other:?}"),
    }
}

/// comment-spacing replaces the EOL comment leaf, then no-trailing-spaces visits the detached old leaf and emits
/// an offset in that leaf's own tree; mapping it to UTF-16 through the file's tree sliced inside the `é`.
#[test]
fn emit_from_a_replaced_node_maps_its_offset_in_the_detached_tree() {
    assert_eq!(format("// é\n//xy \n").unwrap(), "// é\n// xy\n");
}

/// curly-spacing's `addChildren` runs past the last sibling: IntelliJ dereferences the null `treeNext`.
#[test]
fn add_children_past_the_end_throws_the_jars_npe() {
    assert_rule_throws(
        "fun foo()/** */ {}\n",
        "standard:curly-spacing",
        "java.lang.NullPointerException: Cannot invoke \"org.jetbrains.kotlin.com.intellij.lang.ASTNode.getTreeNext()\" because \"f\" is null",
    );
}

/// colon-spacing anchors `addChild` on a leaf of another parent: `LOG.assertTrue` throws in ktlint's environment.
#[test]
fn add_child_with_a_foreign_anchor_throws_the_jars_assertion() {
    let text = "class/**    Code block at the beginning!\n * Some text\n ***    Code Block (not well-formed)\n \
                *    Code Block 2 (not well-formed)\n */vararg/**\n */:Any";
    assert_rule_throws(
        text,
        "standard:colon-spacing",
        "java.lang.Throwable: Assertion failed: anchorBefore == null || anchorBefore.getTreeParent() == parent",
    );
}
