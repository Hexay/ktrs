//! `detekt-api/.../Rule.kt`, `RuleSet.kt`, `RuleSetProvider.kt`, `internal/Validation.kt`.

use std::fmt;
use std::sync::Arc;

use ktrs_psi::KtFile;

use super::config::{self, AUTO_CORRECT_KEY, Config, Value};
use super::entity::Finding;
use crate::visitor::DetektVisitor;

/// The state every `Rule` has: its config, description and the findings of the current file.
pub struct RuleBase {
    pub config: Arc<dyn Config>,
    pub description: &'static str,
    findings: Vec<Finding>,
}

impl RuleBase {
    pub fn new(config: Arc<dyn Config>, description: &'static str) -> RuleBase {
        RuleBase { config, description, findings: Vec::new() }
    }
}

/// A rule defines how one specific code structure should look like. It is a `DetektVisitor` started with
/// `visit(KtFile)`; `pre_visit`/`post_visit` run before and after.
pub trait Rule: DetektVisitor {
    fn base(&self) -> &RuleBase;

    fn base_mut(&mut self) -> &mut RuleBase;

    /// `ruleName`: the class's simple name.
    fn rule_name(&self) -> RuleName;

    fn config(&self) -> &Arc<dyn Config> {
        &self.base().config
    }

    fn description(&self) -> &'static str {
        self.base().description
    }

    fn auto_correct(&self) -> bool {
        let own = self.config().value_or_default(AUTO_CORRECT_KEY, Value::Boolean(false));
        let parent = self.config().parent().map(|p| p.value_or_default(AUTO_CORRECT_KEY, Value::Boolean(true)));
        own == Value::Boolean(true) && parent != Some(Value::Boolean(false))
    }

    /// `visitFile(root, languageVersionSettings)`; not `visit_file`, which is `PsiElementVisitor.visitFile` here.
    // TODO: languageVersionSettings (read by rules not ported yet: the explicit-API checks)
    fn visit_root_file(&mut self, root: &KtFile) -> Vec<Finding> {
        self.base_mut().findings.clear();
        self.pre_visit(root);
        self.visit(root);
        self.post_visit(root);
        std::mem::take(&mut self.base_mut().findings)
    }

    fn pre_visit(&mut self, _root: &KtFile) {}

    fn visit(&mut self, root: &KtFile) {
        root.accept(self);
    }

    fn post_visit(&mut self, _root: &KtFile) {}

    fn report(&mut self, finding: Finding) {
        self.base_mut().findings.push(finding);
    }
}

/// The three `Rule` methods every rule implements the same way: `rule_base!(RuleName);` inside `impl Rule`.
#[macro_export]
macro_rules! rule_base {
    ($name:ident) => {
        fn base(&self) -> &$crate::api::RuleBase {
            &self.base
        }

        fn base_mut(&mut self) -> &mut $crate::api::RuleBase {
            &mut self.base
        }

        fn rule_name(&self) -> $crate::api::RuleName {
            $crate::api::RuleName::new(stringify!($name))
        }
    };
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RuleName(String);

impl RuleName {
    pub fn new(value: &str) -> RuleName {
        RuleName::try_new(value).unwrap_or_else(|| panic!("id '{value}' must match {IDENTIFIER_REGEX}"))
    }

    pub fn try_new(value: &str) -> Option<RuleName> {
        validate_identifier(value).then(|| RuleName(value.to_owned()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RuleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RuleSetId(String);

impl RuleSetId {
    pub fn new(value: &str) -> RuleSetId {
        assert!(validate_identifier(value), "id '{value}' must match {IDENTIFIER_REGEX}");
        RuleSetId(value.to_owned())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RuleSetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

const IDENTIFIER_REGEX: &str = "[aA-zZ]+(?:[aA-zZ0-9-]+)*[aA-zZ0-9]";

/// `id.matches(identifierRegex)`. Gotcha: `[aA-zZ]` is the ASCII range `A..z`, punctuation between the cases
/// included.
fn validate_identifier(id: &str) -> bool {
    let letter = |c: char| ('A'..='z').contains(&c);
    let chars: Vec<char> = id.chars().collect();
    chars.len() >= 2
        && letter(chars[0])
        && chars.iter().all(|&c| letter(c) || c.is_ascii_digit() || c == '-')
        && chars.last().is_some_and(|&c| letter(c) || c.is_ascii_digit())
}

/// `(Config) -> Rule`.
pub type RuleProvider = fn(Arc<dyn Config>) -> Box<dyn Rule>;

/// `listOf(::RuleA, ::RuleB)`: the constructor references of a rule set, as [`RuleProvider`]s.
#[macro_export]
macro_rules! rule_providers {
    ($($rule:ident),* $(,)?) => {
        &[$({
            fn provider(config: std::sync::Arc<dyn $crate::api::Config>) -> Box<dyn $crate::api::Rule> {
                Box::new($rule::new(config))
            }
            provider as $crate::api::RuleProvider
        }),*]
    };
}

/// A rule set is a collection of rules and must be defined within a rule set provider implementation.
pub struct RuleSet {
    pub id: RuleSetId,
    pub rules: Vec<(RuleName, RuleProvider)>,
}

impl RuleSet {
    /// `RuleSet(id, rules)`: each rule's name is read from an instance built on `Config.empty`.
    pub fn new(id: RuleSetId, rules: &[RuleProvider]) -> RuleSet {
        RuleSet { id, rules: rules.iter().map(|&provider| (provider(config::empty()).rule_name(), provider)).collect() }
    }

    pub fn rule(&self, name: &RuleName) -> Option<RuleProvider> {
        self.rules.iter().find(|(n, _)| n == name).map(|(_, provider)| *provider)
    }
}

/// `RuleSetProvider`; `is_default` is `this is DefaultRuleSetProvider`.
pub struct RuleSetProvider {
    pub rule_set_id: RuleSetId,
    pub instance: fn() -> RuleSet,
    pub is_default: bool,
}
