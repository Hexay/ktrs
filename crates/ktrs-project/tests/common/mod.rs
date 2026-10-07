#![allow(dead_code)]

use std::path::{Path, PathBuf};

use ktrs_project::{KtfmtSettings, KtfmtStyle, KtlintConfig, KtlintVersion, ProjectConfig, detect};

pub fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(rel)
}

pub fn detect_fixture(rel: &str) -> ProjectConfig {
    detect(&fixture(rel))
}

pub fn ktfmt(style: KtfmtStyle) -> KtfmtSettings {
    ktfmt_settings(style, |_| {})
}

pub fn ktfmt_settings(style: KtfmtStyle, f: impl FnOnce(&mut KtfmtSettings)) -> KtfmtSettings {
    let mut k = KtfmtSettings {
        style,
        max_width: None,
        block_indent: None,
        continuation_indent: None,
        remove_unused_imports: None,
        manage_trailing_commas: None,
    };
    f(&mut k);
    k
}

pub fn ktlint(version: KtlintVersion) -> KtlintConfig {
    ktlint_config(version, |_| {})
}

pub fn ktlint_config(version: KtlintVersion, f: impl FnOnce(&mut KtlintConfig)) -> KtlintConfig {
    let mut k = KtlintConfig {
        version,
        android: None,
        experimental: None,
        editorconfig_overrides: Vec::new(),
        rule_sets: Vec::new(),
    };
    f(&mut k);
    k
}

pub fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

pub fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

pub fn has_note(config: &ProjectConfig, needle: &str) -> bool {
    config.notes.iter().any(|n| n.contains(needle))
}
