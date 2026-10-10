//! What the kotlinter drop-in does differently from the kotlinter a build had (research/34): ktlint versions,
//! the 5.x DSL, rule sets.

use super::coords::KOTLINTER;
use super::scan::{Kind, Script};
use super::{Notes, VERSIONS_2_0_AND_1_8};
use crate::catalog::Catalog;

const COMPOSE_RULES: &str = "io.nlopez.compose.rules:ktlint";
/// ktlint's own artifacts: the drop-in leaves them out of the rule sets.
const KTLINT_ITSELF: &str = "com.pinterest.ktlint:";

/// Whether a build file names kotlinter or its drop-in (id or implementation artifact).
pub(crate) fn mentioned(text: &str) -> bool {
    [KOTLINTER.old_id, KOTLINTER.old_artifact, KOTLINTER.new_id].iter().any(|name| text.contains(*name))
}

/// `replaced`: the id, coordinates or catalog entry being swapped; `version`: the plugin version it had.
pub(crate) fn check_plugin_version(replaced: &str, version: &str, notes: &mut Notes) {
    let major = version.split('.').next().and_then(|m| m.parse::<u32>().ok());
    if KOTLINTER.names(replaced) && major.is_some_and(|m| m < 5) {
        notes.gradle(format!(
            "kotlinter {version}: the drop-in has kotlinter 5's DSL; rename `failBuildWhenCannotAutoFormat` to `ignoreFormatFailures` (inverted) and `ignoreFailures` to `ignoreLintFailures`, and declare rule sets of the `buildscript` classpath as `ktlint(..)` dependencies"
        ));
    }
}

/// `kotlinter { ktlintVersion = ".." }` at any depth (nothing else is named `kotlinter`).
pub(crate) fn check_ktlint_version(s: &Script, notes: &mut Notes) {
    for open in (0..s.toks.len()).filter(|&o| s.is_punct(o, '{')) {
        if s.block_name(open) != Some("kotlinter") && s.block_type_arg(open) != Some("KotlinterExtension") {
            continue;
        }
        for i in open + 1..s.close_of(open) {
            if !s.is_ident(i, "ktlintVersion") || !s.is_punct(i + 1, '=') {
                continue;
            }
            if let Some(v) = s.plain_str(i + 2).filter(|v| !VERSIONS_2_0_AND_1_8.contains(v)) {
                notes.gradle(format!(
                    "kotlinter {{ ktlintVersion = \"{v}\" }}: the kotlinter drop-in runs only 1.8.0 or 2.0.0-ALPHA-4; pick one"
                ));
            }
        }
    }
}

/// Rule sets in the `ktlint` configuration of a build that uses kotlinter: `ktlint("g:a:v")`, `ktlint 'g:a:v'`,
/// `ktlint(libs.x)`, `add("ktlint", ..)`, `"ktlint"(..)`. Projects, files and other expressions are skipped.
pub(crate) fn check_rule_sets(s: &Script, catalog: &Catalog, notes: &mut Notes) {
    for i in 0..s.toks.len() {
        let configuration =
            s.is_ident(i, "ktlint") && !s.is_punct(i.wrapping_sub(1), '.') || s.plain_str(i) == Some("ktlint");
        if !configuration {
            continue;
        }
        let arg = if s.is_punct(i + 1, '(') || s.is_punct(i + 1, ',') { i + 2 } else { i + 1 };
        let native = |c: &String| c.starts_with(COMPOSE_RULES) || c.starts_with(KTLINT_ITSELF);
        let Some(coords) = coords_at(s, arg, catalog).filter(|c| !native(c)) else { continue };
        if s.enclosing(i).iter().any(|&o| s.block_name(o) == Some("dependencies")) {
            notes.gradle(format!(
                "ktlint(\"{coords}\"): only compose-rules runs natively; with this rule set the kotlinter drop-in runs through the real ktlint jar"
            ));
        }
    }
}

/// The coordinates of the dependency notation at `arg`: a string literal (templates kept as written) or a
/// `libs.x.y` library of the catalog.
fn coords_at(s: &Script, arg: usize, catalog: &Catalog) -> Option<String> {
    let tok = s.toks.get(arg)?;
    if matches!(tok.kind, Kind::Str { .. }) {
        let content = &s.text[tok.inner.clone()];
        return content.contains(':').then(|| content.to_string());
    }
    if !s.is_ident(arg, "libs") {
        return None;
    }
    let mut alias = Vec::new();
    let mut i = arg + 1;
    while s.is_punct(i, '.') && s.toks.get(i + 1).is_some_and(|t| t.kind == Kind::Ident) && !s.is_ident(i + 1, "get") {
        alias.push(s.src(i + 1));
        i += 2;
    }
    catalog.library(&alias.join(".")).map(str::to_string)
}
