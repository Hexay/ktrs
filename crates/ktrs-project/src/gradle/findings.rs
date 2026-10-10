//! What the Gradle sources of one module configure, and the choice of tools from it.

use crate::findings::{KtfmtPartial, KtlintPartial};
use crate::{FormatTool, KtlintConfig};

pub(crate) const KTFMT_PLUGINS: [&str; 2] = ["com.ncorti.ktfmt.gradle", "io.github.hexay.ktrs"];
pub(crate) const KTLINT_PLUGINS: [&str; 2] = ["org.jlleitschuh.gradle.ktlint", "io.github.hexay.ktrs.ktlint"];
pub(crate) const SPOTLESS_PLUGINS: [&str; 2] = ["com.diffplug.spotless", "com.diffplug.gradle.spotless"];
pub(crate) const KOTLINTER_PLUGINS: [&str; 2] = ["org.jmailen.kotlinter", "io.github.hexay.ktrs.kotlinter"];

/// Plugin classes applied by type (`apply<KtlintPlugin>()`).
pub(crate) fn plugin_of_type(name: &str) -> Option<&'static str> {
    match name.split("::").next()?.rsplit('.').next()? {
        "KtfmtPlugin" => Some(KTFMT_PLUGINS[0]),
        "KtlintPlugin" => Some(KTLINT_PLUGINS[0]),
        "SpotlessPlugin" => Some(SPOTLESS_PLUGINS[0]),
        "KotlinterPlugin" => Some(KOTLINTER_PLUGINS[0]),
        _ => None,
    }
}

pub(crate) fn is_tool_plugin(id: &str) -> bool {
    [&KTFMT_PLUGINS[..], &KTLINT_PLUGINS, &SPOTLESS_PLUGINS, &KOTLINTER_PLUGINS].iter().any(|ids| ids.contains(&id))
}

/// ktlint's own artifacts in a `ktlint` configuration (the CLI), as opposed to rule sets.
fn is_ktlint_cli(coords: &str) -> bool {
    let mut parts = coords.split(':');
    let group = parts.next().unwrap_or_default();
    let artifact = parts.next().unwrap_or_default();
    matches!(group, "com.pinterest" | "com.pinterest.ktlint" | "io.github.ktlint.core" | "com.github.shyiko")
        && matches!(artifact, "ktlint" | "ktlint-cli")
        || coords.ends_with("ktlintCli")
}

/// A Spotless format section: `kotlin {}` or `kotlinGradle {}`.
#[derive(Debug, Default, Clone)]
pub(crate) struct Section {
    pub ktfmt: Option<KtfmtPartial>,
    pub ktlint: Option<KtlintPartial>,
    pub targets: Vec<String>,
}

impl Section {
    fn merge(&mut self, o: Section) {
        merge_opt(&mut self.ktfmt, o.ktfmt, KtfmtPartial::merge);
        merge_opt(&mut self.ktlint, o.ktlint, KtlintPartial::merge);
        self.targets.extend(o.targets);
    }

    fn has_step(&self) -> bool {
        self.ktfmt.is_some() || self.ktlint.is_some()
    }
}

fn merge_opt<T>(into: &mut Option<T>, from: Option<T>, merge: fn(&mut T, &T)) {
    match (into.as_mut(), from) {
        (Some(a), Some(b)) => merge(a, &b),
        (None, Some(b)) => *into = Some(b),
        _ => {}
    }
}

#[derive(Debug, Default)]
pub(crate) struct Findings {
    pub plugins: Vec<String>,
    /// `ktfmt { }` (ktfmt-gradle).
    pub ktfmt: Option<KtfmtPartial>,
    /// `ktlint { }` (ktlint-gradle).
    pub ktlint: Option<KtlintPartial>,
    pub kotlinter: Option<KtlintPartial>,
    pub spotless_kotlin: Section,
    pub spotless_kotlin_gradle: Section,
    /// Dependencies in the `ktlint` configuration.
    pub ktlint_deps: Vec<String>,
    /// Dependencies in `ktlintRuleset`.
    pub ruleset_deps: Vec<String>,
    /// A task sets ktlint's `Main` as `mainClass` / passes a `**` pattern in `args` (the JavaExec recipe).
    pub ktlint_main_class: bool,
    pub wide_args: bool,
    pub notes: Vec<String>,
}

