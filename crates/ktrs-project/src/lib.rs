//! Best-effort detection of the Kotlin formatter and linter a project's build configures (for `ktrs lsp`).
//! Nothing is executed: build files are read statically, so anything computed at configuration time is
//! invisible. When nothing is recognised, `format` and `ktlint` are `None`; fallbacks belong to the caller.
//!
//! # Sources
//! - Gradle Kotlin DSL (`*.gradle.kts`, convention `*.kt`): parsed with ktrs-parser, lowered to a small call
//!   IR (`ir.rs`), then interpreted (`gradle/interpret.rs`). Groovy (`*.gradle`): a tolerant tokenizer and
//!   command-expression parser (`groovy/`) into the same IR.
//! - Per module: the root build's `allprojects {}`/`subprojects {}` blocks (plus a root Spotless section whose
//!   `target` has `**`, and a root task running ktlint's `Main` over `**` args), convention plugins applied by id (precompiled `*.gradle[.kts]` scripts and
//!   `gradlePlugin { plugins { register } }` classes in `buildSrc`, `build-logic` or an `includeBuild` dir,
//!   with calls into their top-level functions inlined), `apply(from = ...)` scripts, then the module's own
//!   build file; later sources override earlier ones field by field.
//! - Plugins: ktfmt-gradle (`com.ncorti.ktfmt.gradle`, `io.github.hexay.ktrs`), ktlint-gradle
//!   (`org.jlleitschuh.gradle.ktlint`, `io.github.hexay.ktrs.ktlint`) + `ktlintRuleset` deps, Spotless
//!   `kotlin {}`/`kotlinGradle {}` (`ktfmt()`, `ktlint()`, ktrs's `KtrsStep`/`KtrsKtlintStep`), kotlinter
//!   (`org.jmailen.kotlinter`, `io.github.hexay.ktrs.kotlinter`) + `ktlint` deps, and a `ktlint`
//!   configuration holding the ktlint CLI (the JavaExec recipe). Values resolve through string
//!   literals, `val`s / `ext` properties of the same file, `gradle.properties` and `gradle/libs.versions.toml`.
//! - Maven (`maven/`): the pom chain from the topmost contiguous `pom.xml` down to the module: gantsign
//!   `ktlint-maven-plugin` (and ktrs's drop-in), `spotless-maven-plugin` `<kotlin>`, antrun/exec running the
//!   ktlint CLI. `${prop}` resolves from `<properties>`; `<parent>` outside the directory chain is not read.
//!
//! # Limits
//! Values from code (`if`, loops, string building beyond templates of known names), other version catalogs
//! than `libs`, `configure(Class)` forms other than the type-argument ones, Groovy slashy strings and
//! closures passed as values, and Maven profiles are not followed. Versions outside 1.8 / 2.0 map to the
//! nearest supported mode with a note.
//!
//! [`migrate`] (`ktrs migrate`) rewrites a build to the ktrs drop-ins; it tokenizes with spans of its own.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

mod catalog;
mod findings;
mod gradle;
mod groovy;
mod ir;
mod kotlin_dsl;
mod layout;
mod maven;
pub mod migrate;
mod reader;
mod version;
mod xml;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    /// Build root: the dir of `settings.gradle[.kts]`, of the topmost `pom.xml`, or of the file.
    pub root: PathBuf,
    /// `None`: no formatter detected.
    pub format: Option<FormatTool>,
    /// `None`: no ktlint detected.
    pub ktlint: Option<KtlintConfig>,
    /// Provenance, e.g. `app/build.gradle.kts: ktfmt { kotlinLangStyle() }`.
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatTool {
    Ktfmt(KtfmtSettings),
    Ktlint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KtfmtStyle {
    Meta,
    Google,
    Kotlinlang,
}

/// `None` fields were not set by the build (the style's defaults apply).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KtfmtSettings {
    pub style: KtfmtStyle,
    pub max_width: Option<u32>,
    pub block_indent: Option<u32>,
    pub continuation_indent: Option<u32>,
    pub remove_unused_imports: Option<bool>,
    pub manage_trailing_commas: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Coordinates or paths as written in the build.
    pub rule_sets: Vec<String>,
}

struct Entry {
    config: ProjectConfig,
    inputs: Vec<PathBuf>,
}

static CACHE: LazyLock<Mutex<HashMap<(PathBuf, bool), Entry>>> = LazyLock::new(Default::default);

/// Detects for the module containing `file` (a file or a directory); cached per module build file.
pub fn detect(file: &Path) -> ProjectConfig {
    let file = absolute(file);
    let script = file.extension().is_some_and(|e| e == "kts");
    let layout = layout::locate(&file);
    let key = (layout.key().to_path_buf(), script);
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(entry) = cache.get(&key) {
        return entry.config.clone();
    }
    let mut reader = reader::Reader::default();
    let config = layout.analyze(script, &mut reader);
    cache.insert(key, Entry { config: config.clone(), inputs: reader.into_inputs() });
    config
}

/// Drops cached results that read `changed`, and every result under its root when `changed` is a build file
/// (it may be new).
pub fn invalidate(changed: &Path) {
    let changed = absolute(changed);
    let build_file = is_build_file(&changed);
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache.retain(|_, e| !(e.inputs.contains(&changed) || build_file && changed.starts_with(&e.config.root)));
}

/// Whether a change to `path` can change a detection result.
pub fn is_build_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else { return false };
    if matches!(name, "pom.xml" | "gradle.properties" | "libs.versions.toml" | "ktlint-plugins.properties")
        || name.ends_with(".gradle.kts")
        || name.ends_with(".gradle")
    {
        return true;
    }
    name.ends_with(".kt")
        && path.components().any(|c| matches!(c.as_os_str().to_str(), Some("buildSrc" | "build-logic")))
}

fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}
