//! 2.0's `generateEditorConfig` with a ktlint 1.x (`RuleSetProviderV3`) rule set loaded: the backward compatibility
//! layer (`Rule.toRuleV2`) wraps every `EditorConfigProperty` per rule instance, so a property that two rules share
//! has distinct identities and `EditorConfig.requireSingularIdentities` throws. The only such rule set ktrs runs
//! natively is compose-rules (`ktrs_compose`).

use ktrs_lint::editorconfig::CodeStyleValue;
use ktrs_lint::rule_provider::RuleV2Provider;

use crate::ktlint::command_line::Exit;

const LEGACY_RULE_SET: &str = "compose";

/// `requireSingularIdentities` over the properties of the rules in provider order. The JVM prints lambda identities
/// (`$$Lambda/0x…@…`) that differ per run; ktrs prints made-up ones of the same shape.
pub fn require_singular_identities(rule_providers: &[RuleV2Provider]) -> Result<(), Exit> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for property in rule_providers
        .iter()
        .filter(|p| p.rule_id().rule_set_id().value() == LEGACY_RULE_SET)
        .flat_map(|p| p.uses_editor_config_properties().iter())
    {
        let name = property.name().to_owned();
        let instance = groups.iter().map(|(_, g)| g.len()).sum::<usize>();
        let rendered = format!(
            "  - EditorConfigProperty(type={}, defaultValue={}, ktlintOfficialCodeStyleDefaultValue={}, \
             intellijIdeaCodeStyleDefaultValue={}, androidStudioCodeStyleDefaultValue={}, \
             propertyMapper=com.pinterest.ktlint.rule.engine.core.api.RuleKt$$Lambda/0x0000000000000000@{:08x}, \
             propertyWriter=com.pinterest.ktlint.rule.engine.core.api.editorconfig.EditorConfigProperty$$Lambda/0x0000000000000000@0, \
             deprecationWarning=null, deprecationError=null, name={name})",
            property.property_type().name(),
            property.write_default_value(CodeStyleValue::KtlintOfficial),
            property.write_default_value(CodeStyleValue::KtlintOfficial),
            property.write_default_value(CodeStyleValue::IntellijIdea),
            property.write_default_value(CodeStyleValue::AndroidStudio),
            instance + 1,
        );
        match groups.iter_mut().find(|(n, _)| *n == name) {
            Some((_, group)) => group.push(rendered),
            None => groups.push((name, vec![rendered])),
        }
    }
    let message = groups
        .iter()
        .filter(|(_, group)| group.len() > 1)
        .map(|(name, group)| {
            format!("Found multiple editorconfig properties with name '{name}' but having distinct identities:\n{}", group.join("\n"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    if message.is_empty() { Ok(()) } else { Err(Exit::Crash(format!("java.lang.IllegalArgumentException: {message}"))) }
}
