//! `ktrs serve`'s ktlint requests (`tool=ktlint`): format like Spotless's `ktlint()` step does with ktlint's
//! engine (`KtLintCompat1Dot0Dot0Adapter`, research/28-spotless-ktlint-step.md), then report what could not be
//! autocorrected.
//!
//! | key | values | default |
//! |---|---|---|
//! | `ktlint-version` | `1.8`, `2.0` | `1.8` |
//! | `editorconfig-defaults` | an `.editorconfig`-format file: values for properties no `.editorconfig` on `path` sets | none |
//! | `editorconfig-override` | `<property>=<value>`, repeatable: wins over every `.editorconfig` (Spotless's rules, below) | none |
//! | `ruleset` | a rule set JAR, repeatable; only compose-rules releases ktrs ports natively | none |
//! | `path` | the file's path: its `.editorconfig` files and file name rules | none (a snippet) |
//!
//! An ok response has one `violation=<line>\t<col>\t<rule id>\t<detail>` header line per error that was not
//! autocorrected, sorted by position; `\`, tab, line feed and carriage return in the detail are escaped as `\\`,
//! `\t`, `\n` and `\r`. An unparsable file is an error whose message is ktlint's `KtLintParseException` text,
//! `<line>:<col> <message>`.
//!
//! Overrides follow Spotless: names that are neither a rule's property, a standard one, nor `ktlint_<rule set>`
//! or `ktlint_<rule set>_<rule>` are ignored; and unless `ktlint_code_style` is overridden or set by the defaults
//! file, a non-empty override also sets it to `intellij_idea`.

use std::path::Path;
use std::panic::{AssertUnwindSafe, catch_unwind};

use ktrs_editorconfig::EnumValue;
use ktrs_lint::editorconfig::{
    CODE_STYLE_PROPERTY, END_OF_LINE_PROPERTY, EXPERIMENTAL_RULES_EXECUTION_PROPERTY, INDENT_SIZE_PROPERTY, INDENT_STYLE_PROPERTY,
    INSERT_FINAL_NEWLINE_PROPERTY, KtlintVersion, MAX_LINE_LENGTH_PROPERTY, PropertyRef, RuleExecution,
    create_rule_execution_editor_config_property, create_rule_set_execution_editor_config_property,
};
use ktrs_lint::rule_provider::{RuleV2Provider, property_types, rule_providers_in};
use ktrs_lint::rules::standard_rule_providers;
use ktrs_lint::{AutocorrectDecision, Code, EditorConfigDefaults, EditorConfigOverride, KtLintException, KtLintRuleEngine, LintError};

use crate::ktlint::compose_jar::native_compose_rules_release;
use crate::ktlint::version::with_ktlint_version;
use crate::serve::{Fields, Formatted, unknown_key};

/// The server's ktlint state: the engine of the last configuration, reused while requests share it (its rule
/// set JARs are only inspected when it is built).
#[derive(Default)]
pub(crate) struct KtlintRequests {
    engine: Option<(String, KtLintRuleEngine)>,
}

struct Request<'a> {
    version: KtlintVersion,
    defaults: Option<&'a str>,
    overrides: Vec<(&'a str, &'a str)>,
    rulesets: Vec<&'a str>,
    path: Option<&'a str>,
}

impl KtlintRequests {
    pub(crate) fn request(&mut self, fields: &Fields, code: &str) -> Result<Formatted, String> {
        let request = Request::parse(fields)?;
        let key: String = fields.iter().filter(|(k, _)| *k != "path").map(|(k, v)| format!("{k}={v}\n")).collect();
        if self.engine.as_ref().is_none_or(|(cached, _)| *cached != key) {
            self.engine = None;
            self.engine = Some((key, engine(&request)?));
        }
        let engine = &self.engine.as_ref().expect("engine was just built").1;
        let code = match request.path {
            Some(path) => Code::from_file_content(Path::new(path), code.to_owned()),
            None => Code::from_snippet(code, false),
        };
        let mut header = String::new();
        let is_1_8 = request.version.is_1_8();
        // ktlint 2.0 dropped the overload Spotless calls: there, decide like the CLI's `--format`.
        let mut autocorrect = |e: &LintError| {
            if is_1_8 || e.can_be_auto_corrected { AutocorrectDecision::AllowAutocorrect } else { AutocorrectDecision::NoAutocorrect }
        };
        let result = catch_unwind(AssertUnwindSafe(|| {
            engine.format_reporting(&code, &mut autocorrect, &mut |e, corrected| {
                if !corrected {
                    header.push_str(&format!("violation={}\t{}\t{}\t{}\n", e.line, e.col, e.rule_id.value(), escape(&e.detail)));
                }
            })
        }));
        match result {
            Ok(Ok(formatted)) => Ok(Formatted { changed: formatted != code.content, code: formatted, header }),
            Ok(Err(KtLintException::Rule(e))) => Err(format!("{}\nCaused by: {}", e.message, e.cause)),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err(format!("internal error in ktrs (please report it with this file): {}", code.file_path_or_stdin())),
        }
    }
}

