//! Port of ktlint-rule-engine-core's `Rule.kt` (`RuleV2`, `RuleId`) and `AutocorrectDecision.kt`, plus the
//! slice of `EditorConfig` the ported rules read.

use ktrs_ast::{Ast, NodeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuleId(pub &'static str);

impl RuleId {
    pub fn value(self) -> &'static str {
        self.0
    }

    /// `ruleSetId`: the part before `:` ("" without one).
    pub fn rule_set_id(self) -> &'static str {
        self.0.split_once(':').map_or("", |(set, _)| set)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutocorrectDecision {
    AllowAutocorrect,
    NoAutocorrect,
}

impl AutocorrectDecision {
    /// `ifAutocorrectAllowed { }`.
    pub fn if_autocorrect_allowed<T>(self, function: impl FnOnce() -> T) -> Option<T> {
        (self == AutocorrectDecision::AllowAutocorrect).then(function)
    }
}

/// The `emit(offset, errorMessage, canBeAutoCorrected)` lambda. It also takes the tree, because the
/// offset (UTF-8, in the current tree) is converted to UTF-16 at emit time.
pub type Emit<'a> = dyn FnMut(&Ast, usize, &str, bool) -> AutocorrectDecision + 'a;

/// `RuleV2`. A new instance runs every pass (`RuleV2Provider.createNewRuleInstance`).
// TODO: stopTraversalOfAST (no ported rule calls it; the engine already filters on the state).
pub trait RuleV2 {
    fn rule_id(&self) -> RuleId;

    fn before_first_node(&mut self, _editor_config: &EditorConfig) {}

    fn before_visit_child_nodes(&mut self, _ast: &mut Ast, _node: NodeId, _emit: &mut Emit<'_>) {}

    fn after_visit_child_nodes(&mut self, _ast: &mut Ast, _node: NodeId, _emit: &mut Emit<'_>) {}

    fn after_last_node(&mut self) {}
}

pub type RuleV2Provider = fn() -> Box<dyn RuleV2>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndentStyle {
    Space,
    Tab,
}

/// The `.editorconfig` properties the ported rules use, at their `ktlint_official` defaults.
// TODO: EditorConfigLoader (.editorconfig files, ktlint_<set>_<rule> execution properties).
#[derive(Clone, Debug)]
pub struct EditorConfig {
    pub indent_style: IndentStyle,
    pub indent_size: i32,
}

impl Default for EditorConfig {
    fn default() -> EditorConfig {
        EditorConfig { indent_style: IndentStyle::Space, indent_size: 4 }
    }
}