impl Findings {
    pub(crate) fn merge(&mut self, o: Findings) {
        for id in o.plugins {
            if !self.plugins.contains(&id) {
                self.plugins.push(id);
            }
        }
        merge_opt(&mut self.ktfmt, o.ktfmt, KtfmtPartial::merge);
        merge_opt(&mut self.ktlint, o.ktlint, KtlintPartial::merge);
        merge_opt(&mut self.kotlinter, o.kotlinter, KtlintPartial::merge);
        self.spotless_kotlin.merge(o.spotless_kotlin);
        self.spotless_kotlin_gradle.merge(o.spotless_kotlin_gradle);
        self.ktlint_deps.extend(o.ktlint_deps);
        self.ruleset_deps.extend(o.ruleset_deps);
        self.ktlint_main_class |= o.ktlint_main_class;
        self.wide_args |= o.wide_args;
        self.notes.extend(o.notes);
    }

    /// Keeps what a root project's own configuration applies to every module: Spotless sections with a `**`
    /// target, and the ktlint CLI run by a task over `**` patterns.
    pub(crate) fn root_wide(self) -> Findings {
        let wide = |s: Section| if s.targets.iter().any(|t| t.contains("**")) { s } else { Section::default() };
        let spotless_kotlin = wide(self.spotless_kotlin);
        let spotless_kotlin_gradle = wide(self.spotless_kotlin_gradle);
        let cli_wide = self.ktlint_main_class && self.wide_args;
        let ktlint_deps = if cli_wide { self.ktlint_deps } else { Vec::new() };
        let any = spotless_kotlin.has_step() || spotless_kotlin_gradle.has_step() || !ktlint_deps.is_empty();
        let notes = if any { self.notes } else { Vec::new() };
        Findings { spotless_kotlin, spotless_kotlin_gradle, ktlint_deps, notes, ..Findings::default() }
    }

    /// ktlint-gradle's `ktlint-plugins.properties` fallback for an unset `version`.
    pub(crate) fn default_ktlint_gradle_version(&mut self, v: &str) {
        if !self.applied(&KTLINT_PLUGINS) && self.ktlint.is_none() {
            return;
        }
        let k = self.ktlint.get_or_insert_with(Default::default);
        if k.version.is_none() {
            k.version = Some(v.to_string());
            self.notes.push(format!("ktlint-plugins.properties: ktlint-version={v}"));
        }
    }

    fn applied(&self, ids: &[&str]) -> bool {
        self.plugins.iter().any(|p| ids.contains(&p.as_str()))
    }

    /// `script`: the file is a `.kts` (Spotless `kotlinGradle {}` wins when it has a step).
    pub(crate) fn resolve(self, script: bool) -> (Option<FormatTool>, Option<KtlintConfig>, Vec<String>) {
        let mut notes = self.notes.clone();
        let section = if script && self.spotless_kotlin_gradle.has_step() {
            &self.spotless_kotlin_gradle
        } else {
            &self.spotless_kotlin
        };
        let ktfmt = if self.applied(&KTFMT_PLUGINS) || self.ktfmt.is_some() {
            Some(self.ktfmt.clone().unwrap_or_default())
        } else {
            section.ktfmt.clone()
        };
        let ktlint = self.ktlint_config(section, &mut notes);
        if ktfmt.is_some() && ktlint.is_some() {
            notes.push("both ktfmt and ktlint configured: formatting with ktfmt".into());
        }
        let format = match ktfmt {
            Some(k) => Some(FormatTool::Ktfmt(k.finish())),
            None => ktlint.as_ref().map(|_| FormatTool::Ktlint),
        };
        (format, ktlint, notes)
    }

    fn ktlint_config(&self, section: &Section, notes: &mut Vec<String>) -> Option<KtlintConfig> {
        if self.applied(&KTLINT_PLUGINS) || self.ktlint.is_some() {
            let mut p = self.ktlint.clone().unwrap_or_default();
            self.ruleset_deps.iter().for_each(|r| p.add_rule_set(r));
            return Some(p.finish("ktlint-gradle", notes));
        }
        if let Some(p) = &section.ktlint {
            return Some(p.finish("Spotless ktlint()", notes));
        }
        let (cli, rule_sets): (Vec<&String>, Vec<&String>) = self.ktlint_deps.iter().partition(|d| is_ktlint_cli(d));
        if self.applied(&KOTLINTER_PLUGINS) || self.kotlinter.is_some() {
            let mut p = self.kotlinter.clone().unwrap_or_default();
            rule_sets.iter().for_each(|r| p.add_rule_set(r));
            return Some(p.finish("kotlinter", notes));
        }
        let cli = cli.first()?;
        let mut p = KtlintPartial { version: cli.split(':').nth(2).map(str::to_string), ..KtlintPartial::default() };
        rule_sets.iter().for_each(|r| p.add_rule_set(r));
        notes.push(format!("ktlint CLI from the `ktlint` configuration: {cli}"));
        Some(p.finish("ktlint CLI", notes))
    }
}
