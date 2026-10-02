//! Ports of ktlint 1.8.0's `internal/rulefilter/RunAfterRuleFilter.kt` and `internal/RuleProviderSorter.kt`
//! (both removed in 2.0, #3252): the rules' `VisitorModifier`s decide which rules load and in which order
//! they run, one after another.

use crate::engine::internal_rules::KTLINT_SUPPRESSION_RULE_ID;
use crate::rule::{RuleId, RuleSetId, RunAfterRuleMode};
use crate::rule_provider::RuleV2Provider;

/// `RunAfterRuleFilter.RunAfterRuleOrderModifier`, by severity.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RunAfterRuleOrderModifier {
    Add,
    Ignore,
    BlockUntilRunAfterRuleIsLoaded,
    RequiredRunAfterRuleNotLoaded,
}

/// `RunAfterRuleFilter().filter(ruleProviders)`. `Err` is the message of the `IllegalStateException` that 1.8
/// throws (out of `lint`/`format`) when a required rule is missing or the dependencies are cyclic.
pub fn run_after_rule_filter(rule_providers: Vec<RuleV2Provider>) -> Result<Vec<RuleV2Provider>, String> {
    let mut filter = RunAfterRuleFilter::default();
    filter.unprocessed_rule_providers = rule_providers.clone();
    let (loadable, unprocessed): (Vec<_>, Vec<_>) =
        filter.unprocessed_rule_providers.drain(..).partition(|p| p.run_after_rules().next().is_none());
    filter.loadable_rule_providers = loadable;
    filter.unprocessed_rule_providers = unprocessed;
    let mut rule_providers_iterator = rule_providers;
    rule_providers_iterator.sort_by_key(|p| p.rule_id().value());
    loop {
        let new_rule_providers_added = filter.filter(rule_providers_iterator);
        if !new_rule_providers_added {
            break;
        }
        rule_providers_iterator = can_run_with(&filter.blocked_rule_providers, &ids(&filter.loadable_rule_providers));
        filter.blocked_rule_providers.clear();
    }
    if !filter.required_but_missing_rule_ids.is_empty() {
        return Err(filter.create_required_rule_is_missing_message());
    }
    if !filter.blocked_rule_providers.is_empty() {
        return Err(filter.create_cyclic_dependency_message());
    }
    if !filter.unprocessed_rule_providers.is_empty() {
        return Err("Check failed.".to_owned());
    }
    Ok(filter.loadable_rule_providers)
}

#[derive(Default)]
struct RunAfterRuleFilter {
    unprocessed_rule_providers: Vec<RuleV2Provider>,
    loadable_rule_providers: Vec<RuleV2Provider>,
    blocked_rule_providers: Vec<RuleV2Provider>,
    /// `(ruleId, runAfterRuleId)`, a set in insertion order.
    required_but_missing_rule_ids: Vec<(RuleId, RuleId)>,
}

impl RunAfterRuleFilter {
    /// `Iterator<RuleProvider>.filter()`: whether a provider was added to the loadable ones.
    fn filter(&mut self, rule_providers: Vec<RuleV2Provider>) -> bool {
        let mut new_rule_providers_added = false;
        for current_rule_provider in rule_providers {
            match self.max_run_after_rule_order_modifiers(&current_rule_provider) {
                RunAfterRuleOrderModifier::Add | RunAfterRuleOrderModifier::Ignore => {
                    self.unprocessed_rule_providers.retain(|p| p.rule_id() != current_rule_provider.rule_id());
                    add_to_set(&mut self.loadable_rule_providers, current_rule_provider);
                    new_rule_providers_added = true;
                }
                RunAfterRuleOrderModifier::BlockUntilRunAfterRuleIsLoaded => {
                    add_to_set(&mut self.blocked_rule_providers, current_rule_provider);
                }
                RunAfterRuleOrderModifier::RequiredRunAfterRuleNotLoaded => {}
            }
        }
        new_rule_providers_added
    }

    /// Every run-after rule is classified (missing required ones are recorded), then the most severe wins.
    fn max_run_after_rule_order_modifiers(&mut self, rule_provider: &RuleV2Provider) -> RunAfterRuleOrderModifier {
        let mut max = RunAfterRuleOrderModifier::Add;
        for (run_after_rule_id, mode) in rule_provider.run_after_rules() {
            let modifier = if ids(&self.loadable_rule_providers).contains(&run_after_rule_id) {
                RunAfterRuleOrderModifier::Add
            } else if ids(&self.unprocessed_rule_providers).contains(&run_after_rule_id) {
                RunAfterRuleOrderModifier::BlockUntilRunAfterRuleIsLoaded
            } else if mode == RunAfterRuleMode::OnlyWhenRunAfterRuleIsLoadedAndEnabled {
                let missing = (rule_provider.rule_id(), run_after_rule_id);
                if !self.required_but_missing_rule_ids.contains(&missing) {
                    self.required_but_missing_rule_ids.push(missing);
                }
                RunAfterRuleOrderModifier::RequiredRunAfterRuleNotLoaded
            } else {
                RunAfterRuleOrderModifier::Ignore
            };
            max = max.max(modifier);
        }
        max
    }

