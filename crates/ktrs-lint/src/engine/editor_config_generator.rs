//! Port of ktlint-rule-engine `internal/EditorConfigGenerator.kt` and `KtLintRuleEngine.generateKotlinEditorConfigSection`.

use std::path::Path;

use ktrs_editorconfig::{EnumValue, ParseException};

use crate::editorconfig::{
    CODE_STYLE_PROPERTY, CODE_STYLE_PROPERTY_TYPE, CodeStyleValue, END_OF_LINE_PROPERTY,
    EXPERIMENTAL_RULES_EXECUTION_PROPERTY, KTLINT_VERSION_PROPERTY, KTLINT_VERSION_PROPERTY_TYPE, KtlintVersion, PropertyRef,
};
use crate::engine::editor_config_defaults::{EditorConfigDefaults, EditorConfigOverride};
use crate::engine::editor_config_loader::{EditorConfigLoader, EditorConfigLoaderEc4j};
use crate::engine::formatter_tags::{
    FORMATTER_TAG_OFF_ENABLED_PROPERTY, FORMATTER_TAG_ON_ENABLED_PROPERTY,
    FORMATTER_TAGS_ENABLED_PROPERTY,
};
use crate::engine::ktlint_rule_engine::KtLintRuleEngine;
use crate::rule_provider::property_types;

const LINE_SEPARATOR: &str = if cfg!(windows) { "\r\n" } else { "\n" };

impl KtLintRuleEngine {
    /// `generateKotlinEditorConfigSection(filePath)`: `name = value` lines (sorted) for every property the
    /// rules use, the `.editorconfig` files on `file_path` applied, else the code style's defaults.
    pub fn generate_kotlin_editor_config_section(&self, file_path: &Path) -> Result<String, ParseException> {
        let code_style = self
            .editor_config_override()
            .get(&PropertyRef::from(&*CODE_STYLE_PROPERTY))
            .and_then(|value| CODE_STYLE_PROPERTY_TYPE.parse(value.source()).into_parsed())
            .unwrap_or(CODE_STYLE_PROPERTY.default_value);
        let mut used_editor_config_properties: Vec<PropertyRef> = Vec::new();
        for property in self.rule_providers().iter().flat_map(|p| p.uses_editor_config_properties()) {
            if !used_editor_config_properties.iter().any(|p| p.identity() == property.identity()) {
                used_editor_config_properties.push(property.clone());
            }
        }
        let mut kept_names: Vec<String> = used_editor_config_properties.iter().map(|p| p.name().to_owned()).collect();
        kept_names.extend(GENERATOR_DEFAULT_PROPERTIES.iter().map(|p| p.to_string()));
        used_editor_config_properties.extend(default_editor_config_properties());
        let editor_config = load_editor_config(self, code_style, file_path)?
            .add_properties_with_default_value_if_missing(&used_editor_config_properties);
        // 1.8's `filterBy` keeps only the used properties; 2.0's keeps whatever was loaded too.
        let ktlint_1_8 = self
            .editor_config_override()
            .get(&PropertyRef::from(&*KTLINT_VERSION_PROPERTY))
            .and_then(|value| KTLINT_VERSION_PROPERTY_TYPE.parse(value.source()).into_parsed())
            .is_some_and(KtlintVersion::is_1_8);
        let mut lines: Vec<String> = Vec::new();
        for line in editor_config
            .map(|p| (p.name().to_owned(), format!("{} = {}", p.name(), p.source_value().unwrap_or("null"))))
            .into_iter()
            .filter(|(name, _)| !ktlint_1_8 || kept_names.contains(name))
            .map(|(_, line)| line)
        {
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        lines.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
        Ok(lines.join(LINE_SEPARATOR))
    }
}

/// `DEFAULT_EDITOR_CONFIG_PROPERTIES` of `DefaultEditorConfigProperties.kt`, which the generator adds to the rules'.
const GENERATOR_DEFAULT_PROPERTIES: [&str; 6] =
    ["ktlint_code_style", "end_of_line", "indent_style", "indent_size", "insert_final_newline", "max_line_length"];

/// `DEFAULT_EDITOR_CONFIG_PROPERTIES` (`EditorConfigLoader.kt`).
fn default_editor_config_properties() -> [PropertyRef; 6] {
    [
        PropertyRef::from(&*CODE_STYLE_PROPERTY),
        PropertyRef::from(&*EXPERIMENTAL_RULES_EXECUTION_PROPERTY),
        PropertyRef::from(&*END_OF_LINE_PROPERTY),
        PropertyRef::from(&*FORMATTER_TAGS_ENABLED_PROPERTY),
        PropertyRef::from(&*FORMATTER_TAG_OFF_ENABLED_PROPERTY),
        PropertyRef::from(&*FORMATTER_TAG_ON_ENABLED_PROPERTY),
    ]
}

fn load_editor_config(
    engine: &KtLintRuleEngine,
    code_style: CodeStyleValue,
    file_path: &Path,
) -> Result<crate::editorconfig::EditorConfig, ParseException> {
    EditorConfigLoader::new(
        EditorConfigLoaderEc4j::new(&property_types(engine.rule_providers())),
        EditorConfigDefaults::empty(),
        EditorConfigOverride::from(vec![(
            PropertyRef::from(&*CODE_STYLE_PROPERTY),
            Some(code_style.name().to_owned()),
        )]),
    )
    .load(Some(file_path))
}
