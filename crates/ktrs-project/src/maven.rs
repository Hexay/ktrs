//! Maven builds: the `pom.xml` chain from the build root down to the module (inherited `<build><plugins>`,
//! `<pluginManagement>` configuration, `<properties>`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::findings::{KtfmtPartial, KtlintPartial};
use crate::reader::{Reader, display};
use crate::xml::{self, Element};
use crate::{FormatTool, ProjectConfig};

const KTLINT_GROUPS: [&str; 4] = ["com.pinterest", "com.pinterest.ktlint", "io.github.ktlint.core", "com.github.shyiko"];

struct Pom {
    props: HashMap<String, String>,
}

impl Pom {
    /// Text with `${name}` properties substituted.
    fn text(&self, e: &Element) -> String {
        let mut s = e.text.trim().to_string();
        for _ in 0..4 {
            let Some(start) = s.find("${") else { break };
            let Some(len) = s[start..].find('}') else { break };
            let Some(value) = self.props.get(&s[start + 2..start + len]) else { break };
            s.replace_range(start..start + len + 1, value);
        }
        s
    }

    fn child_text(&self, e: &Element, name: &str) -> Option<String> {
        e.child(name).map(|c| self.text(c))
    }

    fn coords(&self, dep: &Element) -> String {
        let parts = ["groupId", "artifactId", "version", "classifier"].map(|n| self.child_text(dep, n));
        parts.into_iter().flatten().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(":")
    }
}

