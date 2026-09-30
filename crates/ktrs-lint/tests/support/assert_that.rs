//! Port of the parts of ktlint-test `KtLintAssertThat.kt` the engine tests use: the rule under test plus
//! additional rules, `ktlint_experimental` and every loaded rule set enabled by override, lint violations
//! of the rule under test only (distinct, any order), and format checks that re-lint the result.

use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use ktrs_lint::editorconfig::{
    EXPERIMENTAL_RULES_EXECUTION_PROPERTY, PropertyRef, RuleExecution,
    create_rule_set_execution_editor_config_property,
};
use ktrs_lint::{
    AutocorrectDecision, Code, EditorConfigDefaults, EditorConfigOverride, KtLintRuleEngine,
    LintError, RuleId, RuleV2, RuleV2Provider,
};

/// `DummyRule(id)`: a rule that does nothing, so its id counts as loaded.
pub struct DummyRule(pub &'static str);

impl RuleV2 for DummyRule {
    fn rule_id(&self) -> RuleId {
        RuleId(self.0)
    }
}

/// `LintViolation(line, col, detail, canBeAutoCorrected)`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LintViolation {
    pub line: usize,
    pub col: usize,
    pub detail: String,
    pub can_be_auto_corrected: bool,
}

/// `LintViolation(line, col, detail)` (autocorrectable).
pub fn v(line: usize, col: usize, detail: &str) -> LintViolation {
    LintViolation {
        line,
        col,
        detail: detail.to_owned(),
        can_be_auto_corrected: true,
    }
}

/// `LintViolation(line, col, detail, false)`.
pub fn v_manual(line: usize, col: usize, detail: &str) -> LintViolation {
    LintViolation {
        line,
        col,
        detail: detail.to_owned(),
        can_be_auto_corrected: false,
    }
}

type Provider = Arc<dyn Fn() -> Box<dyn RuleV2> + Send + Sync>;

/// `assertThatRuleBuilder { rule }` ... `.assertThat()`.
#[derive(Clone)]
pub struct AssertThatBuilder {
    rule: Provider,
    additional: Vec<Provider>,
}

pub fn assert_that_rule_builder(
    rule: impl Fn() -> Box<dyn RuleV2> + Send + Sync + 'static,
) -> AssertThatBuilder {
    AssertThatBuilder {
        rule: Arc::new(rule),
        additional: Vec::new(),
    }
}

impl AssertThatBuilder {
    pub fn add_additional_rule_provider(
        mut self,
        provider: impl Fn() -> Box<dyn RuleV2> + Send + Sync + 'static,
    ) -> Self {
        self.additional.push(Arc::new(provider));
        self
    }

    /// The `(code) -> KtLintAssertThat` lambda.
    pub fn code(&self, code: &str) -> AssertThat {
        AssertThat {
            builder: self.clone(),
            code: code.to_owned(),
            script: false,
        }
    }
}

pub struct AssertThat {
    builder: AssertThatBuilder,
    code: String,
    script: bool,
}

fn provider(p: &Provider) -> RuleV2Provider {
    let p = p.clone();
    RuleV2Provider::new(move || p())
}

impl AssertThat {
    pub fn as_kotlin_script(mut self) -> Self {
        self.script = true;
        self
    }

    fn rule_id(&self) -> RuleId {
        (self.builder.rule)().rule_id()
    }

    /// `createKtLintRuleEngine()`.
    fn engine(&self) -> KtLintRuleEngine {
        let mut rule_providers = vec![provider(&self.builder.rule)];
        rule_providers.extend(self.builder.additional.iter().map(provider));
        let mut editor_config_override = EditorConfigOverride::empty().plus(vec![(
            PropertyRef::from(&*EXPERIMENTAL_RULES_EXECUTION_PROPERTY),
            Some("enabled".to_owned()),
        )]);
        let mut rule_sets: Vec<&'static str> = Vec::new();
        for p in &rule_providers {
            let set = p.rule_id().rule_set_id().value();
            if !rule_sets.contains(&set) {
                rule_sets.push(set);
            }
        }
        let rule_set_executions: Vec<(PropertyRef, Option<String>)> = rule_sets
            .into_iter()
            .map(|set| {
                PropertyRef::from(create_rule_set_execution_editor_config_property(
                    set,
                    RuleExecution::Enabled,
                ))
            })
            .filter(|p| editor_config_override.get(p).is_none())
            .map(|p| (p, Some("enabled".to_owned())))
            .collect();
        if !rule_set_executions.is_empty() {
            editor_config_override = editor_config_override.plus(rule_set_executions);
        }
        KtLintRuleEngine::with_editor_config(
            rule_providers,
            EditorConfigDefaults::empty(),
            editor_config_override,
        )
    }

