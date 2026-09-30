//! Port of ktlint-rule-engine `api/KtLintTest.kt`: the API consumer cases, hook order, BOM handling and
//! the state/suppression cases (traversal stopping is in `engine_ktlint_traversal.rs`).

mod engine_common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use engine_common::{
    Calls, RuleExecutionCall, RuleMethod::*, SimpleTestRule, VisitNodeType::*, never,
};
use ktrs_ast::{Ast, NodeId};
use ktrs_lint::editorconfig::{END_OF_LINE_PROPERTY, EndOfLineValue};
use ktrs_lint::{
    AstNodeEdit, AutocorrectDecision, Code, EditorConfig, EditorConfigOverride, Emit,
    KtLintRuleEngine, RuleId, RuleV2, RuleV2Provider,
};
use ktrs_syntax::SyntaxKind::{
    EOL_COMMENT, FILE, IMPORT_LIST, PACKAGE_DIRECTIVE, REGULAR_STRING_PART, WHITE_SPACE,
};

/// `DummyRule { node -> ... }` counting root visits.
struct DummyRule(Arc<AtomicUsize>);

impl RuleV2 for DummyRule {
    fn rule_id(&self) -> RuleId {
        RuleId("test:dummy")
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, _emit: &mut Emit<'_>) {
        if ast.element_type(node) == FILE {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
}

const AUTOCORRECT_ERROR_RULE_ID: RuleId = RuleId("test:auto-correct");
const STRING_VALUE_TO_BE_AUTOCORRECTED: &str = "string-value-to-be-autocorrected";
const STRING_VALUE_NOT_TO_BE_CORRECTED: &str = "string-value-not-to-be-corrected";
const STRING_VALUE_AFTER_AUTOCORRECT: &str = "string-value-after-autocorrect";
const ERROR_MESSAGE_CAN_BE_AUTOCORRECTED: &str =
    "This string value is not allowed and can be autocorrected";
const ERROR_MESSAGE_CAN_NOT_BE_AUTOCORRECTED: &str =
    "This string value is not allowed but can not be autocorrected";

struct AutoCorrectErrorRule;

impl RuleV2 for AutoCorrectErrorRule {
    fn rule_id(&self) -> RuleId {
        AUTOCORRECT_ERROR_RULE_ID
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == REGULAR_STRING_PART {
            match ast.text(node).as_str() {
                STRING_VALUE_TO_BE_AUTOCORRECTED => {
                    emit(
                        ast,
                        ast.start_offset(node),
                        ERROR_MESSAGE_CAN_BE_AUTOCORRECTED,
                        true,
                    )
                    .if_autocorrect_allowed(|| {
                        ast.replace_text_with(node, STRING_VALUE_AFTER_AUTOCORRECT)
                    });
                }
                STRING_VALUE_NOT_TO_BE_CORRECTED => {
                    emit(
                        ast,
                        ast.start_offset(node),
                        ERROR_MESSAGE_CAN_NOT_BE_AUTOCORRECTED,
                        false,
                    );
                }
                _ => {}
            }
        }
    }
}

fn provider(rule: fn() -> Box<dyn RuleV2>) -> Vec<RuleV2Provider> {
    vec![RuleV2Provider::new(rule)]
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CallbackResult(usize, usize, RuleId, String, bool, bool);

#[test]
fn given_an_api_consumer_given_lint_given_a_non_empty_rule_providers_then_do_not_throw_an_error() {
    let visited = Arc::new(AtomicUsize::new(0));
    let v = visited.clone();
    let engine = KtLintRuleEngine::new(vec![RuleV2Provider::new(move || {
        Box::new(DummyRule(v.clone())) as Box<dyn RuleV2>
    })]);
    engine
        .lint(&Code::from_snippet("fun main() {}", false), &mut |_| {})
        .unwrap();
    assert_eq!(visited.load(Ordering::Relaxed), 1);
}

#[test]
fn given_an_api_consumer_given_lint_given_errors_which_can_and_can_not_be_autocorrected() {
    let code = format!(
        "val foo = \"{STRING_VALUE_NOT_TO_BE_CORRECTED}\"\nval bar = \"{STRING_VALUE_TO_BE_AUTOCORRECTED}\""
    );
    let mut callbacks = Vec::new();
    KtLintRuleEngine::new(provider(|| Box::new(AutoCorrectErrorRule)))
        .lint(&Code::from_snippet(&code, false), &mut |e| {
            callbacks.push(CallbackResult(
                e.line,
                e.col,
                e.rule_id,
                e.detail.clone(),
                e.can_be_auto_corrected,
                false,
            ));
        })
        .unwrap();
    assert_eq!(
        callbacks,
        [
            CallbackResult(
                1,
                12,
                AUTOCORRECT_ERROR_RULE_ID,
                ERROR_MESSAGE_CAN_NOT_BE_AUTOCORRECTED.into(),
                false,
                false
            ),
            CallbackResult(
                2,
                12,
                AUTOCORRECT_ERROR_RULE_ID,
                ERROR_MESSAGE_CAN_BE_AUTOCORRECTED.into(),
                true,
                false
            ),
        ]
    );
}

#[test]
fn given_an_api_consumer_given_format_given_a_non_empty_rule_providers_then_do_not_throw_an_error()
{
    let visited = Arc::new(AtomicUsize::new(0));
    let v = visited.clone();
    let engine = KtLintRuleEngine::new(vec![RuleV2Provider::new(move || {
        Box::new(DummyRule(v.clone())) as Box<dyn RuleV2>
    })]);
    engine
        .format(&Code::from_snippet("fun main() {}", false), &mut |_| {
            AutocorrectDecision::AllowAutocorrect
        })
        .unwrap();
    assert_eq!(visited.load(Ordering::Relaxed), 1);
}

#[test]
fn given_an_api_consumer_given_format_given_errors_which_can_and_can_not_be_autocorrected() {
    let code = format!(
        "val foo = \"{STRING_VALUE_NOT_TO_BE_CORRECTED}\"\nval bar = \"{STRING_VALUE_TO_BE_AUTOCORRECTED}\""
    );
    let formatted_code = format!(
        "val foo = \"{STRING_VALUE_NOT_TO_BE_CORRECTED}\"\nval bar = \"{STRING_VALUE_AFTER_AUTOCORRECT}\""
    );
    let mut callbacks: Vec<CallbackResult> = Vec::new();
    let actual = KtLintRuleEngine::new(provider(|| Box::new(AutoCorrectErrorRule)))
        .format(&Code::from_snippet(&code, false), &mut |e| {
            let result = CallbackResult(
                e.line,
                e.col,
                e.rule_id,
                e.detail.clone(),
                e.can_be_auto_corrected,
                e.can_be_auto_corrected,
            );
            if !callbacks.contains(&result) {
                callbacks.push(result);
            }
            if e.can_be_auto_corrected {
                AutocorrectDecision::AllowAutocorrect
            } else {
                AutocorrectDecision::NoAutocorrect
            }
        })
        .unwrap();
    assert_eq!(actual, formatted_code);
    assert_eq!(
        callbacks,
        [
            CallbackResult(
                1,
                12,
                AUTOCORRECT_ERROR_RULE_ID,
                ERROR_MESSAGE_CAN_NOT_BE_AUTOCORRECTED.into(),
                false,
                false
            ),
            CallbackResult(
                2,
                12,
                AUTOCORRECT_ERROR_RULE_ID,
                ERROR_MESSAGE_CAN_BE_AUTOCORRECTED.into(),
                true,
                true
            ),
        ]
    );
}

#[test]
fn given_a_normal_rule_then_execute_on_root_node_and_child_nodes() {
    let calls: Calls = Arc::new(Mutex::new(Vec::new()));
    let (a, b) = (RuleId("simple-test:a"), RuleId("simple-test:b"));
    KtLintRuleEngine::new(vec![
        SimpleTestRule::provider(&calls, a, false, never, never),
        SimpleTestRule::provider(&calls, b, false, never, never),
    ])
    .lint(&Code::from_snippet("", false), &mut |_| {})
    .unwrap();
    let node = |id, m, v, t| RuleExecutionCall::node(id, m, v, t, None);
    assert_eq!(
        *calls.lock().unwrap(),
        [
            RuleExecutionCall::new(a, BeforeFirst),
            RuleExecutionCall::new(b, BeforeFirst),
            node(a, BeforeChildren, Root, FILE),
            node(b, BeforeChildren, Root, FILE),
            node(a, BeforeChildren, Child, PACKAGE_DIRECTIVE),
            node(b, BeforeChildren, Child, PACKAGE_DIRECTIVE),
            node(a, AfterChildren, Child, PACKAGE_DIRECTIVE),
            node(b, AfterChildren, Child, PACKAGE_DIRECTIVE),
            node(a, BeforeChildren, Child, IMPORT_LIST),
            node(b, BeforeChildren, Child, IMPORT_LIST),
            node(a, AfterChildren, Child, IMPORT_LIST),
            node(b, AfterChildren, Child, IMPORT_LIST),
            node(a, AfterChildren, Root, FILE),
            node(b, AfterChildren, Root, FILE),
            RuleExecutionCall::new(a, AfterLast),
            RuleExecutionCall::new(b, AfterLast),
        ]
    );
}

/// Stand-in for IndentationRule on the BOM spec files: removes the indent before an EOL comment.
struct UnindentCommentRule;

impl RuleV2 for UnindentCommentRule {
    fn rule_id(&self) -> RuleId {
        RuleId("test:unindent-comment")
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        let next_is_comment = ast
            .tree_next(node)
            .is_some_and(|n| ast.element_type(n) == EOL_COMMENT);
        if ast.element_type(node) == WHITE_SPACE
            && next_is_comment
            && ast.leaf_text(node) != "\n"
            && ast.leaf_text(node).contains('\n')
        {
            emit(ast, ast.start_offset(node), "Unexpected indentation", true)
                .if_autocorrect_allowed(|| ast.replace_text_with(node, "\n"));
        }
    }
}

fn format_spec(before: &str, after: &str) {
    // third_party/ is in the main checkout, above a worktree.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|a| a.join("third_party/ktlint/ktlint-rule-engine/src/test/resources/spec"))
        .find(|d| d.is_dir())
        .expect("third_party/ktlint (tools/sync-ktlint.sh)");
    let read = |name: &str| {
        std::fs::read_to_string(dir.join(name))
            .unwrap()
            .replace("\r\n", "\n")
            .replace('\r', "\n")
    };
    let engine = KtLintRuleEngine::with_editor_config(
        provider(|| Box::new(UnindentCommentRule)),
        Default::default(),
        EditorConfigOverride::empty().with(&END_OF_LINE_PROPERTY, EndOfLineValue::Lf),
    );
    let actual = engine
        .format(&Code::from_snippet(&read(before), false), &mut |_| {
            AutocorrectDecision::AllowAutocorrect
        })
        .unwrap();
    assert_eq!(actual, read(after));
}

// Upstream formats with IndentationRule; UnindentCommentRule makes the same edit on these files.
#[test]
fn given_a_file_that_starts_with_the_utf8_bom_character_then_the_formatted_file_starts_with_it_as_well()
 {
    format_spec(
        "format-unicode-bom-at-start-of-file--before-ktlint-format.kt",
        "format-unicode-bom-at-start-of-file--after-ktlint-format.kt",
    );
}

#[test]
fn issue_3220_given_code_that_contains_the_utf8_bom_character_somewhere_but_not_at_the_start_then_it_is_not_removed()
 {
    format_spec(
        "do-not-format-unicode-bom-when-not-at-start-of-file--before-ktlint-format.kt",
        "do-not-format-unicode-bom-when-not-at-start-of-file--after-ktlint-format.kt",
    );
}

/// Throws when an instance is used for a second traversal.
struct WithStateRule {
    has_not_been_visited_yet: bool,
}

impl RuleV2 for WithStateRule {
    fn rule_id(&self) -> RuleId {
        RuleId("test:with-state")
    }

    fn before_first_node(&mut self, _editor_config: &EditorConfig) {
        assert!(
            self.has_not_been_visited_yet,
            "IllegalStateException: Rule has been visited before"
        );
        self.has_not_been_visited_yet = false;
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        emit(
            ast,
            ast.start_offset(node),
            "Fake violation which can be autocorrected",
            true,
        );
    }
}

#[test]
fn given_that_format_is_started_using_the_rule_providers_parameter_then_no_exception_is_thrown() {
    KtLintRuleEngine::new(provider(|| {
        Box::new(WithStateRule {
            has_not_been_visited_yet: true,
        })
    }))
    .format(&Code::from_snippet("", false), &mut |_| {
        AutocorrectDecision::AllowAutocorrect
    })
    .unwrap();
}

#[test]
fn issue_1623_given_a_file_with_multiple_top_level_declarations_then_a_file_suppression_applies_on_each()
 {
    let code = format!(
        "@file:Suppress(\"ktlint:{}\")\nval foo = \"{STRING_VALUE_TO_BE_AUTOCORRECTED}\" // Won't be auto corrected due to suppress annotation\n\
         val bar = \"{STRING_VALUE_TO_BE_AUTOCORRECTED}\" // Won't be auto corrected due to suppress annotation",
        AUTOCORRECT_ERROR_RULE_ID.value()
    );
    let actual = KtLintRuleEngine::new(provider(|| Box::new(AutoCorrectErrorRule)))
        .format(&Code::from_snippet(&code, false), &mut |_| {
            AutocorrectDecision::AllowAutocorrect
        })
        .unwrap();
    assert_eq!(actual, code);
}
