//! Test rules shared by the `engine_*` tests (ports of the private rules of ktlint-rule-engine's tests).
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use ktrs_ast::{Ast, NodeId};
use ktrs_lint::editorconfig::{RuleExecution, create_rule_execution_editor_config_property};
use ktrs_lint::engine::internal_rules::KTLINT_SUPPRESSION_RULE_ID;
use ktrs_lint::{
    Code, EditorConfig, EditorConfigOverride, Emit, KtLintRuleEngine, LintError, RuleId, RuleV2,
    RuleV2Provider, TraversalState,
};
use ktrs_syntax::SyntaxKind::{self, CLASS, FILE, IDENTIFIER};

pub const STANDARD_NO_FOO_IDENTIFIER_RULE_ID: RuleId =
    RuleId("standard:no-foo-identifier-standard");
pub const NON_STANDARD_NO_FOO_IDENTIFIER_RULE_ID: RuleId = RuleId("custom:no-foo-identifier");
pub const NO_FOO_MESSAGE: &str = "Line should not contain a foo identifier";

/// `SuppressionLocatorTest.NoFooIdentifierRule`.
pub struct NoFooIdentifierRule(pub RuleId);

impl RuleV2 for NoFooIdentifierRule {
    fn rule_id(&self) -> RuleId {
        self.0
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, emit: &mut Emit<'_>) {
        if ast.element_type(node) == IDENTIFIER && ast.text(node).starts_with("foo") {
            emit(ast, ast.start_offset(node), NO_FOO_MESSAGE, false);
        }
    }
}

/// `SuppressionLocatorTest.lint(code, editorConfigOverride, ruleProviders, ignoreKtlintSuppressionRule)`.
pub fn lint_no_foo(
    code: &str,
    editor_config_override: EditorConfigOverride,
    rule_providers: Vec<RuleV2Provider>,
    ignore_ktlint_suppression_rule: bool,
) -> Vec<LintError> {
    let mut providers = vec![
        RuleV2Provider::new(|| {
            Box::new(NoFooIdentifierRule(STANDARD_NO_FOO_IDENTIFIER_RULE_ID)) as Box<dyn RuleV2>
        }),
        RuleV2Provider::new(|| {
            Box::new(NoFooIdentifierRule(NON_STANDARD_NO_FOO_IDENTIFIER_RULE_ID)) as Box<dyn RuleV2>
        }),
    ];
    providers.extend(rule_providers);
    let editor_config_override = editor_config_override.plus(vec![
        (
            create_rule_execution_editor_config_property(
                STANDARD_NO_FOO_IDENTIFIER_RULE_ID.value(),
                RuleExecution::Enabled,
            )
            .into(),
            Some("enabled".into()),
        ),
        (
            create_rule_execution_editor_config_property(
                NON_STANDARD_NO_FOO_IDENTIFIER_RULE_ID.value(),
                RuleExecution::Enabled,
            )
            .into(),
            Some("enabled".into()),
        ),
    ]);
    let engine =
        KtLintRuleEngine::with_editor_config(providers, Default::default(), editor_config_override);
    let mut errors = Vec::new();
    engine
        .lint(&Code::from_snippet(code, false), &mut |e| {
            if !(ignore_ktlint_suppression_rule && e.rule_id == KTLINT_SUPPRESSION_RULE_ID) {
                errors.push(e.clone());
            }
        })
        .unwrap();
    errors
}

pub fn no_foo_error(line: usize, col: usize, rule_id: &'static str) -> LintError {
    LintError {
        line,
        col,
        rule_id: RuleId(rule_id),
        detail: NO_FOO_MESSAGE.to_owned(),
        can_be_auto_corrected: false,
    }
}

