//! Maven (README "Integrations > Maven"): gantsign's ktlint-maven-plugin -> `io.github.hexay:
//! ktrs-ktlint-maven-plugin`; Spotless `<kotlin>` `<ktfmt>`/`<ktlint>` get `implementation=` and the plugin the
//! `io.github.hexay:ktrs` dependency.

use std::collections::HashMap;

use super::coords::{JAR_ARTIFACT, JAR_GROUP, KTLINT_JAR, no_drop_in};
use super::edits::{indent_at, indent_unit, line_start, starts_line};
use super::pom::{El, parse};
use super::{Doc, Notes, VERSIONS_2_0_AND_1_8};

const GANTSIGN: (&str, &str) = ("com.github.gantsign.maven", "ktlint-maven-plugin");
const DROP_IN: (&str, &str) = ("io.github.hexay", "ktrs-ktlint-maven-plugin");
const KTFMT_IMPL: &str = "io.github.hexay.ktrs.spotless.maven.KtrsKtfmt";
const KTLINT_IMPL: &str = "io.github.hexay.ktrs.spotless.maven.KtrsKtlint";

/// Rewrites every pom of the build in place (`docs`), `properties` merged from all of them.
pub(crate) fn rewrite(docs: &mut [Doc], version: &str, notes: &mut Notes) {
    let trees: Vec<El> = docs.iter().map(|d| parse(&d.text)).collect();
    let mut properties = HashMap::new();
    for (doc, tree) in docs.iter().zip(&trees) {
        if let Some(props) = tree.path(&["project", "properties"]) {
            for p in &props.children {
                properties.entry(p.name.clone()).or_insert_with(|| p.text(&doc.text).to_string());
            }
        }
    }
    let all_text: String = docs.iter().map(|d| d.text.as_str()).collect();
    for (doc, tree) in docs.iter_mut().zip(&trees) {
        notes.file(&doc.rel);
        let src = doc.text.clone();
        let ktlint_cli = || src.contains("<artifactId>ktlint-cli</artifactId>").then_some(KTLINT_JAR);
        if let Some(advice) = no_drop_in(&src).or_else(ktlint_cli) {
            notes.drop_in(advice.to_string());
        }
        let mut plugins = Vec::new();
        tree.descendants("plugin", &mut plugins);
        for plugin in plugins {
            let coords = (text_of(plugin, "groupId", &src), text_of(plugin, "artifactId", &src));
            if coords == (Some(GANTSIGN.0), Some(GANTSIGN.1)) {
                gantsign(doc, plugin, &src, tree, &all_text, version, notes);
            } else if coords.1 == Some("spotless-maven-plugin") {
                spotless(doc, plugin, &src, &properties, version, notes);
            }
        }
    }
}

fn text_of<'a>(el: &El, child: &str, src: &'a str) -> Option<&'a str> {
    el.child(child).map(|c| c.text(src))
}

fn gantsign(doc: &mut Doc, plugin: &El, src: &str, tree: &El, all_text: &str, version: &str, notes: &mut Notes) {
    let mut version_edit = None;
    if let Some(v) = plugin.child("version") {
        let text = v.text(src);
        version_edit = match text.strip_prefix("${").and_then(|p| p.strip_suffix('}')) {
            None => Some(v.text_span(src)),
            Some(prop) => {
                let def = tree.path(&["project", "properties", prop]);
                match def.filter(|_| all_text.matches(text).count() == 1) {
                    Some(def) => Some(def.text_span(src)),
                    None => {
                        notes.maven(format!(
                            "ktlint-maven-plugin's version {text} is shared or defined elsewhere; set <groupId>{}</groupId>, <artifactId>{}</artifactId>, <version>{version}</version> by hand",
                            DROP_IN.0, DROP_IN.1
                        ));
                        return;
                    }
                }
            }
        };
    }
    let (Some(g), Some(a)) = (plugin.child("groupId"), plugin.child("artifactId")) else { return };
    doc.edits.replace(g.text_span(src), DROP_IN.0);
    doc.edits.replace(a.text_span(src), DROP_IN.1);
    if let Some(span) = version_edit {
        doc.edits.replace(span, version);
    }
}

fn spotless(doc: &mut Doc, plugin: &El, src: &str, props: &HashMap<String, String>, version: &str, notes: &mut Notes) {
    let mut kotlins = Vec::new();
    plugin.descendants("kotlin", &mut kotlins);
    let mut swapped = false;
    for kotlin in kotlins {
        for step in kotlin.children.iter().filter(|c| c.name == "ktfmt" || c.name == "ktlint") {
            if step.has_attr(src, "implementation") {
                continue;
            }
            if let Some(v) = step.child("version") {
                let raw = v.text(src);
                let resolved = raw
                    .strip_prefix("${")
                    .and_then(|p| p.strip_suffix('}'))
                    .map_or(Some(raw), |p| props.get(p).map(String::as_str));
                let ok = match (step.name.as_str(), resolved) {
                    ("ktfmt", Some(v)) => v == "0.64" || v.starts_with("0.64."),
                    (_, Some(v)) => VERSIONS_2_0_AND_1_8.contains(&v),
                    (_, None) => false,
                };
                if !ok {
                    notes.maven(format!(
                        "Spotless <{}> version {raw}: ktrs matches ktfmt 0.64 and ktlint 1.8.0 / 2.0.0-ALPHA-4; change or drop <version>, then rerun",
                        step.name
                    ));
                    continue;
                }
            }
            let class = if step.name == "ktfmt" { KTFMT_IMPL } else { KTLINT_IMPL };
            doc.edits.insert(step.open.start + 1 + step.name.len(), format!(" implementation=\"{class}\""));
            swapped = true;
        }
    }
    if swapped && !has_jar_dependency(plugin, src) {
        add_jar_dependency(doc, plugin, src, version);
    }
}

fn has_jar_dependency(plugin: &El, src: &str) -> bool {
    plugin.child("dependencies").is_some_and(|deps| {
        deps.children.iter().any(|d| {
            text_of(d, "groupId", src) == Some(JAR_GROUP) && text_of(d, "artifactId", src) == Some(JAR_ARTIFACT)
        })
    })
}

fn add_jar_dependency(doc: &mut Doc, plugin: &El, src: &str, version: &str) {
    let dependency = format!(
        "<dependency><groupId>{JAR_GROUP}</groupId><artifactId>{JAR_ARTIFACT}</artifactId><version>{version}</version></dependency>"
    );
    let child_indent = |el: &El| match el.children.first() {
        Some(c) => indent_at(src, c.open.start).to_string(),
        None => format!("{}{}", indent_at(src, el.open.start), indent_unit(src)),
    };
    let (target, text) = match plugin.child("dependencies") {
        Some(deps) => (deps, dependency),
        None => {
            let ci = child_indent(plugin);
            let step = ci
                .strip_prefix(indent_at(src, plugin.open.start))
                .filter(|s| !s.is_empty())
                .map_or_else(|| indent_unit(src), str::to_string);
            (plugin, format!("<dependencies>\n{ci}{step}{dependency}\n{ci}</dependencies>"))
        }
    };
    if starts_line(src, target.close.start) {
        doc.edits.insert(line_start(src, target.close.start), format!("{}{text}\n", child_indent(target)));
    } else {
        doc.edits.insert(target.close.start, text);
    }
}
