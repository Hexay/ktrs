//! ktlint 2.0.0-ALPHA-4's `standard:indent` throws `Stack should be empty` on 64 corpus files in format mode (lint
//! passes); the port must throw the same message, which prints each left-over `IndentContext` via `ASTNode.toString()`.
//! Input: corpus kotlinx.coroutines/.../guide/example-flow-09.kt; expected: the oracle's failed.tsv row for it.

use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::{AutocorrectDecision, Code, KtLintException, KtLintRuleEngine};

const INPUT: &str = "package kotlinx.coroutines.guide.exampleFlow09

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*

fun main() = runBlocking<Unit> { \n    val nums = (1..3).asFlow() // numbers 1..3
    val strs = flowOf(\"one\", \"two\", \"three\") // strings \n    nums.zip(strs) { a, b -> \"$a -> $b\" } // compose a single string
        .collect { println(it) } // collect and print
}
";

const EXPECTED: &str = "IllegalArgumentException: Stack should be empty:
\tIndentContext(fromASTNode=Element(kotlin.FILE), toASTNode=PsiWhiteSpace, nodeIndent=, firstChildIndent=, childIndent=, lastChildIndent=, activated=true)
\tIndentContext(fromASTNode=PsiElement(EQ), toASTNode=PsiElement(RBRACE), nodeIndent=, firstChildIndent=    , childIndent=    , lastChildIndent=    , activated=true)
\tIndentContext(fromASTNode=PsiElement(LBRACE), toASTNode=PsiElement(RBRACE), nodeIndent=    , firstChildIndent=, childIndent=    , lastChildIndent=, activated=true)
\tIndentContext(fromASTNode=PsiElement(IDENTIFIER), toASTNode=PsiComment(EOL_COMMENT), nodeIndent=        , firstChildIndent=    , childIndent=    , lastChildIndent=    , activated=true)";

#[test]
fn format_throws_the_jars_stack_should_be_empty_message() {
    let engine = KtLintRuleEngine::new(standard_rule_providers());
    let code = Code::from_snippet(INPUT, false);
    match engine.format(&code, &mut |_| AutocorrectDecision::AllowAutocorrect) {
        Err(KtLintException::Rule(e)) => {
            assert_eq!(e.rule_id, "standard:indent");
            assert_eq!(e.cause, EXPECTED);
        }
        other => panic!("expected the indent rule to throw, got {other:?}"),
    }
    assert!(engine.lint(&code, &mut |_| {}).is_ok(), "lint does not throw on this file, as in ktlint");
}
