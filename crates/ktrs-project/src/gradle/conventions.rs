//! Convention plugins of a build: precompiled script plugins and registered plugin classes in `buildSrc`,
//! `build-logic` and `includeBuild(..)` dirs, plus the functions their Kotlin sources declare.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::parse_script;
use crate::catalog::Catalog;
use crate::ir::{Expr, Scope, Seg, Stmt};
use crate::kotlin_dsl;
use crate::reader::Reader;

#[derive(Default)]
pub(crate) struct Conventions {
    scripts: HashMap<String, PathBuf>,
    /// Plugin id -> implementation class simple name.
    binaries: HashMap<String, String>,
    classes: HashMap<String, (PathBuf, Vec<Stmt>)>,
    pub funs: HashMap<String, Vec<Stmt>>,
}

pub(crate) enum Convention<'c> {
    Script(&'c Path),
    /// A plugin class's file and its `apply` body.
    Class(&'c Path, &'c [Stmt]),
}

pub(crate) const SKIPPED_DIRS: [&str; 7] = ["build", ".gradle", ".git", "out", "node_modules", ".idea", "test"];
const MAX_DEPTH: usize = 10;

impl Conventions {
    pub(crate) fn discover(
        root: &Path,
        settings: &[Stmt],
        props: &HashMap<String, String>,
        catalog: &Catalog,
        reader: &mut Reader,
    ) -> Conventions {
        let mut dirs = vec![root.join("buildSrc"), root.join("build-logic")];
        let scope = Scope::new(settings, props, catalog);
        let mut include_builds = Vec::new();
        find_calls(settings, "includeBuild", &mut include_builds);
        for seg in include_builds {
            if let Some(dir) = seg.arg(0).and_then(|a| scope.str(a)) {
                dirs.push(root.join(dir));
            }
        }
        let mut files = Vec::new();
        let mut seen = Vec::new();
        for dir in dirs.iter().filter(|d| d.is_dir()) {
            let canonical = std::path::absolute(dir).unwrap_or_else(|_| dir.clone());
            if !seen.contains(&canonical) {
                walk(dir, 0, &mut files);
                seen.push(canonical);
            }
        }
        let mut c = Conventions::default();
        for file in files {
            c.add_file(&file, props, catalog, reader);
        }
        c
    }

    fn add_file(&mut self, file: &Path, props: &HashMap<String, String>, catalog: &Catalog, reader: &mut Reader) {
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        let in_src = file.components().any(|c| c.as_os_str() == "src");
        let script_plugin = name.ends_with(".gradle.kts") || name.ends_with(".gradle");
        if in_src && script_plugin {
            let stem = name.trim_end_matches(".kts").trim_end_matches(".gradle").to_string();
            if name.ends_with(".kts")
                && let Some(package) = reader.read(file).as_deref().and_then(package_of)
            {
                self.scripts.insert(format!("{package}.{stem}"), file.to_path_buf());
            }
            self.scripts.insert(stem, file.to_path_buf());
        } else if !in_src && name.starts_with("build.gradle") {
            let Some(text) = reader.read(file) else { return };
            let stmts = parse_script(file, &text);
            let scope = Scope::new(&stmts, props, catalog);
            registrations(&stmts, &scope, &mut self.binaries);
        } else if in_src && name.ends_with(".kt") {
            let Some(text) = reader.read(file) else { return };
            for stmt in kotlin_dsl::parse(&text, false) {
                self.add_declaration(file, stmt);
            }
        }
    }

    fn add_declaration(&mut self, file: &Path, stmt: Stmt) {
        match stmt {
            Stmt::Fun(name, body) => {
                self.funs.insert(name, body);
            }
            Stmt::Class(name, body) => {
                for s in &body {
                    if let Stmt::Fun(f, b) = s
                        && f != "apply"
                    {
                        self.funs.insert(f.clone(), b.clone());
                    }
                }
                self.classes.insert(name, (file.to_path_buf(), body));
            }
            _ => {}
        }
    }

    pub(crate) fn find(&self, id: &str) -> Option<Convention<'_>> {
        if let Some(path) = self.scripts.get(id) {
            return Some(Convention::Script(path));
        }
        let (path, body) = self.classes.get(self.binaries.get(id)?)?;
        body.iter().find_map(|s| match s {
            Stmt::Fun(name, apply) if name == "apply" => Some(Convention::Class(path, apply)),
            _ => None,
        })
    }
}

fn package_of(text: &str) -> Option<String> {
    let line = text.lines().map(str::trim).find(|l| l.starts_with("package "))?;
    Some(line["package ".len()..].trim().trim_end_matches(';').to_string())
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
        } else if name.ends_with(".kt") || name.ends_with(".gradle.kts") || name.ends_with(".gradle") {
            files.push(path);
        }
    }
}

/// Every call named `name`, at any depth.
pub(crate) fn find_calls<'s>(stmts: &'s [Stmt], name: &str, out: &mut Vec<&'s Seg>) {
    for stmt in stmts {
        if let Stmt::Expr(Expr::Path(segs)) = stmt {
            for seg in segs {
                if seg.name == name && seg.args.is_some() {
                    out.push(seg);
                }
                if let Some(block) = &seg.block {
                    find_calls(block, name, out);
                }
            }
        }
    }
}

/// `gradlePlugin { plugins { register("x") { id = ".."; implementationClass = ".." } } }` (any block that
/// sets both).
fn registrations(stmts: &[Stmt], scope: &Scope, out: &mut HashMap<String, String>) {
    let mut id = None;
    let mut class = None;
    for stmt in stmts {
        let (name, value) = match stmt {
            Stmt::Assign(Expr::Path(lhs), v) if lhs.len() == 1 => (lhs[0].name.as_str(), Some(v)),
            Stmt::Expr(Expr::Path(segs)) => {
                for block in segs.iter().filter_map(|s| s.block.as_deref()) {
                    registrations(block, scope, out);
                }
                match &segs[..] {
                    [s] => (s.name.as_str(), s.arg(0)),
                    [s, set] if set.name == "set" => (s.name.as_str(), set.arg(0)),
                    _ => continue,
                }
            }
            _ => continue,
        };
        match name {
            "id" => id = value.and_then(|v| scope.str(v)),
            "implementationClass" => class = value.and_then(|v| scope.str(v)),
            _ => {}
        }
    }
    if let (Some(id), Some(class)) = (id, class) {
        out.insert(id, class.rsplit('.').next().unwrap_or_default().to_string());
    }
}