    /// `RuleId` is a Kotlin data class here, so its `toString` shows.
    fn create_required_rule_is_missing_message(&self) -> String {
        let mut message = "Skipping rule(s) which are depending on a rule which is not loaded. Please check if you need to add \
                           additional rule sets before creating an issue."
            .to_owned();
        for (rule_id, run_after_rule_id) in &self.required_but_missing_rule_ids {
            message.push_str(&format!(
                "\n  - Rule with id 'RuleId(value={rule_id})' requires rule with id 'RuleId(value={run_after_rule_id})' to be loaded"
            ));
        }
        message
    }

    fn create_cyclic_dependency_message(&self) -> String {
        let mut custom_rule_set_ids: Vec<&str> = self
            .blocked_rule_providers
            .iter()
            .map(|p| p.rule_id().rule_set_id())
            .filter(|it| *it != RuleSetId::STANDARD)
            .map(RuleSetId::value)
            .collect();
        custom_rule_set_ids.sort_unstable();
        custom_rule_set_ids.dedup();
        let mut message = if custom_rule_set_ids.is_empty() {
            "Found cyclic dependencies between required rules that should run after another rule:".to_owned()
        } else {
            format!(
                "Found cyclic dependencies between required rules that should run after another rule. Please contact the \
                 maintainer(s) of the custom rule set(s) [{}] before creating an issue in the KtLint project. Dependencies:",
                custom_rule_set_ids.join(", ")
            )
        };
        for p in &self.blocked_rule_providers {
            let run_after: Vec<&str> = p.run_after_rules().map(|(id, _)| id.value()).collect();
            message.push_str(&format!(
                "\n  - Rule with id '{}' should run after rule(s) with id '{}'",
                p.rule_id().value(),
                run_after.join(", ")
            ));
        }
        message
    }
}

/// `Set<RuleProvider>.canRunWithRuleIds(loadedRuleIds)`: the blocked providers that can run once the loaded
/// ones (and, transitively, the unblocked ones) are in.
fn can_run_with(rule_providers: &[RuleV2Provider], loaded_rule_ids: &[RuleId]) -> Vec<RuleV2Provider> {
    let unblocked: Vec<RuleV2Provider> = rule_providers
        .iter()
        .filter(|p| {
            p.run_after_rules().all(|(id, mode)| {
                loaded_rule_ids.contains(&id) || mode == RunAfterRuleMode::RegardlessWhetherRunAfterRuleIsLoadedOrDisabled
            })
        })
        .cloned()
        .collect();
    if unblocked.is_empty() {
        return unblocked;
    }
    let unblocked_rule_ids = ids(&unblocked);
    let still_blocked: Vec<RuleV2Provider> =
        rule_providers.iter().filter(|p| !unblocked_rule_ids.contains(&p.rule_id())).cloned().collect();
    let mut loaded = loaded_rule_ids.to_vec();
    loaded.extend(unblocked_rule_ids);
    let mut result = can_run_with(&still_blocked, &loaded);
    for p in unblocked {
        add_to_set(&mut result, p);
    }
    result
}

fn ids(rule_providers: &[RuleV2Provider]) -> Vec<RuleId> {
    rule_providers.iter().map(RuleV2Provider::rule_id).collect()
}

fn add_to_set(set: &mut Vec<RuleV2Provider>, rule_provider: RuleV2Provider) {
    if !set.iter().any(|p| p.rule_id() == rule_provider.rule_id()) {
        set.push(rule_provider);
    }
}

/// `RuleProviderSorter.getSortedRuleProviders`: the default order (suppression rule, then the rules that need
/// not run late, standard rules first, by id), then each rule with run-after rules once those have been placed.
pub fn get_sorted_rule_providers_1_8(rule_providers: &[RuleV2Provider]) -> Vec<RuleV2Provider> {
    for p in rule_providers {
        assert!(
            p.run_after_rules().all(|(id, _)| id != p.rule_id()),
            "Rule with id '{}' has a visitor modifier of type 'RunAfterRule' which may not refer to the rule itself.",
            p.rule_id().value()
        );
    }
    let rule_ids_to_be_sorted = ids(rule_providers);
    let mut unprocessed_rule_providers = rule_providers.to_vec();
    unprocessed_rule_providers.sort_by_key(|p| {
        (
            p.rule_id() != KTLINT_SUPPRESSION_RULE_ID,
            p.run_as_late_as_possible(),
            p.rule_id().rule_set_id() != RuleSetId::STANDARD,
            p.rule_id().value(),
        )
    });
    let (mut sorted_rule_providers, mut unprocessed_rule_providers): (Vec<_>, Vec<_>) = unprocessed_rule_providers
        .into_iter()
        .partition(|p| !p.run_as_late_as_possible() && p.run_after_rules().next().is_none());
    while !unprocessed_rule_providers.is_empty() {
        let sorted_ids = ids(&sorted_rule_providers);
        let index = unprocessed_rule_providers
            .iter()
            .position(|p| {
                p.run_after_rules()
                    .filter(|(id, _)| rule_ids_to_be_sorted.contains(id))
                    .all(|(id, _)| sorted_ids.contains(&id))
            })
            .expect("Can not complete sorting of rule providers as next item can not be determined.");
        sorted_rule_providers.push(unprocessed_rule_providers.remove(index));
    }
    sorted_rule_providers
}
