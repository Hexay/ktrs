//! The editor settings (keys: the crate docs).

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Settings {
    pub format: FormatSettings,
    pub ktfmt: KtfmtSettings,
    pub ktlint: KtlintSettings,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct FormatSettings {
    pub tool: FormatToolSetting,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FormatToolSetting {
    #[default]
    Auto,
    Ktfmt,
    Ktlint,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum KtfmtStyleSetting {
    Meta,
    Google,
    Kotlinlang,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct KtfmtSettings {
    pub style: Option<KtfmtStyleSetting>,
    pub max_width: Option<u32>,
    pub block_indent: Option<u32>,
    pub continuation_indent: Option<u32>,
    pub remove_unused_imports: Option<bool>,
    pub manage_trailing_commas: Option<bool>,
    pub editorconfig: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct KtlintSettings {
    pub enable: Enable,
    pub version: Option<KtlintVersionSetting>,
    pub android: Option<bool>,
    pub experimental: Option<bool>,
    pub editorconfig_overrides: Option<serde_json::Map<String, Value>>,
    pub rule_sets: Option<Vec<String>>,
    pub unfixable_as_error: bool,
}

/// `"auto"`, `true` or `false`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Value")]
pub(crate) enum Enable {
    #[default]
    Auto,
    On,
    Off,
}

impl TryFrom<Value> for Enable {
    type Error = String;

    fn try_from(value: Value) -> Result<Enable, String> {
        match value {
            Value::Bool(true) => Ok(Enable::On),
            Value::Bool(false) => Ok(Enable::Off),
            Value::String(s) if s == "auto" => Ok(Enable::Auto),
            Value::String(s) if s == "true" => Ok(Enable::On),
            Value::String(s) if s == "false" => Ok(Enable::Off),
            other => Err(format!("expected \"auto\", true or false, got {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum KtlintVersionSetting {
    #[serde(rename = "1.8")]
    V1_8,
    #[serde(rename = "2.0")]
    V2_0,
}

impl Settings {
    /// `value` bare or under a `ktrs` key; `null` is the defaults.
    pub(crate) fn parse(value: &Value) -> Result<Settings, String> {
        let value = value.get("ktrs").unwrap_or(value);
        if value.is_null() {
            return Ok(Settings::default());
        }
        Settings::deserialize(value).map_err(|e| format!("invalid ktrs settings: {e}"))
    }

    /// The `.editorconfig` overrides as `(name, value)`, non-string values in their JSON form.
    pub(crate) fn ktlint_editorconfig_overrides(&self) -> Option<Vec<(String, String)>> {
        let overrides = self.ktlint.editorconfig_overrides.as_ref()?;
        Some(
            overrides
                .iter()
                .map(|(name, value)| (name.clone(), value.as_str().map_or_else(|| value.to_string(), str::to_owned)))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_nested_settings_bare_or_under_ktrs() {
        let value = json!({"ktrs": {"format": {"tool": "ktfmt"}, "ktfmt": {"style": "google", "maxWidth": 120},
            "ktlint": {"enable": false, "version": "2.0", "editorconfigOverrides": {"max_line_length": 100}}}});
        let settings = Settings::parse(&value).unwrap();
        assert_eq!(settings, Settings::parse(&value["ktrs"]).unwrap());
        assert_eq!(settings.format.tool, FormatToolSetting::Ktfmt);
        assert_eq!(settings.ktfmt.style, Some(KtfmtStyleSetting::Google));
        assert_eq!(settings.ktfmt.max_width, Some(120));
        assert_eq!(settings.ktlint.enable, Enable::Off);
        assert_eq!(settings.ktlint.version, Some(KtlintVersionSetting::V2_0));
        assert_eq!(settings.ktlint_editorconfig_overrides(), Some(vec![("max_line_length".to_owned(), "100".to_owned())]));
        assert_eq!(Settings::parse(&Value::Null).unwrap(), Settings::default());
        assert!(Settings::parse(&json!({"ktlint": {"enable": "sometimes"}})).is_err());
    }
}