pub(crate) fn analyze(root: &Path, pom: &Path, reader: &mut Reader) -> ProjectConfig {
    let module_dir = pom.parent().unwrap_or(root);
    let mut chain: Vec<PathBuf> = module_dir
        .ancestors()
        .take_while(|d| d.starts_with(root))
        .map(|d| d.join("pom.xml"))
        .filter(|p| p.is_file())
        .collect();
    chain.reverse();
    let docs: Vec<(String, Element)> = chain
        .iter()
        .filter_map(|p| Some((display(root, p), xml::parse(&reader.read(p)?).child("project")?.clone())))
        .collect();

    let mut ctx = Pom { props: HashMap::new() };
    let mut managed: HashMap<String, &Element> = HashMap::new();
    let mut plugins: Vec<(&str, &Element)> = Vec::new();
    for (label, project) in &docs {
        for p in project.child("properties").into_iter().flat_map(|p| &p.children) {
            let value = ctx.text(p);
            ctx.props.insert(p.name.clone(), value);
        }
        for p in project.path(&["build", "pluginManagement", "plugins"]).into_iter().flat_map(|p| &p.children) {
            managed.insert(ctx.child_text(p, "artifactId").unwrap_or_default(), p);
        }
        for p in project.path(&["build", "plugins"]).into_iter().flat_map(|p| p.children_named("plugin")) {
            plugins.push((label, p));
        }
    }

    let mut notes = Vec::new();
    let mut ktfmt: Option<KtfmtPartial> = None;
    let mut ktlint: Option<KtlintConfigDraft> = None;
    for (label, plugin) in plugins {
        let artifact = ctx.child_text(plugin, "artifactId").unwrap_or_default();
        let sources: Vec<&Element> = managed.get(&artifact).copied().into_iter().chain([plugin]).collect();
        let config = configuration(&sources);
        let deps: Vec<String> = sources
            .iter()
            .filter_map(|s| s.child("dependencies"))
            .flat_map(|d| d.children_named("dependency"))
            .map(|d| ctx.coords(d))
            .collect();
        match artifact.as_str() {
            "ktlint-maven-plugin" | "ktrs-ktlint-maven-plugin" => {
                let ours = artifact.starts_with("ktrs");
                let flag = |n: &str| config.iter().find(|c| c.name == n).and_then(|c| ctx.text(c).parse().ok());
                let mut p = KtlintPartial { android: flag("android"), experimental: flag("experimental"), ..Default::default() };
                if ours {
                    let version = config.iter().find(|c| c.name == "ktlintVersion").map(|c| ctx.text(c));
                    p.version = Some(version.unwrap_or_else(|| "1.8.0".into()));
                }
                deps.iter().filter(|d| !is_ktlint_runtime(d)).for_each(|d| p.add_rule_set(d));
                notes.push(format!("{label}: {artifact}"));
                ktlint = Some(KtlintConfigDraft { partial: p, tool: artifact.clone() });
            }
            "spotless-maven-plugin" => {
                let Some(kotlin) = config.iter().find(|c| c.name == "kotlin") else { continue };
                if let Some(k) = kotlin.child("ktfmt") {
                    let mut partial = KtfmtPartial::default();
                    for c in k.children.iter().filter(|c| c.name != "version") {
                        partial.set(&c.name, &ctx.text(c));
                    }
                    notes.push(format!("{label}: spotless-maven-plugin <ktfmt>{}", implementation(k)));
                    ktfmt = Some(partial);
                }
                if let Some(k) = kotlin.child("ktlint") {
                    let mut p = KtlintPartial { version: ctx.child_text(k, "version"), ..Default::default() };
                    for o in k.child("editorConfigOverride").into_iter().flat_map(|o| &o.children) {
                        p.set_override(&o.name, &ctx.text(o));
                    }
                    for r in k.child("customRuleSets").into_iter().flat_map(|r| &r.children) {
                        p.add_rule_set(&ctx.text(r));
                    }
                    notes.push(format!("{label}: spotless-maven-plugin <ktlint>{}", implementation(k)));
                    ktlint = Some(KtlintConfigDraft { partial: p, tool: "Spotless <ktlint>".into() });
                }
            }
            "maven-antrun-plugin" | "exec-maven-plugin" => {
                let text: String = sources.iter().map(|s| s.all_text()).collect();
                let cli = deps.iter().find(|d| is_ktlint_cli(d));
                if !text.contains("ktlint.Main") && !text.contains("ktlint.core.Main") && cli.is_none() {
                    continue;
                }
                let mut p = KtlintPartial { version: cli.and_then(|c| c.split(':').nth(2)).map(str::to_string), ..Default::default() };
                deps.iter().filter(|d| !is_ktlint_runtime(d)).for_each(|d| p.add_rule_set(d));
                notes.push(format!("{label}: {artifact} running the ktlint CLI"));
                ktlint = Some(KtlintConfigDraft { partial: p, tool: artifact.clone() });
            }
            _ => {}
        }
    }
    let ktlint = ktlint.map(|d| d.partial.finish(&d.tool, &mut notes));
    if ktfmt.is_some() && ktlint.is_some() {
        notes.push("both ktfmt and ktlint configured: formatting with ktfmt".into());
    }
    let format = match ktfmt {
        Some(k) => Some(FormatTool::Ktfmt(k.finish())),
        None => ktlint.as_ref().map(|_| FormatTool::Ktlint),
    };
    ProjectConfig { root: root.to_path_buf(), format, ktlint, notes }
}

struct KtlintConfigDraft {
    partial: KtlintPartial,
    tool: String,
}

/// Plugin-level then execution-level `<configuration>` children, later ones replacing same-named earlier ones.
fn configuration<'e>(sources: &[&'e Element]) -> Vec<&'e Element> {
    let mut out: Vec<&Element> = Vec::new();
    for source in sources {
        let executions = source.child("executions").into_iter().flat_map(|e| e.children_named("execution"));
        let configs = source.child("configuration").into_iter().chain(executions.filter_map(|e| e.child("configuration")));
        for c in configs.flat_map(|c| &c.children) {
            out.retain(|o| o.name != c.name);
            out.push(c);
        }
    }
    out
}

fn implementation(e: &Element) -> String {
    e.attr("implementation").map(|i| format!(" (implementation {i})")).unwrap_or_default()
}

fn is_ktlint_runtime(coords: &str) -> bool {
    let group = coords.split(':').next().unwrap_or_default();
    KTLINT_GROUPS.iter().any(|g| group == *g || group.starts_with(&format!("{g}."))) || group == "io.github.hexay"
}

fn is_ktlint_cli(coords: &str) -> bool {
    let mut parts = coords.split(':');
    KTLINT_GROUPS.contains(&parts.next().unwrap_or_default())
        && matches!(parts.next(), Some("ktlint" | "ktlint-cli"))
}
