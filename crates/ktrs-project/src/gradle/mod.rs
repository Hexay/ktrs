//! Gradle builds: which sources apply to a module (see the crate docs), interpreted in override order.

mod conventions;
mod findings;
mod interpret;
mod plugins;
mod steps;

use std::collections::HashMap;
use std::path::Path;

use conventions::{Convention, Conventions};
use findings::Findings;
use interpret::{Ctx, Interp};

use crate::catalog::Catalog;
use crate::ir::{Expr, Scope, Stmt};
use crate::layout::{GRADLE_BUILD_FILES, GRADLE_SETTINGS_FILES, first_existing};
use crate::reader::{Reader, display};
use crate::{ProjectConfig, groovy, kotlin_dsl};

pub(crate) fn parse_script(path: &Path, text: &str) -> Vec<Stmt> {
    if path.extension().is_some_and(|e| e == "kts") { kotlin_dsl::parse(text, true) } else { groovy::parse(text) }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// The root build file seen from another module: only `allprojects {}`, `subprojects {}` and Spotless
    /// sections targeting `**`.
    Root,
    RootProject,
    Module,
}

const MAX_APPLY_FROM_DEPTH: usize = 3;

struct Build<'a> {
    root: &'a Path,
    props: HashMap<String, String>,
    catalog: Catalog,
    conventions: Conventions,
}

pub(crate) fn analyze(root: &Path, module: Option<&Path>, script: bool, reader: &mut Reader) -> ProjectConfig {
    let mut props = properties(&reader.read(&root.join("gradle.properties")).unwrap_or_default());
    props.insert("rootDir".into(), root.to_string_lossy().into_owned());
    let catalog = Catalog::parse(&reader.read(&root.join("gradle/libs.versions.toml")).unwrap_or_default());
    let settings = first_existing(root, &GRADLE_SETTINGS_FILES)
        .and_then(|p| Some(parse_script(&p, &reader.read(&p)?)))
        .unwrap_or_default();
    let conventions = Conventions::discover(root, &settings, &props, &catalog, reader);
    let build = Build { root, props, catalog, conventions };

    let root_build = first_existing(root, &GRADLE_BUILD_FILES);
    let mut findings = Findings::default();
    if let Some(rb) = root_build.as_deref().filter(|rb| module != Some(*rb)) {
        findings.merge(build.file(rb, Mode::Root, reader, 0));
    }
    let module_findings = match module {
        Some(m) if root_build.as_deref() == Some(m) => build.file(m, Mode::RootProject, reader, 0),
        Some(m) => build.file(m, Mode::Module, reader, 0),
        None => Findings::default(),
    };
    let mut done: Vec<String> = Vec::new();
    while let Some(id) = findings.plugins.iter().chain(&module_findings.plugins).find(|id| !done.contains(id)).cloned()
    {
        match build.conventions.find(&id) {
            Some(Convention::Script(path)) => findings.merge(build.file(path, Mode::Module, reader, 0)),
            Some(Convention::Class(path, body)) => findings.merge(build.class(path, body)),
            None => {}
        }
        done.push(id);
    }
    findings.merge(module_findings);
    let plugin_props = properties(&reader.read(&root.join("ktlint-plugins.properties")).unwrap_or_default());
    if let Some(v) = plugin_props.get("ktlint-version") {
        findings.default_ktlint_gradle_version(v);
    }
    let (format, ktlint, notes) = findings.resolve(script);
    ProjectConfig { root: root.to_path_buf(), format, ktlint, notes }
}

impl Build<'_> {
    fn funs<'s>(&'s self, stmts: &'s [Stmt]) -> HashMap<&'s str, &'s [Stmt]> {
        let mut funs: HashMap<&str, &[Stmt]> =
            self.conventions.funs.iter().map(|(k, v)| (k.as_str(), v.as_slice())).collect();
        for stmt in stmts {
            if let Stmt::Fun(name, body) = stmt {
                funs.insert(name, body);
            }
        }
        funs
    }

    fn file(&self, path: &Path, mode: Mode, reader: &mut Reader, depth: usize) -> Findings {
        let Some(text) = reader.read(path) else { return Findings::default() };
        let stmts = parse_script(path, &text);
        let scope = Scope::new(&stmts, &self.props, &self.catalog);
        let mut it = Interp::new(&scope, self.funs(&stmts), display(self.root, path));
        it.root_project = mode == Mode::RootProject;
        if mode == Mode::Root {
            run_root(&mut it, &stmts);
        } else {
            it.stmts(&stmts, Ctx::Top);
        }
        let applied = std::mem::take(&mut it.applied_from);
        let mut out = it.out;
        if depth < MAX_APPLY_FROM_DEPTH {
            let dir = path.parent().unwrap_or(self.root);
            for rel in applied {
                out.merge(self.file(&dir.join(rel), Mode::Module, reader, depth + 1));
            }
        }
        out
    }

    fn class(&self, path: &Path, apply_body: &[Stmt]) -> Findings {
        let scope = Scope::new(apply_body, &self.props, &self.catalog);
        let mut it = Interp::new(&scope, self.funs(&[]), display(self.root, path));
        it.stmts(apply_body, Ctx::Top);
        it.out
    }
}

fn run_root(it: &mut Interp, stmts: &[Stmt]) {
    let (shared, own): (Vec<&Stmt>, Vec<&Stmt>) = stmts.iter().partition(|s| {
        matches!(s, Stmt::Expr(Expr::Path(segs))
            if segs.first().is_some_and(|s| matches!(s.name.as_str(), "allprojects" | "subprojects")))
    });
    for stmt in shared {
        it.stmts(std::slice::from_ref(stmt), Ctx::Top);
    }
    let saved = std::mem::take(&mut it.out);
    for stmt in own {
        it.stmts(std::slice::from_ref(stmt), Ctx::Top);
    }
    let own = std::mem::replace(&mut it.out, saved);
    it.out.merge(own.root_wide());
}

fn properties(text: &str) -> HashMap<String, String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#') && !l.starts_with('!'))
        .filter_map(|l| l.split_once(['=', ':']))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}
