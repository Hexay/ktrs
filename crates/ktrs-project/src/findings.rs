//! Partially known tool settings, merged source by source (later sources win per field), then finished
//! into the public types.

use crate::version::ktlint_version;
use crate::{KtfmtSettings, KtfmtStyle, KtlintConfig, KtlintVersion};

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct KtfmtPartial {
    pub style: Option<KtfmtStyle>,
    pub max_width: Option<u32>,
    pub block_indent: Option<u32>,
    pub continuation_indent: Option<u32>,
    pub remove_unused_imports: Option<bool>,
    pub manage_trailing_commas: Option<bool>,
}

fn take<T: Clone>(into: &mut Option<T>, from: &Option<T>) {
    if from.is_some() {
        into.clone_from(from);
    }
}

pub(crate) fn style(name: &str) -> Option<KtfmtStyle> {
    let name = name.to_ascii_lowercase();
    let name = name.trim_end_matches("style").trim_end_matches("_format");
    match name {
        "meta" | "default" | "facebook" => Some(KtfmtStyle::Meta),
        "google" => Some(KtfmtStyle::Google),
        // ktfmt's Dropbox style was renamed Kotlinlang.
        "kotlinlang" | "kotlin_lang" | "dropbox" => Some(KtfmtStyle::Kotlinlang),
        _ => None,
    }
}

impl KtfmtPartial {
    pub(crate) fn merge(&mut self, o: &KtfmtPartial) {
        take(&mut self.style, &o.style);
        take(&mut self.max_width, &o.max_width);
        take(&mut self.block_indent, &o.block_indent);
        take(&mut self.continuation_indent, &o.continuation_indent);
        take(&mut self.remove_unused_imports, &o.remove_unused_imports);
        take(&mut self.manage_trailing_commas, &o.manage_trailing_commas);
    }

    /// Sets an option by any of its spellings (ktfmt-gradle property, Spotless `setX`, `KtrsOptions.withX`,
    /// Maven element); `false` if `name` is not a ktfmt option or `value` doesn't parse.
    pub(crate) fn set(&mut self, name: &str, value: &str) -> bool {
        let name = name.strip_prefix("set").or_else(|| name.strip_prefix("with")).unwrap_or(name);
        let value = value.trim();
        let int = || value.parse::<u32>().ok();
        let bool = || value.parse::<bool>().ok();
        let slot_set = match name.to_ascii_lowercase().as_str() {
            "maxwidth" => int().map(|v| self.max_width = Some(v)),
            "blockindent" => int().map(|v| self.block_indent = Some(v)),
            "continuationindent" => int().map(|v| self.continuation_indent = Some(v)),
            "removeunusedimports" => bool().map(|v| self.remove_unused_imports = Some(v)),
            "managetrailingcommas" => bool().map(|v| self.manage_trailing_commas = Some(v)),
            "trailingcommamanagementstrategy" | "trailingcommas" => {
                let manage = match value.rsplit('.').next().unwrap_or(value).to_ascii_uppercase().as_str() {
                    "COMPLETE" => Some(true),
                    "NONE" => Some(false),
                    // ONLY_ADD has no ktfmt 0.64 flag: leave the style's default.
                    _ => None,
                };
                manage.map(|v| self.manage_trailing_commas = Some(v))
            }
            "style" => style(value).map(|s| self.style = Some(s)),
            _ => None,
        };
        slot_set.is_some()
    }

    pub(crate) fn finish(&self) -> KtfmtSettings {
        KtfmtSettings {
            style: self.style.unwrap_or(KtfmtStyle::Meta),
            max_width: self.max_width,
            block_indent: self.block_indent,
            continuation_indent: self.continuation_indent,
            remove_unused_imports: self.remove_unused_imports,
            manage_trailing_commas: self.manage_trailing_commas,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct KtlintPartial {
    pub version: Option<String>,
    pub android: Option<bool>,
    pub experimental: Option<bool>,
    pub overrides: Vec<(String, String)>,
    pub rule_sets: Vec<String>,
}

impl KtlintPartial {
    pub(crate) fn merge(&mut self, o: &KtlintPartial) {
        take(&mut self.version, &o.version);
        take(&mut self.android, &o.android);
        take(&mut self.experimental, &o.experimental);
        for (k, v) in &o.overrides {
            self.set_override(k, v);
        }
        for r in &o.rule_sets {
            self.add_rule_set(r);
        }
    }

    pub(crate) fn set_override(&mut self, key: &str, value: &str) {
        self.overrides.retain(|(k, _)| k != key);
        self.overrides.push((key.to_string(), value.to_string()));
    }

    pub(crate) fn add_rule_set(&mut self, coords: &str) {
        if !self.rule_sets.iter().any(|r| r == coords) {
            self.rule_sets.push(coords.to_string());
        }
    }

    /// `tool` names the source in the note when the version falls back to 1.8.
    pub(crate) fn finish(&self, tool: &str, notes: &mut Vec<String>) -> KtlintConfig {
        let version = match &self.version {
            Some(raw) => {
                let (v, note) = ktlint_version(raw);
                notes.extend(note);
                v
            }
            None => {
                notes.push(format!("{tool}: no ktlint version set; using 1.8"));
                KtlintVersion::V1_8
            }
        };
        KtlintConfig {
            version,
            android: self.android,
            experimental: self.experimental,
            editorconfig_overrides: self.overrides.clone(),
            rule_sets: self.rule_sets.clone(),
        }
    }
}
