//! A file's effective configuration: the build's ([`ktrs_project::detect`]), the editor settings over it, and
//! the fallback when neither says anything (precedence: the crate docs).

use std::path::PathBuf;

use ktrs_project::{FormatTool, KtfmtSettings, KtfmtStyle, KtlintConfig, KtlintVersion, ProjectConfig};

use crate::settings::{Enable, FormatToolSetting, KtfmtStyleSetting, KtlintVersionSetting, Settings};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Effective {
    pub root: PathBuf,
    pub formatter: Option<Formatter>,
    /// The diagnostics' configuration; `None`: no diagnostics.
    pub ktlint: Option<KtlintConfig>,
    pub unfixable_as_error: bool,
    /// Where the values came from, for the log.
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Formatter {
    Ktfmt { settings: KtfmtSettings, editorconfig: bool },
    Ktlint(KtlintConfig),
}

pub(crate) fn resolve(detected: ProjectConfig, settings: &Settings) -> Effective {
    let mut notes = detected.notes;
    let nothing_detected = detected.format.is_none() && detected.ktlint.is_none();
    if nothing_detected {
        notes.push("no ktfmt/ktlint configuration found in the build files".to_owned());
    }
    let with_settings = |config: Option<KtlintConfig>| ktlint_with_settings(config.unwrap_or_else(default_ktlint), settings);
    let ktlint = match settings.ktlint.enable {
        Enable::Off => {
            notes.push("ktlint diagnostics off: ktlint.enable = false".to_owned());
            None
        }
        Enable::On => Some(with_settings(detected.ktlint.clone())),
        Enable::Auto if detected.ktlint.is_some() || nothing_detected => {
            if nothing_detected {
                notes.push("ktlint diagnostics by default (ktlint 1.8 unless ktlint.version says otherwise)".to_owned());
            }
            Some(with_settings(detected.ktlint.clone()))
        }
        Enable::Auto => {
            notes.push("ktlint diagnostics off: the build doesn't use ktlint (ktlint.enable = true turns them on)".to_owned());
            None
        }
    };
    let ktlint_formatter = || Formatter::Ktlint(ktlint.clone().unwrap_or_else(|| with_settings(detected.ktlint.clone())));
    let detected_ktfmt = match &detected.format {
        Some(FormatTool::Ktfmt(ktfmt)) => Some(ktfmt.clone()),
        _ => None,
    };
    let formatter = match (settings.format.tool, &detected.format) {
        (FormatToolSetting::None, _) | (FormatToolSetting::Auto, None) => None,
        (FormatToolSetting::Ktfmt, _) | (FormatToolSetting::Auto, Some(FormatTool::Ktfmt(_))) => Some(Formatter::Ktfmt {
            settings: ktfmt_with_settings(detected_ktfmt.unwrap_or_else(default_ktfmt), settings),
            editorconfig: settings.ktfmt.editorconfig,
        }),
        (FormatToolSetting::Ktlint, _) | (FormatToolSetting::Auto, Some(FormatTool::Ktlint)) => Some(ktlint_formatter()),
    };
    if formatter.is_none() {
        notes.push("formatting off: no formatter in the build files or format.tool".to_owned());
    }
    Effective { root: detected.root, formatter, ktlint, unfixable_as_error: settings.ktlint.unfixable_as_error, notes }
}

impl Effective {
    /// The log line: the configuration and its notes.
    pub(crate) fn describe(&self) -> String {
        let formatter = match &self.formatter {
            None => "none".to_owned(),
            Some(Formatter::Ktfmt { settings, editorconfig }) => format!("ktfmt {settings:?}, .editorconfig: {editorconfig}"),
            Some(Formatter::Ktlint(config)) => format!("ktlint {config:?}"),
        };
        let diagnostics = self.ktlint.as_ref().map_or("none".to_owned(), |c| format!("ktlint {c:?}"));
        let notes: String = self.notes.iter().map(|n| format!("\n  - {n}")).collect();
        let root = if self.root.as_os_str().is_empty() { "documents without a file".to_owned() } else { self.root.display().to_string() };
        format!("ktrs: {root}\n  formatting: {formatter}\n  diagnostics: {diagnostics}{notes}")
    }
}

fn default_ktlint() -> KtlintConfig {
    KtlintConfig { version: KtlintVersion::V1_8, android: None, experimental: None, editorconfig_overrides: Vec::new(), rule_sets: Vec::new() }
}

fn default_ktfmt() -> KtfmtSettings {
    KtfmtSettings {
        style: KtfmtStyle::Meta,
        max_width: None,
        block_indent: None,
        continuation_indent: None,
        remove_unused_imports: None,
        manage_trailing_commas: None,
    }
}

fn ktlint_with_settings(mut config: KtlintConfig, settings: &Settings) -> KtlintConfig {
    let s = &settings.ktlint;
    if let Some(version) = s.version {
        config.version = match version {
            KtlintVersionSetting::V1_8 => KtlintVersion::V1_8,
            KtlintVersionSetting::V2_0 => KtlintVersion::V2_0,
        };
    }
    config.android = s.android.or(config.android);
    config.experimental = s.experimental.or(config.experimental);
    if let Some(overrides) = settings.ktlint_editorconfig_overrides() {
        config.editorconfig_overrides = overrides;
    }
    if let Some(rule_sets) = &s.rule_sets {
        config.rule_sets = rule_sets.clone();
    }
    config
}

fn ktfmt_with_settings(mut ktfmt: KtfmtSettings, settings: &Settings) -> KtfmtSettings {
    let s = &settings.ktfmt;
    if let Some(style) = s.style {
        ktfmt.style = match style {
            KtfmtStyleSetting::Meta => KtfmtStyle::Meta,
            KtfmtStyleSetting::Google => KtfmtStyle::Google,
            KtfmtStyleSetting::Kotlinlang => KtfmtStyle::Kotlinlang,
        };
    }
    ktfmt.max_width = s.max_width.or(ktfmt.max_width);
    ktfmt.block_indent = s.block_indent.or(ktfmt.block_indent);
    ktfmt.continuation_indent = s.continuation_indent.or(ktfmt.continuation_indent);
    ktfmt.remove_unused_imports = s.remove_unused_imports.or(ktfmt.remove_unused_imports);
    ktfmt.manage_trailing_commas = s.manage_trailing_commas.or(ktfmt.manage_trailing_commas);
    ktfmt
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn detected(format: Option<FormatTool>, ktlint: Option<KtlintConfig>) -> ProjectConfig {
        ProjectConfig { root: PathBuf::from("/p"), format, ktlint, notes: Vec::new() }
    }

    fn settings(value: serde_json::Value) -> Settings {
        Settings::parse(&value).unwrap()
    }

    #[test]
    fn nothing_detected_falls_back_to_ktlint_1_8_diagnostics_without_formatting() {
        let effective = resolve(detected(None, None), &Settings::default());
        assert_eq!(effective.ktlint, Some(default_ktlint()));
        assert_eq!(effective.formatter, None);
    }

    #[test]
    fn a_ktfmt_build_formats_with_ktfmt_and_has_no_ktlint_diagnostics() {
        let effective = resolve(detected(Some(FormatTool::Ktfmt(default_ktfmt())), None), &settings(json!({"ktfmt": {"maxWidth": 90}})));
        assert_eq!(effective.ktlint, None);
        let Some(Formatter::Ktfmt { settings, editorconfig: false }) = effective.formatter else { panic!("{effective:?}") };
        assert_eq!((settings.style, settings.max_width), (KtfmtStyle::Meta, Some(90)));
    }

    #[test]
    fn editor_settings_override_the_build() {
        let build = KtlintConfig { version: KtlintVersion::V2_0, android: Some(true), ..default_ktlint() };
        let s = settings(json!({"format": {"tool": "ktlint"}, "ktlint": {"version": "1.8", "ruleSets": ["x.jar"]}}));
        let effective = resolve(detected(None, Some(build)), &s);
        let expected = KtlintConfig { version: KtlintVersion::V1_8, android: Some(true), rule_sets: vec!["x.jar".to_owned()], ..default_ktlint() };
        assert_eq!(effective.ktlint.as_ref(), Some(&expected));
        assert_eq!(effective.formatter, Some(Formatter::Ktlint(expected)));
        let off = resolve(detected(None, None), &settings(json!({"ktlint": {"enable": false}, "format": {"tool": "ktfmt"}})));
        assert_eq!(off.ktlint, None);
        assert!(matches!(off.formatter, Some(Formatter::Ktfmt { .. })));
    }
}
