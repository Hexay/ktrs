//! Port of ktlint-rule-engine `api/KtLintTest.kt`, nested class `Given that the traversal is stopped`.

mod engine_common;

use std::sync::{Arc, Mutex};

use engine_common::{
    Calls, NodePredicate, RuleExecutionCall, RuleMethod::*, SimpleTestRule, VisitNodeType::Child,
    is_class_foo, never,
};
use ktrs_lint::{AutocorrectDecision, Code, KtLintRuleEngine, RuleId};
use ktrs_syntax::SyntaxKind::CLASS;

const RULE_ID_STOP_TRAVERSAL: RuleId = RuleId("simple-test:stop-traversal");

fn format(
    code: &str,
    stop_first: bool,
    stop_before: NodePredicate,
    stop_after: NodePredicate,
) -> Vec<RuleExecutionCall> {
    let calls: Calls = Arc::new(Mutex::new(Vec::new()));
    KtLintRuleEngine::new(vec![SimpleTestRule::provider(
        &calls,
        RULE_ID_STOP_TRAVERSAL,
        stop_first,
        stop_before,
        stop_after,
    )])
    .format(&Code::from_snippet(code, false), &mut |_| {
        AutocorrectDecision::AllowAutocorrect
    })
    .unwrap();
    calls.lock().unwrap().clone()
}

/// `filteredOn { it.elementType == null || it.classIdentifier != null }`.
fn hooks_and_classes(calls: Vec<RuleExecutionCall>) -> Vec<RuleExecutionCall> {
    calls
        .into_iter()
        .filter(|c| c.element_type.is_none() || c.class_identifier.is_some())
        .collect()
}

fn class_call(method: engine_common::RuleMethod, name: &str) -> RuleExecutionCall {
    RuleExecutionCall::node(RULE_ID_STOP_TRAVERSAL, method, Child, CLASS, Some(name))
}

#[test]
fn given_that_the_traversal_is_stopped_in_the_before_first_node_hook_then_do_not_traverse_but_call_after_last_node()
 {
    assert_eq!(
        format("class Foo", true, never, never),
        [
            RuleExecutionCall::new(RULE_ID_STOP_TRAVERSAL, BeforeFirst),
            RuleExecutionCall::new(RULE_ID_STOP_TRAVERSAL, AfterLast)
        ]
    );
}

#[test]
fn given_that_the_traversal_is_stopped_in_before_visit_child_nodes_at_class_foo_then_inside_foo_and_after_foo_are_not_traversed()
 {
    let code = "class FooBar {\n    class Foo {\n        class InsideFoo // Won't be visited as traversal is stopped when entering class Foo\n    \
                }\n\n    class AfterFoo // Won't be visited as traversal is stopped when entering class Foo\n}";
    assert_eq!(
        hooks_and_classes(format(code, false, is_class_foo, never)),
        [
            RuleExecutionCall::new(RULE_ID_STOP_TRAVERSAL, BeforeFirst),
            class_call(BeforeChildren, "FooBar"),
            class_call(BeforeChildren, "Foo"),
            class_call(AfterChildren, "Foo"),
            class_call(AfterChildren, "FooBar"),
            RuleExecutionCall::new(RULE_ID_STOP_TRAVERSAL, AfterLast),
        ]
    );
}

#[test]
fn given_that_the_traversal_is_stopped_in_after_visit_child_nodes_at_class_foo_then_after_foo_is_not_traversed()
 {
    let code = "class FooBar {\n    class Foo {\n        class InsideFoo // Is visited as traversal is stopped when after leaving class Foo\n    \
                }\n\n    class AfterFoo // Won't be visited as traversal is stopped when entering class Foo\n}";
    assert_eq!(
        hooks_and_classes(format(code, false, never, is_class_foo)),
        [
            RuleExecutionCall::new(RULE_ID_STOP_TRAVERSAL, BeforeFirst),
            class_call(BeforeChildren, "FooBar"),
            class_call(BeforeChildren, "Foo"),
            class_call(BeforeChildren, "InsideFoo"),
            class_call(AfterChildren, "InsideFoo"),
            class_call(AfterChildren, "Foo"),
            class_call(AfterChildren, "FooBar"),
            RuleExecutionCall::new(RULE_ID_STOP_TRAVERSAL, AfterLast),
        ]
    );
}

/// Upstream builds this case with `stopTraversalInBeforeFirstNode = true` too (a copy-paste), so the
/// expectation is the same as the first case.
#[test]
fn given_that_the_traversal_is_stopped_in_the_after_last_node_hook_then_do_nothing_special() {
    assert_eq!(
        format("class Foo", true, never, never),
        [
            RuleExecutionCall::new(RULE_ID_STOP_TRAVERSAL, BeforeFirst),
            RuleExecutionCall::new(RULE_ID_STOP_TRAVERSAL, AfterLast)
        ]
    );
}