/// Both no-foo rules at the same position, in the order ktlint reports them (standard rule set first).
pub fn no_foo_errors(line: usize, col: usize) -> [LintError; 2] {
    [
        no_foo_error(line, col, "standard:no-foo-identifier-standard"),
        no_foo_error(line, col, "custom:no-foo-identifier"),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleMethod {
    BeforeFirst,
    BeforeChildren,
    AfterChildren,
    AfterLast,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisitNodeType {
    Root,
    Child,
}

/// `KtLintTest.RuleExecutionCall`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleExecutionCall {
    pub rule_id: RuleId,
    pub rule_method: RuleMethod,
    pub visit_node_type: Option<VisitNodeType>,
    pub element_type: Option<SyntaxKind>,
    pub class_identifier: Option<String>,
}

impl RuleExecutionCall {
    pub fn new(rule_id: RuleId, rule_method: RuleMethod) -> RuleExecutionCall {
        RuleExecutionCall {
            rule_id,
            rule_method,
            visit_node_type: None,
            element_type: None,
            class_identifier: None,
        }
    }

    pub fn node(
        rule_id: RuleId,
        rule_method: RuleMethod,
        v: VisitNodeType,
        t: SyntaxKind,
        class: Option<&str>,
    ) -> RuleExecutionCall {
        RuleExecutionCall {
            rule_id,
            rule_method,
            visit_node_type: Some(v),
            element_type: Some(t),
            class_identifier: class.map(str::to_owned),
        }
    }
}

pub type Calls = Arc<Mutex<Vec<RuleExecutionCall>>>;
pub type NodePredicate = fn(&Ast, NodeId) -> bool;

/// `KtLintTest.SimpleTestRule`: records every hook call.
pub struct SimpleTestRule {
    pub rule_execution_calls: Calls,
    pub rule_id: RuleId,
    pub stop_traversal_in_before_first_node: bool,
    pub stop_traversal_in_before_visit_child_nodes: NodePredicate,
    pub stop_traversal_in_after_visit_child_nodes: NodePredicate,
    pub traversal: TraversalState,
}

impl SimpleTestRule {
    pub fn provider(
        calls: &Calls,
        rule_id: RuleId,
        stop_first: bool,
        stop_before: NodePredicate,
        stop_after: NodePredicate,
    ) -> RuleV2Provider {
        let calls = calls.clone();
        RuleV2Provider::new(move || {
            Box::new(SimpleTestRule {
                rule_execution_calls: calls.clone(),
                rule_id,
                stop_traversal_in_before_first_node: stop_first,
                stop_traversal_in_before_visit_child_nodes: stop_before,
                stop_traversal_in_after_visit_child_nodes: stop_after,
                traversal: TraversalState::default(),
            }) as Box<dyn RuleV2>
        })
    }

    fn to_rule_execution_call(
        &self,
        ast: &Ast,
        node: NodeId,
        rule_method: RuleMethod,
    ) -> RuleExecutionCall {
        let element_type = ast.element_type(node);
        let class_identifier = if element_type == CLASS {
            ast.find_child_by_type(node, IDENTIFIER)
                .map(|it| ast.text(it))
        } else {
            None
        };
        let visit_node_type = if element_type == FILE {
            VisitNodeType::Root
        } else {
            VisitNodeType::Child
        };
        RuleExecutionCall {
            rule_id: self.rule_id,
            rule_method,
            visit_node_type: Some(visit_node_type),
            element_type: Some(element_type),
            class_identifier,
        }
    }

    fn record(&self, call: RuleExecutionCall) {
        self.rule_execution_calls.lock().unwrap().push(call);
    }
}

pub fn never(_: &Ast, _: NodeId) -> bool {
    false
}

/// `node.elementType == CLASS && node.findChildByType(IDENTIFIER)?.text == "Foo"`.
pub fn is_class_foo(ast: &Ast, node: NodeId) -> bool {
    ast.element_type(node) == CLASS
        && ast
            .find_child_by_type(node, IDENTIFIER)
            .is_some_and(|it| ast.text(it) == "Foo")
}

impl RuleV2 for SimpleTestRule {
    fn rule_id(&self) -> RuleId {
        self.rule_id
    }

    fn traversal_state(&self) -> Option<TraversalState> {
        Some(self.traversal)
    }

    fn before_first_node(&mut self, _editor_config: &EditorConfig) {
        self.record(RuleExecutionCall::new(
            self.rule_id,
            RuleMethod::BeforeFirst,
        ));
        if self.stop_traversal_in_before_first_node {
            self.traversal.stop_traversal_of_ast();
        }
    }

    fn before_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, _emit: &mut Emit<'_>) {
        self.record(self.to_rule_execution_call(ast, node, RuleMethod::BeforeChildren));
        if (self.stop_traversal_in_before_visit_child_nodes)(ast, node) {
            self.traversal.stop_traversal_of_ast();
        }
    }

    fn after_visit_child_nodes(&mut self, ast: &mut Ast, node: NodeId, _emit: &mut Emit<'_>) {
        self.record(self.to_rule_execution_call(ast, node, RuleMethod::AfterChildren));
        if (self.stop_traversal_in_after_visit_child_nodes)(ast, node) {
            self.traversal.stop_traversal_of_ast();
        }
    }

    fn after_last_node(&mut self) {
        self.record(RuleExecutionCall::new(self.rule_id, RuleMethod::AfterLast));
    }
}