impl<'a> Request<'a> {
    fn parse(fields: &Fields<'a>) -> Result<Request<'a>, String> {
        let mut request = Request { version: KtlintVersion::V1_8, defaults: None, overrides: Vec::new(), rulesets: Vec::new(), path: None };
        for &(key, value) in fields {
            match key {
                "tool" => {}
                "ktlint-version" => {
                    request.version = KtlintVersion::value_of(value).ok_or_else(|| format!("ktlint-version must be 1.8 or 2.0, got '{value}'"))?;
                }
                "editorconfig-defaults" => request.defaults = Some(value),
                "editorconfig-override" => {
                    request.overrides.push(value.split_once('=').ok_or_else(|| format!("editorconfig-override needs name=value, got '{value}'"))?);
                }
                "ruleset" => request.rulesets.push(value),
                "path" => request.path = Some(value),
                _ => return Err(unknown_key(key)),
            }
        }
        Ok(request)
    }
}

fn engine(request: &Request) -> Result<KtLintRuleEngine, String> {
    let mut providers = standard_rule_providers();
    for jar in &request.rulesets {
        if !Path::new(jar).is_file() {
            return Err(format!("rule set JAR '{jar}' does not exist"));
        }
        if native_compose_rules_release(Path::new(jar)).is_none() {
            return Err(format!(
                "ktrs runs no rule set JAR natively but compose-rules {}, so not '{jar}': lint with the real ktlint for it \
                 (in Spotless, keep `ktlint()` for this rule set)",
                ktrs_compose::COMPOSE_RULES_VERSION
            ));
        }
        providers.extend(ktrs_compose::compose_rule_providers());
    }
    let defaults = EditorConfigDefaults::load(request.defaults.map(Path::new), &property_types(&providers)).map_err(|e| e.to_string())?;
    let editor_config_override = spotless_editor_config_override(&request.overrides, &defaults, &rule_providers_in(&providers, request.version));
    let editor_config_override = with_ktlint_version(editor_config_override, request.version);
    Ok(KtLintRuleEngine::with_editor_config(providers, defaults, editor_config_override))
}

/// `KtLintCompat1Dot0Dot0Adapter.createEditorConfigOverride` (empty without entries).
fn spotless_editor_config_override(entries: &[(&str, &str)], defaults: &EditorConfigDefaults, providers: &[RuleV2Provider]) -> EditorConfigOverride {
    if entries.is_empty() {
        return EditorConfigOverride::empty();
    }
    let mut entries = entries.to_vec();
    let code_style = CODE_STYLE_PROPERTY.name.as_ref();
    let code_style_in_defaults = defaults.value.sections().iter().any(|s| s.properties().iter().any(|p| p.name() == code_style));
    if !code_style_in_defaults && !entries.iter().any(|(name, _)| *name == code_style) {
        entries.push((code_style, "intellij_idea"));
    }
    let standard: [PropertyRef; 7] = [
        (&*CODE_STYLE_PROPERTY).into(),
        (&*END_OF_LINE_PROPERTY).into(),
        (&*INDENT_SIZE_PROPERTY).into(),
        (&*INDENT_STYLE_PROPERTY).into(),
        (&*INSERT_FINAL_NEWLINE_PROPERTY).into(),
        (&*MAX_LINE_LENGTH_PROPERTY).into(),
        (&*EXPERIMENTAL_RULES_EXECUTION_PROPERTY).into(),
    ];
    let supported: Vec<&PropertyRef> = providers.iter().flat_map(|p| p.uses_editor_config_properties()).chain(&standard).collect();
    let properties: Vec<(PropertyRef, Option<String>)> = entries
        .into_iter()
        .filter_map(|(name, value)| {
            let property = match supported.iter().find(|p| p.name() == name) {
                Some(property) => (*property).clone(),
                None => match name.strip_prefix("ktlint_")?.split_once('_') {
                    None => create_rule_set_execution_editor_config_property(&name["ktlint_".len()..], RuleExecution::Enabled).into(),
                    Some((rule_set, rule)) => {
                        create_rule_execution_editor_config_property(&format!("{rule_set}:{rule}"), RuleExecution::Enabled).into()
                    }
                },
            };
            Some((property, Some(value.to_owned())))
        })
        .collect();
    if properties.is_empty() { EditorConfigOverride::empty() } else { EditorConfigOverride::from(properties) }
}

fn escape(detail: &str) -> String {
    detail.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n").replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_stay_on_their_header_line() {
        assert_eq!(escape("a\\n\nb\r\n\t"), "a\\\\n\\nb\\r\\n\\t");
    }
}
