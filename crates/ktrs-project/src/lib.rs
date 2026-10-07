//! A project's ktfmt/ktlint configuration, as its build files declare it.
//!
//! TODO: stub with the real crate's API; [`detect`] finds nothing until the real crate replaces it.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    pub root: PathBuf,
    pub format: Option<FormatTool>,
    pub ktlint: Option<KtlintConfig>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatTool {
    Ktfmt(KtfmtSettings),
    Ktlint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KtfmtStyle {
    Meta,
    Google,
    Kotlinlang,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KtfmtSettings {
    pub style: KtfmtStyle,
    pub max_width: Option<u32>,
    pub block_indent: Option<u32>,
    pub continuation_indent: Option<u32>,
    pub remove_unused_imports: Option<bool>,
    pub manage_trailing_commas: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KtlintVersion {
    V1_8,
    V2_0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KtlintConfig {
    pub version: KtlintVersion,
    pub android: Option<bool>,
    pub experimental: Option<bool>,
    pub editorconfig_overrides: Vec<(String, String)>,
    pub rule_sets: Vec<String>,
}

/// The configuration that applies to `file`, cached per build root.
pub fn detect(file: &Path) -> ProjectConfig {
    let root = file.parent().unwrap_or(file).to_path_buf();
    ProjectConfig { root, format: None, ktlint: None, notes: Vec::new() }
}

/// Drops cached configurations that `changed` (a build file) feeds.
pub fn invalidate(_changed: &Path) {}

pub fn is_build_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    matches!(name, "build.gradle" | "build.gradle.kts" | "settings.gradle" | "settings.gradle.kts" | "pom.xml")
}