    fn snippet(&self) -> Code {
        Code::from_snippet(&self.code, self.script)
    }

    fn lint(&self) -> HashSet<LintError> {
        let mut errors = HashSet::new();
        self.engine()
            .lint(&self.snippet(), &mut |e| {
                errors.insert(e.clone());
            })
            .unwrap_or_else(|e| panic!("lint failed: {e}"));
        errors
    }

    fn format(&self) -> (String, Vec<LintError>) {
        let mut errors = Vec::new();
        let formatted = self
            .engine()
            .format(&self.snippet(), &mut |e| {
                errors.push(e.clone());
                AutocorrectDecision::AllowAutocorrect
            })
            .unwrap_or_else(|e| panic!("format failed: {e}"));
        (formatted, errors)
    }

    fn current_rule_violations(&self) -> BTreeSet<LintViolation> {
        let rule_id = self.rule_id();
        self.lint()
            .into_iter()
            .filter(|e| e.rule_id == rule_id)
            .map(|e| LintViolation {
                line: e.line,
                col: e.col,
                detail: e.detail,
                can_be_auto_corrected: e.can_be_auto_corrected,
            })
            .collect()
    }

    pub fn has_lint_violation(self, line: usize, col: usize, detail: &str) -> Self {
        self.has_lint_violations(&[v(line, col, detail)])
    }

    pub fn has_lint_violations(self, expected: &[LintViolation]) -> Self {
        assert!(!expected.is_empty());
        let expected: BTreeSet<LintViolation> = expected.iter().cloned().collect();
        let actual = self.current_rule_violations();
        assert_eq!(
            actual,
            expected,
            "lint violations of {}\n--- code\n{}",
            self.rule_id(),
            self.code
        );
        self
    }

    pub fn has_no_lint_violations(self) {
        assert!(
            self.current_rule_violations().is_empty(),
            "no lint violations expected\n{}",
            self.code
        );
        let (formatted, errors) = self.format();
        assert!(errors.is_empty(), "format found errors: {errors:?}");
        assert_eq!(formatted, self.code, "format changed the code");
        self.relint(&formatted);
    }

    pub fn is_formatted_as(self, formatted_code: &str) -> Self {
        assert_ne!(
            formatted_code, self.code,
            "Use '.hasNoLintViolations()' instead of '.isFormattedAs(<original code>)'"
        );
        let (formatted, _) = self.format();
        assert_eq!(
            formatted, formatted_code,
            "formatted code\n--- input\n{}",
            self.code
        );
        self.relint(&formatted);
        self
    }

    /// "After reformat of code, it can no longer be successfully parsed".
    fn relint(&self, formatted: &str) {
        self.engine()
            .lint(&Code::from_snippet(formatted, self.script), &mut |_| {})
            .unwrap_or_else(|e| panic!("formatted code no longer lints: {e}\n{formatted}"));
    }
}

/// Kotlin `trimIndent()`, so the test sources can keep the upstream raw-string layout.
pub fn trim_indent(s: &str) -> String {
    let lines: Vec<&str> = s.split('\n').collect();
    let indent_width = |l: &str| l.chars().take_while(|c| c.is_whitespace()).count();
    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| indent_width(l))
        .min()
        .unwrap_or(0);
    let last = lines.len() - 1;
    lines
        .iter()
        .enumerate()
        .filter(|&(i, l)| !((i == 0 || i == last) && l.trim().is_empty()))
        .map(|(_, l)| l.chars().skip(min_indent).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
