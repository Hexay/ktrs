//! A Gradle build: which files to rewrite. The drop-in plugins resolve from the Plugin Portal, like the plugins they
//! replace, so no repository changes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::coords::{GRADLE_PLUGIN_ARTIFACT, no_drop_in};
use super::gradle_plugins::{self, PluginUse};
use super::scan::Script;
use super::{Doc, Notes, catalog_edits, spotless_gradle};
use crate::catalog::Catalog;
use crate::gradle::conventions::SKIPPED_DIRS;
use crate::gradle::properties;

const MAX_DEPTH: usize = 12;

pub(crate) fn migrate(root: &Path, version: &str, notes: &mut Notes) -> Vec<Doc> {
    let mut files = Vec::new();
    walk(root, 0, &mut files);
    let mut docs: Vec<Doc> = files.iter().filter_map(|f| Doc::load(root, f)).collect();
    let conventions = convention_dirs(root, &docs);
    let props = properties(&std::fs::read_to_string(root.join("gradle.properties")).unwrap_or_default());
    let catalog_path = root.join("gradle/libs.versions.toml");
    let mut catalog_doc = Doc::load(root, &catalog_path);
    let catalog = Catalog::parse(catalog_doc.as_ref().map_or("", |d| d.text.as_str()));

    let mut uses: Vec<PluginUse> = Vec::with_capacity(docs.len());
    for doc in &mut docs {
        notes.file(&doc.rel);
        let s = Script::new(&doc.text);
        let in_convention_src = conventions.iter().any(|c| in_src(c, &doc.path));
        if doc.path.extension().is_some_and(|e| e == "kt") && !in_convention_src {
            uses.push(PluginUse::default());
            continue;
        }
        uses.push(gradle_plugins::rewrite(&s, version, &mut doc.edits, notes));
        gradle_plugins::check_ktlint_version(&s, notes);
        let resolver = |src: &str| resolve(src, &s, &props, &catalog);
        if spotless_gradle::rewrite(&s, &resolver, in_convention_src, &mut doc.edits, notes) {
            spotless_gradle::ensure_classpath(&s, version, &mut doc.edits);
        }
        if let Some(advice) = no_drop_in(doc.text.as_str()) {
            notes.drop_in(advice.to_string());
        }
    }
    let swapped = match catalog_doc.as_mut() {
        Some(doc) => {
            notes.file(&doc.rel);
            catalog_edits::rewrite(&doc.text.clone(), version, &mut doc.edits, notes)
        }
        None => catalog_edits::Swapped::default(),
    };
    let mentions = |doc: &Doc| swapped.accessors.iter().any(|a| accessor_used(&doc.text, a));

    for conv in &conventions {
        let sources: Vec<usize> = (0..docs.len()).filter(|&k| in_src(conv, &docs[k].path) && uses[k].applied).collect();
        let builds: Vec<usize> =
            (0..docs.len()).filter(|&k| docs[k].path.starts_with(conv) && !in_src(conv, &docs[k].path)).collect();
        let resolved = builds.iter().any(|&k| uses[k].dependency_coords || mentions(&docs[k]));
        if !sources.is_empty() && !resolved {
            for &k in &sources {
                notes.file(&docs[k].rel);
                notes.gradle(format!(
                    "applies ktfmt-gradle/ktlint-gradle, but its build's dependency on the plugin isn't a literal or catalog entry; switch it to {GRADLE_PLUGIN_ARTIFACT}:{version} and the ids by hand"
                ));
                docs[k].edits = Default::default();
                uses[k] = PluginUse::default();
            }
        }
    }
    let total = uses.iter().fold(PluginUse::default(), |mut a, u| {
        a.merge(*u);
        a
    });
    let unresolved = !total.declared && !total.buildscript_coords && !total.dependency_coords;
    if let Some(k) = (0..docs.len()).find(|&k| uses[k].applied).filter(|_| unresolved && swapped.accessors.is_empty()) {
        notes.file(&docs[k].rel);
        notes.gradle(format!(
            "the build applies ktfmt-gradle/ktlint-gradle by id, but no plugins {{}} version, classpath or catalog entry for it was found; declare {GRADLE_PLUGIN_ARTIFACT}:{version} where the old plugin came from"
        ));
    }
    docs.extend(catalog_doc);
    docs
}

/// `libs.plugins.ktfmt` matches `libs.plugins.ktfmt`, `libs.plugins.ktfmt.get()`, not `libs.plugins.ktfmtx`.
fn accessor_used(text: &str, accessor: &str) -> bool {
    let norm = text.replace(['-', '_'], ".");
    norm.match_indices(accessor).any(|(i, _)| !norm[i + accessor.len()..].starts_with(|c: char| c.is_alphanumeric()))
}

/// Whether `path` is a source of convention build `conv` (under a `src` dir of it or of a nested project).
fn in_src(conv: &Path, path: &Path) -> bool {
    path.strip_prefix(conv).is_ok_and(|rel| rel.components().any(|c| c.as_os_str() == "src"))
}

/// `buildSrc`, `build-logic` and `includeBuild("..")` dirs.
fn convention_dirs(root: &Path, docs: &[Doc]) -> Vec<PathBuf> {
    let mut dirs = vec![root.join("buildSrc"), root.join("build-logic")];
    for doc in docs.iter().filter(|d| d.path.parent() == Some(root) && d.rel.starts_with("settings.gradle")) {
        let s = Script::new(&doc.text);
        for i in 0..s.toks.len() {
            if s.is_ident(i, "includeBuild")
                && let Some(dir) = s.plain_str(i + 2).filter(|_| s.is_punct(i + 1, '(')).or_else(|| s.plain_str(i + 1))
            {
                dirs.push(root.join(dir));
            }
        }
    }
    dirs.retain(|d| d.is_dir());
    let mut dirs: Vec<PathBuf> = dirs.iter().map(|d| std::path::absolute(d).unwrap_or_else(|_| d.clone())).collect();
    dirs.sort();
    dirs.dedup();
    dirs
}

fn walk(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if path.is_dir() {
            if depth < MAX_DEPTH && !SKIPPED_DIRS.contains(&name) {
                walk(&path, depth + 1, files);
            }
        } else if name.ends_with(".gradle.kts")
            || name.ends_with(".gradle")
            || name.ends_with(".kt") && is_build_logic(&path)
        {
            files.push(path);
        }
    }
}

fn is_build_logic(path: &Path) -> bool {
    path.components().any(|c| matches!(c.as_os_str().to_str(), Some("buildSrc" | "build-logic")))
}

/// A version expression: a string `val`/`def` of the file, a `gradle.properties` key, `libs.versions.x`.
fn resolve(src: &str, s: &Script, props: &HashMap<String, String>, catalog: &Catalog) -> Option<String> {
    let expr = src.trim().trim_end_matches(".get()").trim_end_matches(".toString()");
    if let Some(alias) = expr.strip_prefix("libs.versions.") {
        return catalog.version(alias).map(str::to_string);
    }
    if !expr.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let local = (0..s.toks.len()).find_map(|i| {
        let decl = s.is_ident(i, "val") || s.is_ident(i, "def") || s.is_ident(i, "var");
        (decl && s.is_ident(i + 1, expr) && s.is_punct(i + 2, '=')).then(|| s.plain_str(i + 3)).flatten()
    });
    local.map(str::to_string).or_else(|| props.get(expr).cloned())
}
