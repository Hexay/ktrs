//! ktfmt-gradle, ktlint-gradle and kotlinter in build scripts: plugin ids (with their `version`) and the implementation
//! artifact or marker coordinates (`buildscript` classpath, convention-build dependencies).

use super::coords::{plugin_by_old_id, swap_coords};
use super::edits::Edits;
use super::scan::{Kind, Script};
use super::{Notes, VERSIONS_2_0_AND_1_8, kotlinter_notes};

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct PluginUse {
    /// An id was rewritten in a `plugins {}` entry (resolved through `pluginManagement`).
    pub declared: bool,
    /// An id was rewritten anywhere.
    pub applied: bool,
    /// Coordinates were rewritten inside `buildscript {}`.
    pub buildscript_coords: bool,
    /// Coordinates were rewritten outside `buildscript {}` (a convention build's dependencies).
    pub dependency_coords: bool,
}

impl PluginUse {
    pub(crate) fn merge(&mut self, o: PluginUse) {
        self.declared |= o.declared;
        self.applied |= o.applied;
        self.buildscript_coords |= o.buildscript_coords;
        self.dependency_coords |= o.dependency_coords;
    }
}

pub(crate) fn rewrite(s: &Script, version: &str, edits: &mut Edits, notes: &mut Notes) -> PluginUse {
    let mut used = PluginUse::default();
    for i in 0..s.toks.len() {
        let Kind::Str { templated } = s.toks[i].kind else { continue };
        let content = &s.text[s.toks[i].inner.clone()];
        if templated {
            if let Some(old) = mentioned_old(content) {
                notes
                    .gradle(format!("`{content}` builds the {old} id or coordinates from a template; swap it by hand"));
            }
            continue;
        }
        if let Some(swap) = plugin_by_old_id(content) {
            if rewrite_id(s, i, swap.new_id, version, edits, notes) {
                used.applied = true;
                used.declared |= is_declaration(s, i);
            }
        } else if let Some((module, old_version)) = swap_coords(content) {
            if let Some(v) = old_version {
                kotlinter_notes::check_plugin_version(content, v, notes);
            }
            edits.replace(s.toks[i].inner.clone(), format!("{module}:{version}"));
            if s.enclosing(i).iter().any(|&o| s.block_name(o) == Some("buildscript")) {
                used.buildscript_coords = true;
            } else {
                used.dependency_coords = true;
            }
        }
    }
    used
}

fn mentioned_old(content: &str) -> Option<&'static str> {
    super::coords::GRADLE_PLUGINS
        .iter()
        .find(|p| content.contains(p.old_id) || content.contains(p.old_artifact))
        .map(|p| p.old_id)
}

/// `id("x")`, `id "x"`, `id 'x'`.
fn is_declaration(s: &Script, i: usize) -> bool {
    (s.is_punct(i.wrapping_sub(1), '(') && s.is_ident(i.wrapping_sub(2), "id")) || s.is_ident(i.wrapping_sub(1), "id")
}

/// Renames the id at `i` and, in a declaration, its `version` literal; `false` (with a note) when the version
/// isn't a literal.
fn rewrite_id(s: &Script, i: usize, new_id: &str, version: &str, edits: &mut Edits, notes: &mut Notes) -> bool {
    if is_declaration(s, i) {
        let mut j = i + 1;
        if s.is_punct(j, ')') {
            j += 1;
        }
        if s.is_punct(j, '.') {
            j += 1;
        }
        if s.is_ident(j, "version") {
            let k = if s.is_punct(j + 1, '(') { j + 2 } else { j + 1 };
            let old = s.plain_str(i).unwrap_or_default();
            let Some(old_version) = s.plain_str(k) else {
                notes.gradle(format!(
                    "plugin {old}: its version isn't a string literal; set id(\"{new_id}\") version \"{version}\" by hand"
                ));
                return false;
            };
            kotlinter_notes::check_plugin_version(old, old_version, notes);
            edits.replace(s.toks[k].inner.clone(), version);
        }
    }
    edits.replace(s.toks[i].inner.clone(), new_id);
    true
}

/// ktlint-gradle's `ktlint { version = ".." }`: the drop-in runs only 1.8.0 and 2.0.0-ALPHA-4.
pub(crate) fn check_ktlint_version(s: &Script, notes: &mut Notes) {
    for open in (0..s.toks.len()).filter(|&o| s.is_punct(o, '{')) {
        let is_ktlint = s.block_name(open) == Some("ktlint") || s.block_type_arg(open) == Some("KtlintExtension");
        if !is_ktlint
            || s.enclosing(open)
                .iter()
                .any(|&o| s.block_name(o) != Some("subprojects") && s.block_name(o) != Some("allprojects"))
        {
            continue;
        }
        for i in open + 1..s.close_of(open) {
            if !s.is_ident(i, "version") || s.is_punct(i.wrapping_sub(1), '.') {
                continue;
            }
            let value = if s.is_punct(i + 1, '=') {
                i + 2
            } else if s.is_punct(i + 1, '.') && s.is_ident(i + 2, "set") && s.is_punct(i + 3, '(') {
                i + 4
            } else {
                continue;
            };
            if let Some(v) = s.plain_str(value).filter(|v| !VERSIONS_2_0_AND_1_8.contains(v)) {
                notes.gradle(format!(
                    "ktlint {{ version = \"{v}\" }}: the ktlint-gradle drop-in runs only 1.8.0 or 2.0.0-ALPHA-4; pick one"
                ));
            }
        }
    }
}
