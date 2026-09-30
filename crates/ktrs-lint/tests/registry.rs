use std::collections::BTreeSet;

use ktrs_lint::rules::standard_rule_providers;

// StandardRuleSetProvider in ktlint 2.0.0-ALPHA-4 registers 105 rules; a duplicate would run a rule twice per file.
#[test]
fn standard_rules_are_registered_once_each() {
    let ids: Vec<String> = standard_rule_providers().iter().map(|p| p.rule_id().value().to_owned()).collect();
    let unique: BTreeSet<&String> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len(), "duplicate rule ids registered");
    assert_eq!(ids.len(), 105);
}
